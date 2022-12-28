use std::path::PathBuf;
use std::thread;
use std::time::{ Duration, Instant };

use sdl2::audio::{ AudioQueue, AudioSpecDesired };
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

mod gameboy;
mod bit_logic;

use gameboy::Gameboy;

const WIDTH: u16 = 160;
const HEIGHT: u16 = 144;
const SCREEN_DATA_SIZE: u32 = (WIDTH as u32) * (HEIGHT as u32) * 3;

const CYCLES_PER_SECOND: u32 = 4_194_304;
const FRAMES_PER_SECOND: f64 = 59.727500569606;
const CYCLES_PER_FRAME: f64 = (CYCLES_PER_SECOND as f64) / FRAMES_PER_SECOND;
const TIME_BETWEEN_FRAMES_IN_NANOSECONDS: f64 = (1_000.0 / FRAMES_PER_SECOND) * 1_000_000.0;
const DURATION_BETWEEN_FRAMES: Duration = Duration::from_nanos(TIME_BETWEEN_FRAMES_IN_NANOSECONDS as u64);

const SAMPLE_RATE: u16 = 44_100;
const TIME_BETWEEN_AUDIO_SAMPLING: u8 = (CYCLES_PER_SECOND / SAMPLE_RATE as u32) as u8;

fn main() {
    let rom_path = std::env::args().nth(1).expect("No ROM path given");  

    let sdl_context = sdl2::init().expect("Couldn't init sdl");
    let video_subsystem = sdl_context.video().expect("Couldn't init sdl video");
    let audio_subsystem = sdl_context.audio().expect("Couldn't init sdl audio");

    let window = video_subsystem.window("RPGBE", WIDTH.into(), HEIGHT.into())
        .position_centered()
        .resizable()
        .build()
        .expect("Couldn't create window from video");

    let mut canvas = window.into_canvas()
        .software()
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

    let mut event_pump = sdl_context.event_pump().expect("Couldn't get event_pump from sdl_context");

    let mut gameboy = Gameboy::new();
    gameboy.memory.load_cartridge(PathBuf::from(rom_path));

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(key_down), repeat: false, .. } => {
                    let key_code: i8 = match key_down {
                        Keycode::W => 2, // UP
                        Keycode::A => 1, // LEFT
                        Keycode::S => 3, // DOWN
                        Keycode::D => 0, // RIGHT
                        Keycode::H => 5, // B
                        Keycode::U => 4, // A
                        Keycode::B => 6, // SELECT
                        Keycode::N => 7, // START
                        _ => -1,
                    };
                    if key_code >= 0 {
                        gameboy.key_pressed(key_code as u8);
                    }
                },
                Event::KeyUp { keycode: Some(key_up), repeat: false, .. } => {
                    let key_code: i8 = match key_up {
                        Keycode::W => 2, // UP
                        Keycode::A => 1, // LEFT
                        Keycode::S => 3, // DOWN
                        Keycode::D => 0, // RIGHT
                        Keycode::H => 5, // B
                        Keycode::U => 4, // A
                        Keycode::B => 6, // SELECT
                        Keycode::N => 7, // START
                        _ => -1,
                    };
                    if key_code >= 0 {
                        gameboy.key_released(key_code as u8);
                    }
                },
                _ => (),
            }
        }

        let start = Instant::now();
        let mut cycles_this_frame: f64 = 0.0;
        while cycles_this_frame <= CYCLES_PER_FRAME {
            cycles_this_frame += gameboy.update() as f64;
        }

        texture.update(None, &gameboy.screen_data, WIDTH.wrapping_mul(3) as usize).expect("Couldn't update texture from main");
        canvas.clear();
        canvas.copy(&texture, None, None).expect("Couldn't copy canvas");
        canvas.present();

        device.queue(&gameboy.spu.audio_data);
        gameboy.spu.audio_data.clear();

        let elapsed_time = start.elapsed();
        if elapsed_time <= DURATION_BETWEEN_FRAMES {
            let time_remaining = DURATION_BETWEEN_FRAMES - elapsed_time;
            thread::sleep(time_remaining);
        }
    }
}
