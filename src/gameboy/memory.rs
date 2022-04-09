use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::bit_logic;
use super::{ MemoryWriteResult, TAC};

pub(crate) struct Memory {
    pub(crate) gamepad_state: u8,
    rom_banking: bool,
    enable_ram: bool,
    mbc1: bool,
    mbc2: bool,
    current_rom_bank: u8,
    current_ram_bank: u8,
    ram_banks: Vec<u8>,
    cartridge: Vec<u8>,
    pub(crate) rom: Vec<u8>,
}

impl Memory {
    pub(crate) fn new() -> Memory {
        let mut rom_vec = vec![0; 0x10000];

        rom_vec[0xff05] = 0x00;
        rom_vec[0xff06] = 0x00;
        rom_vec[0xff07] = 0x00;
        rom_vec[0xff10] = 0x80;
        rom_vec[0xff11] = 0xbf;
        rom_vec[0xff12] = 0xf3;
        rom_vec[0xff14] = 0xbf;
        rom_vec[0xff16] = 0x3f;
        rom_vec[0xff17] = 0x00;
        rom_vec[0xff19] = 0xbf;
        rom_vec[0xff1a] = 0x7f;
        rom_vec[0xff1b] = 0xff;
        rom_vec[0xff1c] = 0x9f;
        rom_vec[0xff1e] = 0xbf;
        rom_vec[0xff20] = 0xff;
        rom_vec[0xff21] = 0x00;
        rom_vec[0xff22] = 0x00;
        rom_vec[0xff23] = 0xbf;
        rom_vec[0xff24] = 0x77;
        rom_vec[0xff25] = 0xf3;
        rom_vec[0xff26] = 0xf1;
        rom_vec[0xff40] = 0x91;
        rom_vec[0xff42] = 0x00;
        rom_vec[0xff43] = 0x00;
        rom_vec[0xff45] = 0x00;
        rom_vec[0xff47] = 0xfc;
        rom_vec[0xff48] = 0xff;
        rom_vec[0xff49] = 0xff;
        rom_vec[0xff4a] = 0x00;
        rom_vec[0xff4b] = 0x00;
        rom_vec[0xffff] = 0x00;

        Memory {
            gamepad_state: 0xff,
            rom_banking: false,
            enable_ram: false,
            mbc1: false,
            mbc2: false,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_banks: vec![0; 0x8000],
            cartridge: Vec::new(),
            rom: rom_vec,
        }
    }

    pub(crate) fn load_cartridge(&mut self, rom_path: PathBuf) {
        let mut file = File::open(rom_path).expect("Invalid ROM path");
        file.read_to_end(&mut self.cartridge).expect("Unable to read ROM");
        for i in 0..0x8000 {
            self.rom[i] = self.cartridge[i];
        }
        match self.cartridge[0x147] {
            1 | 2 | 3 => { self.mbc1 = true },
            5 | 6 => { self.mbc2 = true },
            _ => {},
        }
    }

    fn get_gamepad_state(&self) -> u8 {
        let mut res: u8 = self.rom[0xff00 as usize] ^ 0xff;
        if !bit_logic::check_bit(res, 4) {
            let top_gamepad: u8 = (self.gamepad_state >> 4) | 0xf0;
            res &= top_gamepad;
        } else if !bit_logic::check_bit(res, 5) {
            let bottom_gamepad: u8 = (self.gamepad_state & 0xf) | 0xf0;
            res &= bottom_gamepad;
        }
        res
    }

    pub(crate) fn read_from_memory(&self, address: u16) -> u8 {
        match address {
            0x4000..=0x7fff => {
                let new_address: u16 = address - 0x4000;
                self.cartridge[(new_address.wrapping_add((self.current_rom_bank as u16).wrapping_mul(0x4000))) as usize]
            },
            0xa000..=0xbfff => {
                let new_address: u16 = address - 0xa000;
                self.ram_banks[(new_address.wrapping_add((self.current_ram_bank as u16).wrapping_mul(0x2000))) as usize]
            },
            0xfea0..=0xfeff => 0xff,
            0xff00 => self.get_gamepad_state(),
            _ => self.rom[address as usize],
        }
    }

    fn do_dma_transfer(&mut self, value: u8) -> Vec<MemoryWriteResult> {
        let address: u16 = (value as u16) << 8;
        let mut memory_results: Vec<MemoryWriteResult> = Vec::new();
        for i in 0..0xa0 {
            memory_results.append(&mut self.write_to_memory(0xfe00 + i, self.read_from_memory(address + i)));
        }
        memory_results
    }

    fn do_ram_bank_enable(&mut self, address: u16, value: u8) {
        if self.mbc2 && bit_logic::bit_value(address as u8, 4) == 1 {
            return;
        }
        let test_data: u8 = value & 0xf;
        if test_data == 0xa {
            self.enable_ram = true;
        } else if test_data == 0x0 {
            self.enable_ram = false;
        }
    }
    
    fn do_change_lo_rom_bank(&mut self, value: u8) {
        if self.mbc2 {
            self.current_rom_bank = value & 0xf;
            if self.current_rom_bank == 0 {
                self.current_rom_bank += 1;
            }
            return;
        }
        let lower5: u8 = value & 31;
        self.current_rom_bank &= 224;
        self.current_rom_bank |= lower5;
        if self.current_rom_bank == 0 {
            self.current_rom_bank += 1;
        }
    }
    
    fn do_change_hi_rom_bank(&mut self, value: u8) {
        let new_value: u8 = value & 224;
        self.current_rom_bank &= 31;
        self.current_rom_bank |= new_value;
        if self.current_rom_bank == 0 {
            self.current_rom_bank += 1;
        }
    }
    
    fn do_ram_bank_change(&mut self, value: u8) {
        self.current_ram_bank = value & 0x3;
    }
    
    fn do_change_rom_ram_mode(&mut self, value: u8) {
        let new_value: u8 = value & 0x1;
        self.rom_banking = new_value == 0;
        if self.rom_banking {
            self.current_ram_bank = 0;
        }
    }

    fn handle_banking(&mut self, address: u16, value: u8) {
        match address {
            0..=0x1fff => {
                if self.mbc1 || self.mbc2 {
                    self.do_ram_bank_enable(address, value);
                }
            },
            0x2000..=0x3fff => {
                if self.mbc1 || self.mbc2 {
                    self.do_change_lo_rom_bank(value);
                }
            },
            0x4000..=0x5fff => {
                if self.mbc1 {
                    if self.rom_banking {
                        self.do_change_hi_rom_bank(value);
                    } else {
                        self.do_ram_bank_change(value);
                    }
                }
            },
            0x6000..=0x7fff => {
                if self.mbc1 {
                    self.do_change_rom_ram_mode(value);
                }
            },
            _ => {},
        }
    }

    pub(crate) fn write_to_memory(&mut self, address: u16, value: u8) -> Vec<MemoryWriteResult> {
        let memory_write_results: Vec<MemoryWriteResult> = vec![MemoryWriteResult::None];
        match address {
            0..=0x7fff => { self.handle_banking(address, value) },
            0xa000..=0xbfff => {
                if self.enable_ram {
                    let new_address: u16 = address - 0xa000;
                    self.ram_banks[(new_address + (self.current_ram_bank as u16) * 0x2000) as usize] = value
                }
            },
            0xc000..=0xdfff => {
                self.rom[address as usize] = value;
                if address + 0x2000 <= 0xfdff {
                    self.rom[(address + 0x2000) as usize] = value
                }
            },
            0xe000..=0xfdff => {
                self.rom[address as usize] = value;
                self.rom[(address - 0x2000) as usize] = value
            },
            0xfea0..=0xfeff => {},
            TAC => {
                let current_freq: u8 = self.read_from_memory(TAC) & 0x3;
                self.rom[address as usize] = value;
                let new_freq: u8 = self.read_from_memory(TAC) & 0x3;
                if current_freq != new_freq { return vec![MemoryWriteResult::SetTimerCounter]; }
            },
            0xff04 | 0xff44 => {
                self.rom[address as usize] = 0;
                if address == 0xff04 { return vec![MemoryWriteResult::ResetDividerCounter]; }
            },
            0xff46 => {
                return self.do_dma_transfer(value);
            },
            0xff14 => {
                self.rom[address as usize] = value;
                if value >> 7 == 1 { return vec![MemoryWriteResult::ResetChannel(0, self.read_from_memory(0xff11) & 0x3f)]; }
            },
            0xff19 => {
                self.rom[address as usize] = value;
                if value >> 7 == 1 { return vec![MemoryWriteResult::ResetChannel(1, self.read_from_memory(0xff16) & 0x3f)]; }
            },
            0xff1e => {
                self.rom[address as usize] = value;
                if value >> 7 == 1 { return vec![MemoryWriteResult::ResetChannel(2, self.read_from_memory(0xff1b))]; }
            },
            0xff23 => {
                self.rom[address as usize] = value;
                if value >> 7 == 1 { return vec![MemoryWriteResult::ResetChannel(3, self.read_from_memory(0xff20) & 0x3f)]; }
            },
            _ => { self.rom[address as usize] = value },
        }
        memory_write_results
    }
}