use crate::{bit_logic};

use alloc::vec::Vec;

mod spu;
mod graphic;
mod cpu;
mod memory;

use spu::{ Spu, SoundChannel };
use cpu::Cpu;
use memory::Memory;

pub(crate) const WIDTH: u16 = 160;
pub(crate) const HEIGHT: u16 = 144;
const SCREEN_DATA_SIZE: u32 = (WIDTH as u32) * (HEIGHT as u32) * 3;

const CYCLES_PER_SECOND: u32 = 4_194_304;
const FRAMES_PER_SECOND: f64 = 59.727500569606;
const CYCLES_PER_FRAME: f64 = (CYCLES_PER_SECOND as f64) / FRAMES_PER_SECOND;

const SAMPLE_RATE: u16 = 44_100;
const TIME_BETWEEN_AUDIO_SAMPLING: u8 = (CYCLES_PER_SECOND / SAMPLE_RATE as u32) as u8;

const TIMA: u16 = 0xff05;
const TMA: u16 = 0xff06;
const TAC: u16 = 0xff07;

const FREQUENCY_4096: u16 = 1024; // CYCLES_PER_SECOND / 4096
const FREQUENCY_262144: u16 = 16; // CYCLES_PER_SECOND / 262144
const FREQUENCY_65536: u16 = 64; // CYCLES_PER_SECOND / 65536
const FREQUENCY_16384: u16 = 256; // CYCLES_PER_SECOND / 16384

const VERTICAL_BLANK_SCAN_LINE: u8 = 144;
const VERTICAL_BLANK_SCAN_LINE_MAX: u8 = 153;
const SCANLINE_COUNTER_START: u16 = 456;

pub(crate) enum MemoryWriteResult {
    None,
    ResetDividerCounter,
    SetTimerCounter,
    ResetChannel(u8, u8),
}

pub(crate) struct Gameboy {
    target_pc: i32,
    scanline_counter: i32,
    pub(crate) timer_counter: i32,
    pub(crate) divider_counter: i32,
    pub(crate) screen_data: [u8; SCREEN_DATA_SIZE as usize],
    scanline_bg: [bool; WIDTH as usize],
    pub(crate) spu: Spu,
    cpu: Cpu,
    pub(crate) memory: Memory,
}

impl Gameboy {
    pub(crate) fn new() -> Gameboy {
        Gameboy {
            target_pc: -1,
            scanline_counter: SCANLINE_COUNTER_START as i32,
            timer_counter: 0,
            divider_counter: 0,
            screen_data: [0; SCREEN_DATA_SIZE as usize],
            scanline_bg: [false; WIDTH as usize],
            spu: Spu::new(),
            cpu: Cpu::new(),
            memory: Memory::new(),
        }
    }

    pub(crate) fn key_pressed(&mut self, key: u8) {
        let previously_unset: bool = !bit_logic::check_bit(self.memory.gamepad_state, key);

        self.memory.gamepad_state = bit_logic::reset_bit(self.memory.gamepad_state, key);

        let button: bool = key > 3;

        let key_req: u8 = self.raw_read_from_rom(0xff00);
        let mut should_request_interrupt: bool = false;

        if button && !bit_logic::check_bit(key_req, 5) {
            should_request_interrupt = true;
        } else if !button && !bit_logic::check_bit(key_req, 4) {
            should_request_interrupt = true;
        }

        if should_request_interrupt && !previously_unset {
            self.request_interrupt(4);
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
                    self.divider_counter = 0
                },
                MemoryWriteResult::SetTimerCounter => {
                    self.set_clock_freq()
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

    pub(crate) fn next_frame(&mut self) {
        let mut cycles_this_frame: f64 = 0.0;
        while cycles_this_frame <= CYCLES_PER_FRAME {
            cycles_this_frame += self.update() as f64;
        }
    }

    fn update(&mut self) -> u8 {
        let mut cycles: u8 = 4;
        if !self.cpu.halted {
            let (new_cycles, memory_write_results) = self.cpu.update(&mut self.memory);
            cycles = new_cycles.wrapping_mul(4);
            for memory_result in memory_write_results {
                match memory_result {
                    MemoryWriteResult::ResetDividerCounter => {
                        self.divider_counter = 0
                    },
                    MemoryWriteResult::SetTimerCounter => {
                        self.set_clock_freq()
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
        self.update_timer(cycles);
        self.update_graphics(cycles);
        self.update_audio(cycles);
        cycles += self.do_interrupts();
        cycles
    }

    fn is_clock_enabled(&self) -> bool {
        bit_logic::check_bit(self.read_from_address(TAC), 2)
    }

    fn get_clock_freq(&self) -> u8 {
        self.read_from_address(TAC) & 0x3
    }

    fn set_clock_freq(&mut self) {
        match self.get_clock_freq() {
            0 => { self.timer_counter = FREQUENCY_4096 as i32 },
            1 => { self.timer_counter = FREQUENCY_262144 as i32 },
            2 => { self.timer_counter = FREQUENCY_65536 as i32 },
            3 => { self.timer_counter = FREQUENCY_16384 as i32 },
            _ => { },
        }
    }

    fn do_divider_register(&mut self, cycles: u8) {
        self.divider_counter += cycles as i32;
        if self.divider_counter >= 255 {
            self.divider_counter = 0;
            self.raw_write_to_rom(0xff04, self.raw_read_from_rom(0xff04).wrapping_add(1));
        }
    }

    fn update_timer(&mut self, cycles: u8) {
        self.do_divider_register(cycles);
        if self.is_clock_enabled() {
            self.timer_counter -= cycles as i32;
            if self.timer_counter <= 0 {
                self.set_clock_freq();
                let (tima, tma): (u8, u8) = (self.read_from_address(TIMA), self.read_from_address(TMA));
                if tima == 255 {
                    self.write_to_address(TIMA, tma);
                    self.request_interrupt(2);
                } else {
                    self.write_to_address(TIMA, tima.wrapping_add(1))
                }
            }
        }
    }

    fn is_lcd_enabled(&self) -> bool {
        bit_logic::check_bit(self.read_from_address(0xff40), 7)
    }

    fn set_lcd_status(&mut self) {
        let mut status: u8 = self.read_from_address(0xff41);
        if !self.is_lcd_enabled() {
            self.scanline_counter = SCANLINE_COUNTER_START as i32;
            self.raw_write_to_rom(0xff44, 0);
            status &= 252;
            status = bit_logic::set_bit(status, 0);
            self.write_to_address(0xff41, status);
            return;
        }
        let current_line: u8 = self.read_from_address(0xff44);
        let current_mode: u8 = status & 0x3;
        let mode: u8;
        let mut req_int: bool = false;

        if current_line >= 144 {
            mode = 1;
            status = bit_logic::set_bit(status, 0);
            status = bit_logic::reset_bit(status, 1);
            req_int = bit_logic::check_bit(status, 4);
        } else {
            let mode_2_bounds: i32 = 376; // 456 - 80
            let mode_3_bounds: i32 = 204; // mode_2_bounds - 172
            if self.scanline_counter >= mode_2_bounds {
                mode = 2;
                status = bit_logic::set_bit(status, 1);
                status = bit_logic::reset_bit(status, 0);
                req_int = bit_logic::check_bit(status, 5);
            } else if self.scanline_counter >= mode_3_bounds {
                mode = 3;
                status = bit_logic::set_bit(status, 1);
                status = bit_logic::set_bit(status, 0);
            } else {
                mode = 0;
                status = bit_logic::reset_bit(status, 1);
                status = bit_logic::reset_bit(status, 0);
                req_int = bit_logic::check_bit(status, 3);
            }
        }
        if req_int && current_mode != mode {
            self.request_interrupt(1);
        }
        if current_line == self.read_from_address(0xff45) {
            status = bit_logic::set_bit(status, 2);
            if bit_logic::check_bit(status, 6) {
                self.request_interrupt(1);
            }
        } else {
            status = bit_logic::reset_bit(status, 2);
        }
        self.write_to_address(0xff41, status);
    }

    fn update_graphics(&mut self, cycles: u8) {
        self.set_lcd_status();
        if self.is_lcd_enabled() {
            self.scanline_counter -= cycles as i32;
        } else {
            return;
        }
        if self.scanline_counter <= 0 {
            let current_line: u8 = {
                self.raw_write_to_rom(0xff44, self.raw_read_from_rom(0xff44).wrapping_add(1));
                self.read_from_address(0xff44)
            };
            self.scanline_counter = SCANLINE_COUNTER_START as i32;
            if current_line == VERTICAL_BLANK_SCAN_LINE {
                self.request_interrupt(0);
            } else if current_line > VERTICAL_BLANK_SCAN_LINE_MAX {
                self.raw_write_to_rom(0xff44, 0);
            } else if current_line < VERTICAL_BLANK_SCAN_LINE {
                self.draw_scanline();
            }
        }
    }

    fn update_audio(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.spu.sound_channel_1.update(&mut self.memory);
            self.spu.sound_channel_2.update(&mut self.memory);
            self.spu.sound_channel_3.update(&mut self.memory);
            self.spu.sound_channel_4.update(&mut self.memory);

            if self.spu.audio_fill_timer == 0 {
                self.spu.audio_fill_timer = TIME_BETWEEN_AUDIO_SAMPLING;
                let (_enable_left_vin, left_volume, _enable_right_vin, right_volume) = {
                    let nr50 = self.read_from_address(0xff24);
                    (
                        nr50 & 0x80 != 0,
                        (nr50 & 0x70) >> 4,
                        nr50 & 0x8 != 0,
                        nr50 & 0x7
                    )
                };
                let channel_1 = self.spu.sound_channel_1.get_amplitude(&self.memory);
                let channel_2 = self.spu.sound_channel_2.get_amplitude(&self.memory);
                let channel_3 = self.spu.sound_channel_3.get_amplitude(&self.memory);
                let channel_4 = self.spu.sound_channel_4.get_amplitude(&self.memory);
                let nr51 = self.read_from_address(0xff25);
                if nr51 != 0 {
                    let mut left_results = 0.0;
                    left_results += if bit_logic::check_bit(nr51, 4) { channel_1 * (left_volume as f32 / 7.0) } else { 0.0 };
                    left_results += if bit_logic::check_bit(nr51, 5) { channel_2 * (left_volume as f32 / 7.0) } else { 0.0 };
                    left_results += if bit_logic::check_bit(nr51, 6) { channel_3 * (left_volume as f32 / 7.0) } else { 0.0 };
                    left_results += if bit_logic::check_bit(nr51, 7) { channel_4 * (left_volume as f32 / 7.0) } else { 0.0 };
                    self.spu.audio_data.push(left_results);
                    let mut right_results = 0.0;
                    right_results += if bit_logic::check_bit(nr51, 0) { channel_1 * (right_volume as f32 / 7.0) } else { 0.0 };
                    right_results += if bit_logic::check_bit(nr51, 1) { channel_2 * (right_volume as f32 / 7.0) } else { 0.0 };
                    right_results += if bit_logic::check_bit(nr51, 2) { channel_3 * (right_volume as f32 / 7.0) } else { 0.0 };
                    right_results += if bit_logic::check_bit(nr51, 3) { channel_4 * (right_volume as f32 / 7.0) } else { 0.0 };
                    self.spu.audio_data.push(right_results);
                } else {
                    self.spu.audio_data.push(0.0);
                    self.spu.audio_data.push(0.0);
                }
            } else {
                self.spu.audio_fill_timer = self.spu.audio_fill_timer.saturating_sub(1);
            }
        }
    }

    fn request_interrupt(&mut self, interrupt_id: u8) {
        self.write_to_address(0xff0f, bit_logic::set_bit(self.read_from_address(0xff0f), interrupt_id));
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