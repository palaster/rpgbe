use std::path::PathBuf;
use std::thread;
use std::time::Instant;

use sdl2::audio::{ AudioQueue, AudioSpecDesired };
use sdl2::controller::Button;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;

mod gameboy;

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

    let number_of_joystics = game_controller_subsystem.num_joysticks().expect("Couldn't find any joysticks");
    let _controller = (0..number_of_joystics)
        .find_map(|id| {
            if !game_controller_subsystem.is_game_controller(id) {
                return None;
            }
            game_controller_subsystem.open(id).ok()
        }).expect("Couldn't open any controllers");

    let window = video_subsystem.window("RPGBE", gameboy::WIDTH.into(), gameboy::HEIGHT.into())
        .position_centered()
        .resizable()
        .build()
        .expect("Couldn't create window from video");

    let mut canvas = window.into_canvas()
        .software()
        .build()
        .expect("Couldn't create canvas from window");

    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, gameboy::WIDTH.into(), gameboy::HEIGHT.into()).expect("Couldn't create texture from texture_creator.create_texture_streaming");

    let desired_spec = AudioSpecDesired {
        freq: Some(gameboy::SAMPLE_RATE as i32),
        channels: Some(2),
        samples: None,
    };

    let device: AudioQueue<f32> = audio_subsystem.open_queue(None, &desired_spec).expect("Couldn't get a desired audio device");
    device.resume();

    let mut event_pump = sdl_context.event_pump().expect("Couldn't get event_pump from sdl_context");

    let mut gameboy = gameboy::Gameboy::new();
    gameboy.memory.load_cartridge(include_bytes!("../../tetris.gb").to_vec());
    //gameboy.memory.load_cartridge_from_path(PathBuf::from(rom_path));

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
                        gameboy.key_pressed(key_code as u8);
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
                        gameboy.key_released(key_code as u8);
                    }
                },
                _ => (),
            }
        }

        let start = Instant::now();
        let mut cycles_this_frame: f64 = 0.0;
        while cycles_this_frame <= gameboy::CYCLES_PER_FRAME {
            cycles_this_frame += gameboy.update() as f64;
        }

        texture.update(None, &gameboy.gpu.screen_data, gameboy::WIDTH.wrapping_mul(3) as usize).expect("Couldn't update texture from main");
        canvas.clear();
        canvas.copy(&texture, None, None).expect("Couldn't copy canvas");
        canvas.present();

        let _ = device.queue_audio(&gameboy.spu.audio_data);
        gameboy.spu.audio_data.clear();

        let elapsed_time = start.elapsed();
        if elapsed_time <= gameboy::DURATION_BETWEEN_FRAMES {
            let time_remaining = gameboy::DURATION_BETWEEN_FRAMES - elapsed_time;
            thread::sleep(time_remaining);
        }
    }
}