use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub struct Memory {
    cartridge: Vec<u8>,
    rom: Vec<u8>,
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

    pub fn read_from_memory(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    pub fn write_to_memory(&mut self, address: u16, value: u8) {
        self.rom[address as usize] = value;
    }
}