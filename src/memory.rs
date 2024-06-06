use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use super::{bit_logic, TAC};
use super::gameboy::Gameboy;

impl Gameboy {
    pub(crate) fn load_cartridge_from_path(&mut self, rom_path: PathBuf) {
        let mut file = File::open(rom_path).expect("Invalid ROM path");
        file.read_to_end(&mut self.cartridge).expect("Unable to read ROM");
        for i in 0..0x8000 {
            self.rom[i] = self.cartridge[i];
        }
        match self.cartridge[0x147] {
            1..=3 => { self.mbc1 = true },
            5 | 6 => { self.mbc2 = true },
            _ => {},
        }
    }

    fn get_gamepad_state(&self) -> u8 {
        let mut res: u8 = self.rom[0xff00_usize] ^ 0xff;
        if !bit_logic::check_bit(res, 4) {
            res &= (self.gamepad_state >> 4) | 0xf0;
        } else if !bit_logic::check_bit(res, 5) {
            res &= (self.gamepad_state & 0xf) | 0xf0;
        }
        res
    }

    pub(crate) fn read_from_memory(&self, address: u16) -> u8 {
        match address {
            0x4000..=0x7fff => {
                self.cartridge[((address - 0x4000) + ((self.current_rom_bank as u16) * 0x4000)) as usize]
            },
            0xa000..=0xbfff => {
                self.ram_banks[((address - 0xa000) + ((self.current_ram_bank as u16) * 0x2000)) as usize]
            },
            0xfea0..=0xfeff => 0xff,
            0xff00 => self.get_gamepad_state(),
            _ => self.rom[address as usize],
        }
    }

    fn do_dma_transfer(&mut self, value: u8) {
        let address: u16 = (value as u16) << 8;
        for i in 0..0xa0 {
            self.write_to_memory(0xfe00 + i, self.read_from_memory(address + i));
        }
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
        self.current_rom_bank = (self.current_rom_bank & 224) | (value & 31);
        if self.current_rom_bank == 0 {
            self.current_rom_bank += 1;
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
                        self.current_rom_bank = (self.current_rom_bank & 31) | (value & 224);
                        if self.current_rom_bank == 0 {
                            self.current_rom_bank += 1;
                        }
                    } else {
                        self.current_ram_bank = value & 0x3;
                    }
                }
            },
            0x6000..=0x7fff => {
                if self.mbc1 {
                    self.rom_banking = (value & 0x1) == 0;
                    if self.rom_banking {
                        self.current_ram_bank = 0;
                    }
                }
            },
            _ => {},
        }
    }

    pub(crate) fn write_to_memory(&mut self, address: u16, value: u8) {
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
                if current_freq != new_freq {
                    self.set_clock_freq();
                }
            },
            0xff04 | 0xff44 => {
                self.rom[address as usize] = 0;
                if address == 0xff04 {
                    self.divider_counter = 0;
                }
            },
            0xff46 => {
                self.do_dma_transfer(value);
            },
            0xff14 => {
                self.rom[address as usize] = value;
                if value >> 7 == 1 {
                    self.reset_sound_channel_1(self.read_from_memory(0xff11) & 0x3f);
                }
            },
            0xff19 => {
                self.rom[address as usize] = value;
                if value >> 7 == 1 {
                    self.reset_sound_channel_2(self.read_from_memory(0xff16) & 0x3f);
                }
            },
            0xff1e => {
                self.rom[address as usize] = value;
                if value >> 7 == 1 {
                    self.reset_sound_channel_3(self.read_from_memory(0xff1b));
                }
            },
            0xff23 => {
                self.rom[address as usize] = value;
                if value >> 7 == 1 {
                    self.reset_sound_channel_4(self.read_from_memory(0xff20) & 0x3f);
                }
            },
            _ => { self.rom[address as usize] = value },
        }
    }

    pub(crate) fn request_interrupt(&mut self, interrupt_id: u8) {
        self.write_to_memory(0xff0f, bit_logic::set_bit(self.read_from_memory(0xff0f), interrupt_id));
    }
}