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
use cpu::Cpu;
use memory::Memory;

const WIDTH: u16 = 160;
const HEIGHT: u16 = 144;
const SCREEN_DATA_SIZE: u32 = (WIDTH as u32) * (HEIGHT as u32) * 3;

const CYCLES_PER_SECOND: u64 = 4_194_304;
const FRAMES_PER_SECOND: f64 = 59.727500569606;
const CYCLES_PER_FRAME: f64 = (CYCLES_PER_SECOND as f64) / FRAMES_PER_SECOND;
const TIME_BETWEEN_FRAMES_IN_NANOSECONDS: f64 = (1_000.0 / FRAMES_PER_SECOND) * 1_000_000.0;

lazy_static! {
    pub static ref MEMORY: Arc<Mutex<Memory>> = Arc::new(Mutex::new(Memory::new()));
    pub static ref GAMEBOY: Arc<Mutex<Gameboy>> = Arc::new(Mutex::new(Gameboy::new()));
    pub static ref CPU: Arc<Mutex<Cpu>> = Arc::new(Mutex::new(Cpu::new()));
}

fn main() {
    let rom_path = std::env::args().nth(1).expect("No ROM path given");  

    let sdl_context = sdl2::init().expect("Couldn't init sdl");
    let video_subsystem = sdl_context.video().expect("Couldn't init sdl video");

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

    let mut event_pump = sdl_context.event_pump().expect("Couldn't get event_pump from sdl_context");

    {
        MEMORY.lock().expect("Couldn't get memory from main").load_cartridge(PathBuf::from(rom_path));
    }

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
            cycles_this_frame += { let mut gameboy = GAMEBOY.lock().expect("Couldn't get gameboy from main loop"); gameboy.update() };
        }

        /*
        let mut temp_screen_data: [u8; SCREEN_DATA_SIZE as usize] = [0; SCREEN_DATA_SIZE as usize];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                /*
                let final_x: u16 = x.wrapping_mul(WIDTH).wrapping_mul(3);
                let final_y: u16 = y.wrapping_mul(3);
                let xy: u16 = final_x.wrapping_add(final_y);
                */
                let final_x: u32 = x as u32 * 3;
                let final_y: u32 = (WIDTH as u32) * y as u32 * 3;
                let xy = final_x.wrapping_add(final_y);
                temp_screen_data[(xy) as usize] = 0x0;
                temp_screen_data[(xy + 1) as usize] = 0xff;
                temp_screen_data[(xy + 2) as usize] = 0xff;
            }
        }
        */

        texture.update(None, &{ let gameboy = GAMEBOY.lock().expect("Couldn't get gameboy from texture.update"); gameboy.screen_data }, WIDTH.wrapping_mul(3) as usize).expect("Couldn't update texture from main");
        /*
        texture.update(None, &temp_screen_data, WIDTH as usize * 3).expect("Couldn't update texture from main");
        */
        canvas.clear();
        canvas.copy(&texture, None, None).expect("Couldn't copy canvas");
        canvas.present();

        let elapsed_time = start.elapsed();
        let time_between_frames = Duration::from_nanos(TIME_BETWEEN_FRAMES_IN_NANOSECONDS as u64);
        if elapsed_time <= time_between_frames {
            let time_remaining = time_between_frames - elapsed_time;
            thread::sleep(time_remaining);
        }
    }
}
