pub struct Memory {
    cartridge:[u8; 0x200000],
    rom: [u8; 0x10000],
}

impl Memory {
    pub fn new() -> Memory {
        let mut rom_array = [0; 0x10000];

        rom_array[0xff05] = 0x00;
        rom_array[0xff06] = 0x00;
        rom_array[0xff07] = 0x00;
        rom_array[0xff10] = 0x80;
        rom_array[0xff11] = 0xbf;
        rom_array[0xff12] = 0xf3;
        rom_array[0xff14] = 0xbf;
        rom_array[0xff16] = 0x3f;
        rom_array[0xff17] = 0x00;
        rom_array[0xff19] = 0xbf;
        rom_array[0xff1a] = 0x7f;
        rom_array[0xff1b] = 0xff;
        rom_array[0xff1c] = 0x9f;
        rom_array[0xff1e] = 0xbf;
        rom_array[0xff20] = 0xff;
        rom_array[0xff21] = 0x00;
        rom_array[0xff22] = 0x00;
        rom_array[0xff23] = 0xbf;
        rom_array[0xff24] = 0x77;
        rom_array[0xff25] = 0xf3;
        rom_array[0xff26] = 0xf1;
        rom_array[0xff40] = 0x91;
        rom_array[0xff42] = 0x00;
        rom_array[0xff43] = 0x00;
        rom_array[0xff45] = 0x00;
        rom_array[0xff47] = 0xfc;
        rom_array[0xff48] = 0xff;
        rom_array[0xff49] = 0xff;
        rom_array[0xff4a] = 0x00;
        rom_array[0xff4b] = 0x00;
        rom_array[0xffff] = 0x00;

        Memory {
            cartridge: [0; 0x200000],
            rom: rom_array,
        }
    }

    pub fn read_from_memory(&self, address: usize) -> u8 {
        self.rom[address]
    }

    pub fn write_to_memory(&mut self, address: usize, value: u8) {
        self.rom[address] = value;
    }
}