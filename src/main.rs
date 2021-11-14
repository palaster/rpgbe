#[macro_use]
extern crate lazy_static;
extern crate sdl2;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;

mod gameboy;
mod cpu;
mod memory;
mod bit_logic;

use gameboy::Gameboy;
use memory::Memory;

const WIDTH: u16 = 160;
const HEIGHT: u16 = 144;
const SCREEN_DATA_SIZE: u32 = (WIDTH as u32) * (HEIGHT as u32) * 3;

const CYCLES_PER_SECOND: u64 = 4_194_304;
const FRAMES_PER_SECOND: f64 = 59.727500569606;
const CYCLES_PER_FRAME: f64 = (CYCLES_PER_SECOND as f64) / FRAMES_PER_SECOND;
const TIME_BETWEEN_FRAMES_IN_NANOSECONDS: f64 = (1000.0 / FRAMES_PER_SECOND) * 1_000_000.0;

lazy_static! {
    pub static ref MEMORY: Arc<Mutex<Memory>> = Arc::new(Mutex::new(Memory::new()));
}

fn main() {
    let rom_path = std::env::args().nth(1).expect("No ROM path given");  

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("RPGBE", WIDTH.into(), HEIGHT.into())
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .software()
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, WIDTH.into(), HEIGHT.into()).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    {
        MEMORY.lock().unwrap().load_cartridge(PathBuf::from(rom_path));
    }

    let mut gameboy: Gameboy = Gameboy::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                _ => (),
            }
        }

        let start = Instant::now();
        let mut cycles_this_frame: f64 = 0.0;
        while cycles_this_frame <= CYCLES_PER_FRAME {
            cycles_this_frame += gameboy.update();
        }

        texture.update(None, &gameboy.screen_data, (WIDTH * 3).into()).unwrap();
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        let elapsed_time = start.elapsed();
        let time_between_frames = Duration::from_nanos(TIME_BETWEEN_FRAMES_IN_NANOSECONDS as u64);
        if elapsed_time <= time_between_frames {
            let time_remaining = time_between_frames - elapsed_time;
            thread::sleep(time_remaining);
        }
    }
}
