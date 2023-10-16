pub const WIDTH: u16 = 160;
pub const HEIGHT: u16 = 144;
pub const SCREEN_DATA_SIZE: u32 = (WIDTH as u32) * (HEIGHT as u32) * 3;

pub const CYCLES_PER_SECOND: u32 = 4_194_304;
pub const FRAMES_PER_SECOND: f64 = 59.727500569606;
pub const CYCLES_PER_FRAME: f64 = (CYCLES_PER_SECOND as f64) / FRAMES_PER_SECOND;
pub const TIME_BETWEEN_FRAMES_IN_NANOSECONDS: f64 = (1_000.0 / FRAMES_PER_SECOND) * 1_000_000.0;
pub const DURATION_BETWEEN_FRAMES: Duration = Duration::from_nanos(TIME_BETWEEN_FRAMES_IN_NANOSECONDS as u64);

pub const SAMPLE_RATE: u16 = 44_100;
pub const TIME_BETWEEN_AUDIO_SAMPLING: u8 = (CYCLES_PER_SECOND / SAMPLE_RATE as u32) as u8;

mod bit_logic;
mod cpu;
mod gpu;
mod memory;
mod spu;
mod timer;

use std::time::Duration;

use cpu::Cpu;
use gpu::Gpu;
use memory::Memory;
use spu::{ Spu, SoundChannel };
use timer::Timer;

const IS_DEBUG_MODE: bool = false;

const TAC: u16 = 0xff07;

pub(crate) enum MemoryWriteResult {
    None,
    ResetDividerCounter,
    SetTimerCounter,
    ResetChannel(u8, u8),
}

pub(crate) struct Gameboy {
    target_pc: i32,
    cpu: Cpu,
    pub(crate) gpu: Gpu,
    pub(crate) memory: Memory,
    pub(crate) spu: Spu,
    pub(crate) timer: Timer,
}

impl Gameboy {
    pub(crate) fn new() -> Gameboy {
        Gameboy {
            target_pc: -1,
            cpu: Cpu::new(),
            gpu: Gpu::new(),
            memory: Memory::new(),
            spu: Spu::new(),
            timer: Timer::new(),
        }
    }

    pub(crate) fn key_pressed(&mut self, key: u8) {
        let previously_unset: bool = !bit_logic::check_bit(self.memory.gamepad_state, key);

        self.memory.gamepad_state = bit_logic::reset_bit(self.memory.gamepad_state, key);

        let button: bool = key > 3;

        let key_req: u8 = self.raw_read_from_rom(0xff00);
        let should_request_interrupt: bool = (button && !bit_logic::check_bit(key_req, 5)) || (!button && !bit_logic::check_bit(key_req, 4));

        if should_request_interrupt && !previously_unset {
            self.memory.request_interrupt(4);
        }
    }

    pub(crate) fn key_released(&mut self, key: u8) {
        self.memory.gamepad_state = bit_logic::set_bit(self.memory.gamepad_state, key);
    }

    fn read_from_address(&self, address: u16) -> u8 {
        self.memory.read_from_memory(address)
    }

    fn write_to_address(&mut self, address: u16, value: u8) {
        for memory_result in self.memory.write_to_memory(address, value) {
            match memory_result {
                MemoryWriteResult::ResetDividerCounter => {
                    self.timer.divider_counter = 0
                },
                MemoryWriteResult::SetTimerCounter => {
                    self.timer.set_clock_freq(&self.memory)
                },
                MemoryWriteResult::ResetChannel(id, length) => {
                    match id {
                        0 => { self.spu.sound_channel_1.reset(&self.memory, length) },
                        1 => { self.spu.sound_channel_2.reset(&self.memory, length) },
                        2 => { self.spu.sound_channel_3.reset(&self.memory, length) },
                        3 => { self.spu.sound_channel_4.reset(&self.memory, length) },
                        _ => { },
                    }
                },
                _ => { },
            }
        }
    }

    fn raw_read_from_rom(&self, address: u16) -> u8 {
        self.memory.rom[address as usize]
    }

    fn raw_write_to_rom(&mut self, address: u16, value: u8) {
        self.memory.rom[address as usize] = value;
    }

    pub(crate) fn update(&mut self) -> u8 {
        let mut cycles: u8 = 4;
        if !self.cpu.halted {
            let (new_cycles, memory_write_results) = self.cpu.update(&mut self.memory);
            cycles = new_cycles.wrapping_mul(4);
            for memory_result in memory_write_results {
                match memory_result {
                    MemoryWriteResult::ResetDividerCounter => {
                        self.timer.divider_counter = 0
                    },
                    MemoryWriteResult::SetTimerCounter => {
                        self.timer.set_clock_freq(&self.memory)
                    },
                    MemoryWriteResult::ResetChannel(id, length) => {
                        match id {
                            0 => { self.spu.sound_channel_1.reset(&self.memory, length) },
                            1 => { self.spu.sound_channel_2.reset(&self.memory, length) },
                            2 => { self.spu.sound_channel_3.reset(&self.memory, length) },
                            3 => { self.spu.sound_channel_4.reset(&self.memory, length) },
                            _ => { },
                        }
                    },
                    _ => { },
                }
            }
            if IS_DEBUG_MODE {
                println!("{}", self.cpu.debug());
            }
        }
        if IS_DEBUG_MODE {
            if self.raw_read_from_rom(0xff02) == 0x81 {
                let c: char = self.raw_read_from_rom(0xff01) as char;
                print!("{}", c);
                self.raw_write_to_rom(0xff02, 0x0);
            }
            if self.target_pc == -1 {
                let mut line = String::new();
                println!("Enter new PC to run to:");
                std::io::stdin().read_line(&mut line).unwrap();
                if !line.is_empty() {
                    self.target_pc = line.trim().parse().unwrap_or(-1);
                }
            } else if self.target_pc == (self.cpu.pc as i32) {
                self.target_pc = -1;
            }
        }

        for memory_result in self.timer.update_timer(&mut self.memory, cycles) {
            match memory_result {
                MemoryWriteResult::ResetDividerCounter => {
                    self.timer.divider_counter = 0
                },
                MemoryWriteResult::SetTimerCounter => {
                    self.timer.set_clock_freq(&self.memory)
                },
                MemoryWriteResult::ResetChannel(id, length) => {
                    match id {
                        0 => { self.spu.sound_channel_1.reset(&self.memory, length) },
                        1 => { self.spu.sound_channel_2.reset(&self.memory, length) },
                        2 => { self.spu.sound_channel_3.reset(&self.memory, length) },
                        3 => { self.spu.sound_channel_4.reset(&self.memory, length) },
                        _ => { },
                    }
                },
                _ => { },
            }
        }
        self.gpu.update_graphics(&mut self.memory, cycles);
        //self.spu.update_audio(&mut self.memory, cycles);

        cycles += self.do_interrupts();
        cycles
    }

    fn service_interrupt(&mut self, interrupt_id: u8) {
        self.cpu.interrupts_enabled = false;
        self.write_to_address(0xff0f, bit_logic::reset_bit(self.read_from_address(0xff0f), interrupt_id));
    
        let pc: u16 = self.cpu.pc;
        self.cpu.push(&mut self.memory, (pc >> 8) as u8);
        self.cpu.push(&mut self.memory, pc as u8);

        match interrupt_id {
            0 => { self.cpu.pc = 0x40 },
            1 => { self.cpu.pc = 0x48 },
            2 => { self.cpu.pc = 0x50 },
            3 => { self.cpu.pc = 0x58 },
            4 => { self.cpu.pc = 0x60 },
            _ => {},
        }
    }

    fn do_interrupts(&mut self) -> u8 {
        let (req, enabled): (u8, u8) = (self.read_from_address(0xff0f), self.read_from_address(0xffff));
        let potential_for_interrupts: u8 = req & enabled;
        if potential_for_interrupts == 0 {
            return 0;
        }
        if self.cpu.interrupts_enabled {
            self.cpu.halted = false;
            for i in 0..5 {
                if bit_logic::check_bit(req, i) && bit_logic::check_bit(enabled, i) {
                    self.service_interrupt(i);
                    return 20;
                }
            }
        }
        self.cpu.halted = false;
        0
    }
}