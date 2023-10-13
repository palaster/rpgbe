use std::path::PathBuf;
use std::thread;
use std::time::Instant;

use sdl2::audio::{ AudioQueue, AudioSpecDesired };
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

mod gameboy;

#[link(name = "SceAudioIn_stub")]
#[link(name = "SceAudio_stub")]
#[link(name = "SceCommonDialog_stub")]
#[link(name = "SceCtrl_stub")]
#[link(name = "SceDisplay_stub")]
#[link(name = "SceGxm_stub")]
#[link(name = "SceHid_stub")]
#[link(name = "SceMotion_stub")]
#[link(name = "SceTouch_stub")]
extern "C" {}

fn main() {
    let rom_path = std::env::args().nth(1).expect("No ROM path given");  

    let sdl_context = sdl2::init().expect("Couldn't init sdl");
    let video_subsystem = sdl_context.video().expect("Couldn't init sdl video");
    let audio_subsystem = sdl_context.audio().expect("Couldn't init sdl audio");

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
        while cycles_this_frame <= gameboy::CYCLES_PER_FRAME {
            cycles_this_frame += gameboy.update() as f64;
        }

        texture.update(None, &gameboy.screen_data, gameboy::WIDTH.wrapping_mul(3) as usize).expect("Couldn't update texture from main");
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