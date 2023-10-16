use super::{memory::Memory, bit_logic, MemoryWriteResult, TAC};

const TIMA: u16 = 0xff05;
const TMA: u16 = 0xff06;

const FREQUENCY_4096: u16 = 1024; // CYCLES_PER_SECOND / 4096
const FREQUENCY_262144: u16 = 16; // CYCLES_PER_SECOND / 262144
const FREQUENCY_65536: u16 = 64; // CYCLES_PER_SECOND / 65536
const FREQUENCY_16384: u16 = 256; // CYCLES_PER_SECOND / 16384

pub(crate) struct Timer {
    pub(crate) timer_counter: i32,
    pub(crate) divider_counter: i32,
}

impl Timer {
    pub(crate) fn new() -> Timer {
        Timer {
            timer_counter: 0,
            divider_counter: 0,
        }
    }

    fn is_clock_enabled(&self, memory: &Memory) -> bool {
        bit_logic::check_bit(memory.read_from_memory(TAC), 2)
    }

    fn get_clock_freq(&self, memory: &Memory) -> u8 {
        memory.read_from_memory(TAC) & 0x3
    }

    pub(crate) fn set_clock_freq(&mut self, memory: &Memory) {
        match self.get_clock_freq(memory) {
            0 => { self.timer_counter = FREQUENCY_4096 as i32 },
            1 => { self.timer_counter = FREQUENCY_262144 as i32 },
            2 => { self.timer_counter = FREQUENCY_65536 as i32 },
            3 => { self.timer_counter = FREQUENCY_16384 as i32 },
            _ => { },
        }
    }

    pub(crate) fn update_timer(&mut self, memory: &mut Memory, cycles: u8) -> Vec<MemoryWriteResult> {
        let mut result = Vec::new();

        self.divider_counter += cycles as i32;
        if self.divider_counter >= 255 {
            self.divider_counter = 0;
            memory.rom[0xff04 as usize] = memory.rom[0xff04 as usize].wrapping_add(1);
        }

        if self.is_clock_enabled(memory) {
            self.timer_counter -= cycles as i32;
            if self.timer_counter <= 0 {
                self.set_clock_freq(memory);
                let (tima, tma): (u8, u8) = (memory.read_from_memory(TIMA), memory.read_from_memory(TMA));
                if tima == 255 {
                    result = memory.write_to_memory(TIMA, tma);
                    memory.request_interrupt(2);
                } else {
                    result = memory.write_to_memory(TIMA, tima.wrapping_add(1));
                }
            }
        }
        result
    }
}