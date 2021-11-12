use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::gameboy::{Gameboy, TIMA, TAC};

pub struct Memory {
    rom_banking: bool,
    enable_ram: bool,
    mbc1: bool,
    mbc2: bool,
    current_rom_bank : u8,
    current_ram_bank: u8,
    ram_banks: [u8; 0x8000],
    pub cartridge: Vec<u8>,
    pub rom: Vec<u8>,
}

impl Memory {
    pub fn new() -> Memory {
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
            rom_banking: false,
            enable_ram: false,
            mbc1: false,
            mbc2: false,
            current_rom_bank : 1,
            current_ram_bank: 0,
            ram_banks: [0; 0x8000],
            cartridge: vec![0; 0x200000],
            rom: rom_vec,
        }
    }

    pub fn load_cartridge(&mut self, rom_path: PathBuf) {
        let mut file = File::open(rom_path).expect("Invalid ROM path");
        file.read_to_end(&mut self.cartridge).expect("Unable to read ROM");
        for i in 0..0x8000 {
            self.rom[i] = self.cartridge[i];
        }
    }

    pub fn read_from_memory(&self, gameboy: Option<&mut Gameboy>, address: u16) -> u8 {
        if (address >= 0x4000) && (address <= 0x7fff) {
            let new_address: u16 = address - 0x4000;
            self.cartridge[(new_address + (self.current_rom_bank as u16) * 0x4000) as usize]
        } else if((address >= 0xa000) && (address <= 0xbfff)) {
            let new_address: u16 = address - 0xa000;
            self.ram_banks[(new_address + (self.current_ram_bank as u16) * 0x2000) as usize]
        } else if((address >= 0xfea0) && (address < 0xff00)) {
            // TODO OAM Corruption Bug
            0xff
        } /* else if(address == 0xff00) {
            return getGamepadState(gameBoy);
        } */ else {
            self.rom[address as usize]
        }
    }

    pub fn write_to_memory(&mut self, gameboy: Option<&mut Gameboy>, address: u16, value: u8) {
        if address < 0x8000 {
            // self.handle_banking(address, value);
        } else if (address >= 0xa000) && (address < 0xc000) {
            if self.enable_ram {
                let new_address: u16 = address - 0xa000;
                self.ram_banks[(new_address + (self.current_ram_bank as u16) * 0x2000) as usize] = value;
            }
        } else if (address >= 0xfea0) && (address < 0xff00) {
            // RESTRICTED
        } else if (address >= 0xc000) && (address < 0xe000) {
            self.rom[address as usize] = value;
            if address + 0x2000 <= 0xfdff {
                self.rom[(address + 0x2000) as usize] = value;
            }
        } else if (address >= 0xe000) && (address < 0xfe00) {
            // RESTRICTED
            self.rom[address as usize] = value;
            self.rom[(address - 0x2000) as usize] = value;
        } else if address == TAC {
            match gameboy {
                Some(t) => {
                    let current_freq: u8 = 0 /*t.get_clock_freq()*/;
                    self.rom[address as usize] = value;
                    let new_freq: u8 = 0 /*t.get_clock_freq()*/;
                    if current_freq != new_freq {
                        //t.set_clock_freq();
                    }
                },
                None => {},
            }
        } else if (address == 0xff04) || (address == 0xff44) {
            if address == 0xff04 {
                match gameboy {
                    Some(t) => {
                        t.divider_counter = 0;
                    },
                    None => {},
                }
            }
            self.rom[address as usize] = 0;
        } else if address == 0xff46 {
            // self.do_dma_transfer(value);
        } else {
            self.rom[address as usize] = value;
        }
    }
}