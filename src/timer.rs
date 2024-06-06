use super::{bit_logic, TAC};
use super::gameboy::Gameboy;

const TIMA: u16 = 0xff05;
const TMA: u16 = 0xff06;

const FREQUENCY_4096: u16 = 1024; // CYCLES_PER_SECOND / 4096
const FREQUENCY_262144: u16 = 16; // CYCLES_PER_SECOND / 262144
const FREQUENCY_65536: u16 = 64; // CYCLES_PER_SECOND / 65536
const FREQUENCY_16384: u16 = 256; // CYCLES_PER_SECOND / 16384

impl Gameboy {
    fn is_clock_enabled(&self) -> bool {
        bit_logic::check_bit(self.read_from_memory(TAC), 2)
    }

    fn get_clock_freq(&self) -> u8 {
        self.read_from_memory(TAC) & 0x3
    }

    pub(crate) fn set_clock_freq(&mut self) {
        match self.get_clock_freq() {
            0 => { self.timer_counter = FREQUENCY_4096 as i32 },
            1 => { self.timer_counter = FREQUENCY_262144 as i32 },
            2 => { self.timer_counter = FREQUENCY_65536 as i32 },
            3 => { self.timer_counter = FREQUENCY_16384 as i32 },
            _ => { },
        }
    }

    pub(crate) fn update_timer(&mut self, cycles: u8) {
        self.divider_counter += cycles as i32;
        if self.divider_counter >= 255 {
            self.divider_counter = 0;
            self.rom[0xff04_usize] += 1;
        }

        if self.is_clock_enabled() {
            self.timer_counter -= cycles as i32;
            if self.timer_counter <= 0 {
                self.set_clock_freq();
                let (tima, tma): (u8, u8) = (self.read_from_memory(TIMA), self.read_from_memory(TMA));
                if tima == 255 {
                    self.write_to_memory(TIMA, tma);
                    self.request_interrupt(2);
                } else {
                    self.write_to_memory(TIMA, tima + 1);
                }
            }
        }
    }
}