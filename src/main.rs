use std::path::PathBuf;
use std::thread;
use std::time::Instant;

use sdl2::audio::{ AudioQueue, AudioSpecDesired };
use sdl2::controller::Button;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;

pub const WIDTH: u16 = 160;
pub const HEIGHT: u16 = 144;

pub const CYCLES_PER_SECOND: u32 = 4_194_304;
pub const FRAMES_PER_SECOND: f64 = 59.727500569606;
pub const CYCLES_PER_FRAME: f64 = (CYCLES_PER_SECOND as f64) / FRAMES_PER_SECOND;
pub const TIME_BETWEEN_FRAMES_IN_NANOSECONDS: f64 = (1_000.0 / FRAMES_PER_SECOND) * 1_000_000.0;
pub const DURATION_BETWEEN_FRAMES: Duration = Duration::from_nanos(TIME_BETWEEN_FRAMES_IN_NANOSECONDS as u64);

pub const SAMPLE_RATE: u16 = 44_100;
pub const TIME_BETWEEN_AUDIO_SAMPLING: u8 = (CYCLES_PER_SECOND / SAMPLE_RATE as u32) as u8;

use std::time::Duration;

const TAC: u16 = 0xff07;

mod bit_logic;
mod cpu;
mod gpu;
mod memory;
mod spu;
mod timer;

use cpu::Cpu;
use memory::Memory;

enum MemoryWriteResult {
    None,
    ResetDividerCounter,
    SetTimerCounter,
    ResetChannel(u8, u8),
}

#[link(name = "SceAudioIn_stub", kind = "static", modifiers = "+whole-archive")]
#[link(name = "SceAudio_stub", kind = "static", modifiers = "+whole-archive")]
#[link(name = "SceCommonDialog_stub", kind = "static", modifiers = "+whole-archive")]
#[link(name = "SceCtrl_stub", kind = "static", modifiers = "+whole-archive")]
#[link(name = "SceDisplay_stub", kind = "static", modifiers = "+whole-archive")]
#[link(name = "SceGxm_stub", kind = "static", modifiers = "+whole-archive")]
#[link(name = "SceHid_stub", kind = "static", modifiers = "+whole-archive")]
#[link(name = "SceMotion_stub", kind = "static", modifiers = "+whole-archive")]
#[link(name = "SceTouch_stub", kind = "static", modifiers = "+whole-archive")]
extern "C" {}

fn main() {
    // let rom_path = std::env::args().nth(1).expect("No ROM path given");

    let sdl_context = sdl2::init().expect("Couldn't init sdl");
    let video_subsystem = sdl_context.video().expect("Couldn't init sdl video");
    let audio_subsystem = sdl_context.audio().expect("Couldn't init sdl audio");
    let game_controller_subsystem = sdl_context.game_controller().expect("Couldn't init sdl game_controller");

    let window = video_subsystem.window("RPGBE", WIDTH.into(), HEIGHT.into())
        .fullscreen_desktop()
        .borderless()
        .build()
        .expect("Couldn't create window from video");

    let mut canvas = window.into_canvas()
        .accelerated()
        .build()
        .expect("Couldn't create canvas from window");

    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, WIDTH.into(), HEIGHT.into()).expect("Couldn't create texture from texture_creator.create_texture_streaming");

    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32),
        channels: Some(2),
        samples: None,
    };

    let device: AudioQueue<f32> = audio_subsystem.open_queue(None, &desired_spec).expect("Couldn't get a desired audio device");
    device.resume();

    let number_of_joystics = game_controller_subsystem.num_joysticks().expect("Couldn't find any joysticks");
    let _controller = (0..number_of_joystics)
        .find_map(|id| {
            if !game_controller_subsystem.is_game_controller(id) {
                return None;
            }
            game_controller_subsystem.open(id).ok()
        }).expect("Couldn't open any controllers");

    let mut event_pump = sdl_context.event_pump().expect("Couldn't get event_pump from sdl_context");

    let mut cpu = cpu::Cpu::new();
    let mut gpu = gpu::Gpu::new();
    let mut memory = memory::Memory::new();
    let mut spu = spu::Spu::new();
    let mut timer = timer::Timer::new();
    memory.load_cartridge(include_bytes!("../../tetris.gb").to_vec());

    //gameboy.memory.load_cartridge_from_path(PathBuf::from(rom_path));

    let mut start: Instant;
    let mut cycles_this_frame: f64;
    let mut cycles: u8;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                Event::ControllerButtonDown { button, .. } => {
                    let key_code: i8 = match button {
                        Button::DPadUp => 2, // UP
                        Button::DPadLeft => 1, // LEFT
                        Button::DPadDown => 3, // DOWN
                        Button::DPadRight => 0, // RIGHT
                        Button::A => 5, // B
                        Button::B => 4, // A
                        Button::Back => 6, // SELECT
                        Button::Start => 7, // START
                        _ => -1,
                    };
                    if key_code >= 0 {
                        key_pressed(&mut memory, key_code as u8);
                    }
                },
                Event::ControllerButtonUp { button, .. } => {
                    let key_code: i8 = match button {
                        Button::DPadUp => 2, // UP
                        Button::DPadLeft => 1, // LEFT
                        Button::DPadDown => 3, // DOWN
                        Button::DPadRight => 0, // RIGHT
                        Button::A => 5, // B
                        Button::B => 4, // A
                        Button::Back => 6, // SELECT
                        Button::Start => 7, // START
                        _ => -1,
                    };
                    if key_code >= 0 {
                        key_released(&mut memory, key_code as u8);
                    }
                },
                _ => (),
            }
        }

        start = Instant::now();
        cycles_this_frame = 0.0;
        while cycles_this_frame <= CYCLES_PER_FRAME {
            cycles = 4;
            if !cpu.halted {
                let (new_cycles, memory_write_results) = cpu.update(&mut memory);
                cycles = new_cycles * 4;
                for memory_result in memory_write_results {
                    match memory_result {
                        MemoryWriteResult::ResetDividerCounter => {
                            timer.divider_counter = 0
                        },
                        MemoryWriteResult::SetTimerCounter => {
                            timer.set_clock_freq(&memory)
                        },
                        MemoryWriteResult::ResetChannel(id, length) => {
                            match id {
                                0 => { spu.sound_channel_1.reset(&memory, length) },
                                1 => { spu.sound_channel_2.reset(&memory, length) },
                                2 => { spu.sound_channel_3.reset(&memory, length) },
                                3 => { spu.sound_channel_4.reset(&memory, length) },
                                _ => { },
                            }
                        },
                        _ => { },
                    }
                }
            }

            for memory_result in timer.update_timer(&mut memory, cycles) {
                match memory_result {
                    MemoryWriteResult::ResetDividerCounter => {
                        timer.divider_counter = 0
                    },
                    MemoryWriteResult::SetTimerCounter => {
                        timer.set_clock_freq(&memory)
                    },
                    MemoryWriteResult::ResetChannel(id, length) => {
                        match id {
                            0 => { spu.sound_channel_1.reset(&memory, length) },
                            1 => { spu.sound_channel_2.reset(&memory, length) },
                            2 => { spu.sound_channel_3.reset(&memory, length) },
                            3 => { spu.sound_channel_4.reset(&memory, length) },
                            _ => { },
                        }
                    },
                    _ => { },
                }
            }
            gpu.update_graphics(&mut memory, cycles);
            if let Some((new_nr13, new_nr14)) = spu.update_audio(&memory, cycles) {
                let mut memory_write_results = memory.write_to_memory(0xff13, new_nr13);
                memory_write_results.append(&mut memory.write_to_memory(0xff14, new_nr14));
                for memory_result in memory_write_results {
                    match memory_result {
                        MemoryWriteResult::ResetDividerCounter => {
                            timer.divider_counter = 0
                        },
                        MemoryWriteResult::SetTimerCounter => {
                            timer.set_clock_freq(&memory)
                        },
                        MemoryWriteResult::ResetChannel(id, length) => {
                            match id {
                                0 => { spu.sound_channel_1.reset(&memory, length) },
                                1 => { spu.sound_channel_2.reset(&memory, length) },
                                2 => { spu.sound_channel_3.reset(&memory, length) },
                                3 => { spu.sound_channel_4.reset(&memory, length) },
                                _ => { },
                            }
                        },
                        _ => { },
                    }
                }
            }

            let (memory_write_results, cycles_taken) = do_interrupts(&mut cpu, &mut memory);
            for memory_result in memory_write_results {
                match memory_result {
                    MemoryWriteResult::ResetDividerCounter => {
                        timer.divider_counter = 0
                    },
                    MemoryWriteResult::SetTimerCounter => {
                        timer.set_clock_freq(&memory)
                    },
                    MemoryWriteResult::ResetChannel(id, length) => {
                        match id {
                            0 => { spu.sound_channel_1.reset(&memory, length) },
                            1 => { spu.sound_channel_2.reset(&memory, length) },
                            2 => { spu.sound_channel_3.reset(&memory, length) },
                            3 => { spu.sound_channel_4.reset(&memory, length) },
                            _ => { },
                        }
                    },
                    _ => { },
                }
            }
            cycles_this_frame += (cycles + cycles_taken) as f64;
        }

        texture.update(None, &gpu.screen_data, WIDTH.wrapping_mul(3) as usize).expect("Couldn't update texture from main");
        canvas.clear();
        canvas.copy(&texture, None, None).expect("Couldn't copy canvas");
        canvas.present();

        let _ = device.queue_audio(&spu.audio_data);
        spu.audio_data.clear();

        let elapsed_time = start.elapsed();
        if elapsed_time <= DURATION_BETWEEN_FRAMES {
            thread::sleep(DURATION_BETWEEN_FRAMES - elapsed_time);
        }
    }
}

fn key_pressed(memory: &mut Memory, key: u8) {
    let previously_unset: bool = !bit_logic::check_bit(memory.gamepad_state, key);

    memory.gamepad_state = bit_logic::reset_bit(memory.gamepad_state, key);

    let button: bool = key > 3;

    let key_req: u8 = memory.rom[0xff00_usize];
    let should_request_interrupt: bool = (button && !bit_logic::check_bit(key_req, 5)) || (!button && !bit_logic::check_bit(key_req, 4));

    if should_request_interrupt && !previously_unset {
        memory.request_interrupt(4);
    }
}

fn key_released(memory: &mut Memory, key: u8) {
    memory.gamepad_state = bit_logic::set_bit(memory.gamepad_state, key);
}

fn service_interrupt(cpu: &mut Cpu, memory: &mut Memory, interrupt_id: u8) -> Vec<MemoryWriteResult> {
    cpu.interrupts_enabled = false;
    //self.write_to_address(0xff0f, bit_logic::reset_bit(memory.read_from_memory(0xff0f), interrupt_id));

    let pc: u16 = cpu.pc;
    cpu.push(memory, (pc >> 8) as u8);
    cpu.push(memory, pc as u8);

    match interrupt_id {
        0 => { cpu.pc = 0x40 },
        1 => { cpu.pc = 0x48 },
        2 => { cpu.pc = 0x50 },
        3 => { cpu.pc = 0x58 },
        4 => { cpu.pc = 0x60 },
        _ => {},
    }
    memory.write_to_memory(0xff0f, bit_logic::reset_bit(memory.read_from_memory(0xff0f), interrupt_id))
}

fn do_interrupts(cpu: &mut Cpu, memory: &mut Memory) -> (Vec<MemoryWriteResult>, u8) {
    let result = Vec::new();
    let (req, enabled): (u8, u8) = (memory.read_from_memory(0xff0f), memory.read_from_memory(0xffff));
    let potential_for_interrupts: u8 = req & enabled;
    if potential_for_interrupts == 0 {
        return (result, 0);
    }
    if cpu.interrupts_enabled {
        cpu.halted = false;
        for i in 0..5 {
            if bit_logic::check_bit(req, i) && bit_logic::check_bit(enabled, i) {
                return (service_interrupt(cpu, memory, i), 20);
            }
        }
    }
    cpu.halted = false;
    (result, 0)
}