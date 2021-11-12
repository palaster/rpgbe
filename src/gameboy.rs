use crate::{WIDTH, HEIGHT, SCREEN_DATA_SIZE};
use crate::cpu::CPU;
use crate::memory::Memory;
use crate::bit_logic;

use std::sync::{Arc, Mutex};

const TIMA: u16 = 0xff05;
const TMA: u16 = 0xff06;
const TAC: u16 = 0xff07;

const VERTICAL_BLANK_SCAN_LINE: u16 = 144;
const VERTICAL_BLANK_SCAN_LINE_MAX: u16 = 153;
const SCANLINE_COUNTER_START: u16 = 456;

pub struct Gameboy {
    scanline_counter: i32,
    timer_counter: i32,
    divider_counter: i32,
    halt_bug: bool,
    ei_halt_bug: bool,
    gamepad_state: u8,
    pub screen_data: [u8; SCREEN_DATA_SIZE as usize],
    scanline_bg: [bool; WIDTH as usize],
    cpu: CPU,
    memory: Arc<Mutex<Memory>>,
}

impl Gameboy {
    pub fn new(memory: Arc<Mutex<Memory>>) -> Gameboy {
        Gameboy {
            scanline_counter: SCANLINE_COUNTER_START.into(),
            timer_counter: 0,
            divider_counter: 0,
            halt_bug: false,
            ei_halt_bug: false,
            gamepad_state: 0xff,
            screen_data: [0; SCREEN_DATA_SIZE as usize],
            scanline_bg: [false; WIDTH as usize],
            cpu: CPU::new(memory.clone()),
            memory: memory,
        }
    }

    pub fn update(&mut self) -> f64 {
        let mut cycles: f64 = 4.0;
        if !self.cpu.is_halted() { cycles = self.cpu.update(); }
        self.update_timer(cycles);
        self.update_graphics(cycles);
        cycles += self.do_interrupts();
        cycles
    }


    fn update_timer(&mut self, cycles: f64) {

    }

    fn update_graphics(&mut self, cycles: f64) {
        
    }

    fn do_interrupts(&mut self) -> f64 {
        0.0
    }
}