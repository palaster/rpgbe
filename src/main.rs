#[macro_use]
extern crate lazy_static;

use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};

mod cpu;
mod memory;
mod bit_logic;

use cpu::CPU;
use memory::Memory;

const WIDTH: u16 = 160;
const HEIGHT: u16 = 144;

const CYCLES_PER_SECOND: u64 = 4_194_304;
const FRAMES_PER_SECOND: f64 = 59.727500569606;
const CYCLES_PER_FRAME: f64 = (CYCLES_PER_SECOND as f64) / FRAMES_PER_SECOND;
const TIME_BETWEEN_FRAMES_IN_NANOSECONDS: f64 = ((1000 as f64) / FRAMES_PER_SECOND) * 1_000_000.0;

lazy_static! {
    pub static ref MEMORY: Arc<Mutex<Memory>> = Arc::new(Mutex::new(Memory::new()));
}

fn main() {
    let mut cpu: CPU = CPU::new();

    let mut should_close: bool = false;

    while !should_close {
        let start = Instant::now();
        let mut cycles_this_frame = 0.0;
        while cycles_this_frame <= CYCLES_PER_FRAME {
            let mut cycles = 4.0;
            if !cpu.is_halted() { cycles = cpu.update(); }
            cycles_this_frame += cycles;
            //updateTimer(cycles);
            //updateGraphics(cycles);
            //cyclesThisFrame += doInterrupts();
        }

        let elapsed_time = start.elapsed();
        let time_between_frames = Duration::from_nanos(TIME_BETWEEN_FRAMES_IN_NANOSECONDS as u64);
        if elapsed_time <= time_between_frames {
            let time_remaining = time_between_frames - elapsed_time;
            thread::sleep(time_remaining);
        }
    }
}
