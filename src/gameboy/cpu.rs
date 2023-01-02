use crate::gameboy::Vec;

use crate::bit_logic;
use super::memory::Memory;
use super::MemoryWriteResult;

const INSTRUCTION_TIMINGS: [u8; 256] = [
    1,3,2,2,1,1,2,1,5,2,2,2,1,1,2,1,
    1,3,2,2,1,1,2,1,3,2,2,2,1,1,2,1,
    2,3,2,2,1,1,2,1,2,2,2,2,1,1,2,1,
    2,3,2,2,3,3,3,1,2,2,2,2,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    2,2,2,2,2,2,1,2,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    2,3,3,4,3,4,2,4,2,4,3,0,3,6,2,4,
    2,3,3,0,3,4,2,4,2,4,3,0,3,0,2,4,
    3,3,2,0,0,4,2,4,4,1,4,0,0,0,2,4,
    3,3,2,1,0,4,2,4,3,2,4,1,0,0,2,4,
];
const BRANCH_INSTRUCTION_TIMINGS: [u8; 256] = [
    1,3,2,2,1,1,2,1,5,2,2,2,1,1,2,1,
    1,3,2,2,1,1,2,1,3,2,2,2,1,1,2,1,
    3,3,2,2,1,1,2,1,3,2,2,2,1,1,2,1,
    3,3,2,2,3,3,3,1,3,2,2,2,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    2,2,2,2,2,2,1,2,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
    5,3,4,4,6,4,2,4,5,4,4,0,6,6,2,4,
    5,3,4,0,6,4,2,4,5,4,4,0,6,0,2,4,
    3,3,2,0,0,4,2,4,4,1,4,0,0,0,2,4,
    3,3,2,1,0,4,2,4,3,2,4,1,0,0,2,4,
];
const CB_INSTRUCTION_TIMINGS: [u8; 256] = [
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
    2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
    2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
    2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
];

pub(crate) struct Cpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pub(crate) pc: u16,
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
    pub(crate) halted: bool,
    pub(crate) interrupts_enabled: bool,
    pub(crate) pending_interrupt_enable: bool,
    pub(crate) one_instruction_passed: bool,
}

impl Cpu {
    pub(crate) fn new() -> Cpu {
        Cpu {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xd8,
            h: 0x01,
            l: 0x4d,
            sp: 0xfffe,
            pc: 0x0100,
            zero: true,
            subtract: false,
            half_carry: true,
            carry: true,
            halted: false,
            interrupts_enabled: false,
            pending_interrupt_enable: false,
            one_instruction_passed: false,
        }
    }

    fn get_f(&self) -> u8 {
        let mut f: u8 = 0x0;
        f = bit_logic::set_bit_to(self.zero, f, 7);
        f = bit_logic::set_bit_to(self.subtract, f, 6);
        f = bit_logic::set_bit_to(self.half_carry, f, 5);
        f = bit_logic::set_bit_to(self.carry, f, 4);
        f
    }

    fn set_f(&mut self, new_f: u8) {
        self.carry = bit_logic::check_bit(new_f, 4);
        self.half_carry = bit_logic::check_bit(new_f, 5);
        self.subtract = bit_logic::check_bit(new_f, 6);
        self.zero = bit_logic::check_bit(new_f, 7);
    }

    fn fetch(&mut self, memory: &Memory) -> u8 {
        let value: u8 = memory.read_from_memory(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    fn pop(&mut self, memory: &Memory) -> u8 {
        let value: u8 = memory.read_from_memory(self.sp);
        self.sp = self.sp.wrapping_add(1);
        value
    }
    
    pub fn push(&mut self, memory: &mut Memory, value: u8) -> Vec<MemoryWriteResult> {
        self.sp = self.sp.wrapping_sub(1);
        memory.write_to_memory(self.sp, value)
    }

    fn rlc(&mut self, value: u8) -> u8 {
        let truncated: u8 = if bit_logic::check_bit(value, 7) { 1 } else { 0 };
        let result: u8 = (value << 1) | truncated;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = bit_logic::check_bit(value, 7);
        result
    }
    
    fn rrc(&mut self, value: u8) -> u8 {
        let truncated: u8 = if bit_logic::check_bit(value, 0) { 1 } else { 0 };
        let result: u8 = (value >> 1) | (truncated << 7);
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = bit_logic::check_bit(value, 0);
        result
    }
    
    fn rl(&mut self, value: u8) -> u8 {
        let result: u8 = value << 1 | self.carry as u8;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = bit_logic::check_bit(value, 7);
        result
    }
    
    fn rr(&mut self, value: u8) -> u8 {
        let result: u8 = value >> 1 | (self.carry as u8) << 7;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = bit_logic::check_bit(value, 0);
        result
    }
    
    fn sla(&mut self, value: u8) -> u8 {
        let result: u8 = value << 1;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = bit_logic::check_bit(value, 7);
        result
    }
    
    fn sra(&mut self, value: u8) -> u8 {
        let mut result: u8 = value >> 1;
        result = bit_logic::set_bit_to(bit_logic::check_bit(value, 7), result, 7);
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = bit_logic::check_bit(value, 0);
        result
    }
    
    fn srl(&mut self, value: u8) -> u8 {
        let result: u8 = value >> 1;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = bit_logic::check_bit(value, 0);
        result
    }
    
    fn swap(&mut self, value: u8) -> u8 {
        let lower: u8 = value & 0x0f;
        let upper: u8 = (value & 0xf0) >> 4;
        let result: u8 = (lower << 4) | upper;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = false;
        result
    }
    
    fn bit(&mut self, bit: u8, reg: u8) {
        self.zero = !bit_logic::check_bit(reg, bit);
        self.subtract = false;
        self.half_carry = true;
    }
    
    fn set(bit: u8, reg: &mut u8) { *reg = bit_logic::set_bit(*reg, bit); }
    
    fn res(bit: u8, reg: &mut u8) { *reg = bit_logic::reset_bit(*reg, bit); }

    fn ld_word(lower_des: &mut u8, upper_des: &mut u8, lower: u8, upper: u8) {
        *lower_des = lower;
        *upper_des = upper;
    }

    fn ld_byte(des: &mut u8, src: u8) { *des = src; }

    fn inc_word(lower: &mut u8, upper: &mut u8) {
        let mut word: u16 = bit_logic::compose_bytes(*lower, *upper);
        word = word.wrapping_add(1);
        *upper = (word >> 8) as u8;
        *lower = word as u8;
    }

    fn dec_word(lower: &mut u8, upper: &mut u8) {
        let mut word: u16 = bit_logic::compose_bytes(*lower, *upper);
        word = word.wrapping_sub(1);
        *upper = (word >> 8) as u8;
        *lower = word as u8;
    }

    fn inc_byte(&mut self, value: u8) -> u8 {
        let result: u8 = value.wrapping_add(1);
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = (result & 0xf) == 0;
        result
    }

    fn dec_byte(&mut self, value: u8) -> u8 {
        let result: u8 = value.wrapping_sub(1);
        self.zero = result == 0;
        self.subtract = true;
        self.half_carry = (result & 0x0f) == 0x0f;
        result
    }

    fn add_word(&mut self, lower_value: u8, upper_value: u8, lower_addend: u8, upper_addend: u8) -> (u8, u8) {
        let value_word: u16 = bit_logic::compose_bytes(lower_value, upper_value);
        let addend_word: u16 = bit_logic::compose_bytes(lower_addend, upper_addend);
        let result: u16 = value_word.wrapping_add(addend_word);
        self.subtract = false;
        self.half_carry = (value_word & 0x07ff) + (addend_word & 0x07ff) > 0x07ff;
        self.carry = value_word > 0xffff - addend_word;
        (result as u8, (result >> 8) as u8)
    }

    fn add_byte(&mut self, value: u8, addend: u8) -> u8 {
        let first: u8 = value;
        let second: u8 = addend;
        let result: u8 = first.wrapping_add(second);
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = (first & 0xf) + (second & 0xf) > 0xf;
        self.carry = (first as u16) + (second as u16) > 0xff;
        result
    }

    fn adc_byte(&mut self, value: u8, addend: u8) -> u8 {
        let first: u8 = value;
        let second: u8 = addend;
        let carry: u8 = self.carry as u8;
        let result: u8 = first.wrapping_add(second).wrapping_add(carry);
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = ((first & 0xf) + (second & 0xf) + carry) > 0xf;
        self.carry = (first as u16) + (second as u16) + (carry as u16) > 0xff;
        result
    }

    fn sub_byte(&mut self, value: u8, subtrahend: u8) -> u8 {
        let first: u8 = value;
        let second: u8 = subtrahend;
        let result: u8 = first.wrapping_sub(second);
        self.zero = result == 0;
        self.subtract = true;
        self.half_carry = (first & 0xf) < (second & 0xf);
        self.carry = first < second;
        result
    }

    fn sbc_byte(&mut self, value: u8, subtrahend: u8) -> u8 {
        let first: u8 = value;
        let second: u8 = subtrahend;
        let carry: u8 = self.carry as u8;
        let result: u8 = first.wrapping_sub(second).wrapping_sub(carry);
        self.zero = result == 0;
        self.subtract = true;
        self.half_carry = (first & 0xf) < (second & 0xf) + carry;
        self.carry = (first as u16) < (second as u16) + (carry as u16);
        result
    }

    fn and_byte(&mut self, value: u8, anding_value: u8) -> u8 {
        let result: u8 = value & anding_value;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = true;
        self.carry = false;
        result
    }

    fn xor_byte(&mut self, value: u8, xoring_value: u8) -> u8 {
        let result: u8 = value ^ xoring_value;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = false;
        result
    }

    fn or_byte(&mut self, value: u8, oring_value: u8) -> u8 {
        let result: u8 = value | oring_value;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = false;
        result
    }

    fn cp_byte(&mut self, value: u8, cping_value: u8) {
        let first: u8 = value;
        let second: u8 = cping_value;
        let result: u8 = first.wrapping_sub(second);
        self.zero = result == 0;
        self.subtract = true;
        self.half_carry = (first & 0xf) < (second & 0xf);
        self.carry = first < second;
    }

    fn ret(&mut self, memory: &mut Memory) {
        let lower: u8 = self.pop(memory);
        let upper: u8 = self.pop(memory);
        let pc: u16 = bit_logic::compose_bytes(lower, upper);
        /*
        if(gameBoy->eiHaltBug) {
            pc--;
            gameBoy->eiHaltBug = false;
        }
        */
        self.jp_from_word(pc);
    }

    fn jp_from_word(&mut self, address: u16) { self.pc = address; }

    fn jp_from_bytes(&mut self, lower: u8, upper: u8) { self.jp_from_word(bit_logic::compose_bytes(lower, upper)); }

    fn jp_from_pc(&mut self, memory: &Memory) {
        let lower: u8 = self.fetch(memory);
        let upper: u8 = self.fetch(memory);
        self.jp_from_bytes(lower, upper);
    }

    fn call(&mut self, memory: &mut Memory) -> Vec<MemoryWriteResult> {
        let mut memory_write_results = Vec::new();
        let lower_new: u8 = self.fetch(memory);
        let upper_new: u8 = self.fetch(memory);
        memory_write_results.append(&mut self.push(memory, (self.pc >> 8) as u8));
        memory_write_results.append(&mut self.push(memory, self.pc as u8));
        self.jp_from_bytes(lower_new, upper_new);
        memory_write_results
    }

    fn rst(&mut self, memory: &mut Memory, value: u8) -> Vec<MemoryWriteResult> {
        let mut memory_write_results = Vec::new();
        memory_write_results.append(&mut self.push(memory, (self.pc >> 8) as u8));
        memory_write_results.append(&mut self.push(memory, self.pc as u8));
        self.jp_from_word(0x0000 + value as u16);
        memory_write_results
    }

    fn jr(&mut self, memory: &Memory) {
        let value: u16 = (self.fetch(memory) as i8) as u16;
        self.jp_from_word(self.pc.wrapping_add(value));
    }

    pub(crate) fn update(&mut self, memory: &mut Memory) -> (u8, Vec<MemoryWriteResult>) {
        let instruction: u8 = self.fetch(memory);
        let (cycles, memory_write_results) = self.execute(memory, instruction);
        if self.pending_interrupt_enable {
            if self.one_instruction_passed {
                if !self.interrupts_enabled {
                    self.interrupts_enabled = true;
                }
                self.pending_interrupt_enable = false;
                self.one_instruction_passed = false;
            } else {
                self.one_instruction_passed = true;
            }
        }
        (cycles, memory_write_results)
    }

    fn execute_cb(&mut self, memory: &mut Memory, instruction: u8) -> (u8, Vec<MemoryWriteResult>) {
        let mut memory_write_results: Vec<MemoryWriteResult> = Vec::new();
        match instruction {
            0x00 => {
                // RLC B
                self.b = self.rlc(self.b);
            },
            0x01 => {
                // RLC C
                self.c = self.rlc(self.c);
            },
            0x02 => {
                // RLC D
                self.d = self.rlc(self.d);
            },
            0x03 => {
                // RLC E
                self.e = self.rlc(self.e);
            },
            0x04 => {
                // RLC H
                self.h = self.rlc(self.h);
            },
            0x05 => {
                // RLC L
                self.l = self.rlc(self.l);
            },
            0x06 => {
                // RLC (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                value = self.rlc(value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x07 => {
                // RLC A
                self.a = self.rlc(self.a);
            },
            0x08 => {
                // RRC B
                self.b = self.rrc(self.b);
            },
            0x09 => {
                // RRC C
                self.c = self.rrc(self.c);
            },
            0x0a => {
                // RRC D
                self.d = self.rrc(self.d);
            },
            0x0b => {
                // RRC E
                self.e = self.rrc(self.e);
            },
            0x0c => {
                // RRC H
                self.h = self.rrc(self.h);
            },
            0x0d => {
                // RRC L
                self.l = self.rrc(self.l);
            },
            0x0e => {
                // RRC (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                value = self.rrc(value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x0f => {
                // RRC A
                self.a = self.rrc(self.a);
            },
            0x10 => {
                // RL B
                self.b = self.rl(self.b);
            },
            0x11 => {
                // RL C
                self.c = self.rl(self.c);
            },
            0x12 => {
                // RL D
                self.d = self.rl(self.d);
            },
            0x13 => {
                // RL E
                self.e = self.rl(self.e);
            },
            0x14 => {
                // RL H
                self.h = self.rl(self.h);
            },
            0x15 => {
                // RL L
                self.l = self.rl(self.l);
            },
            0x16 => {
                // RL (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                value = self.rl(value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x17 => {
                // RL A
                self.a = self.rl(self.a);
            },
            0x18 => {
                // RR B
                self.b = self.rr(self.b);
            },
            0x19 => {
                // RR C
                self.c = self.rr(self.c);
            },
            0x1a => {
                // RR D
                self.d = self.rr(self.d);
            },
            0x1b => {
                // RR E
                self.e = self.rr(self.e);
            },
            0x1c => {
                // RR H
                self.h = self.rr(self.h);
            },
            0x1d => {
                // RR L
                self.l = self.rr(self.l);
            },
            0x1e => {
                // RR (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                value = self.rr(value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x1f => {
                // RR A
                self.a = self.rr(self.a);
            },
            0x20 => {
                // SLA B
                self.b = self.sla(self.b);
            },
            0x21 => {
                // SLA C
                self.c = self.sla(self.c);
            },
            0x22 => {
                // SLA D
                self.d = self.sla(self.d);
            },
            0x23 => {
                // SLA E
                self.e = self.sla(self.e);
            },
            0x24 => {
                // SLA H
                self.h = self.sla(self.h);
            },
            0x25 => {
                // SLA L
                self.l = self.sla(self.l);
            },
            0x26 => {
                // SLA (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                value = self.sla(value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x27 => {
                // SLA A
                self.a = self.sla(self.a);
            },
            0x28 => {
                // SRA B
                self.b = self.sra(self.b);
            },
            0x29 => {
                // SRA C
                self.c = self.sra(self.c);
            },
            0x2a => {
                // SRA D
                self.d = self.sra(self.d);
            },
            0x2b => {
                // SRA E
                self.e = self.sra(self.e);
            },
            0x2c => {
                // SRA H
                self.h = self.sra(self.h);
            },
            0x2d => {
                // SRA L
                self.l = self.sra(self.l);
            },
            0x2e => {
                // SRA (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                value = self.sra(value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x2f => {
                // SRA A
                self.a = self.sra(self.a);
            },
            0x30 => {
                // SWAP B
                self.b = self.swap(self.b);
            },
            0x31 => {
                // SWAP C
                self.c = self.swap(self.c);
            },
            0x32 => {
                // SWAP D
                self.d = self.swap(self.d);
            },
            0x33 => {
                // SWAP E
                self.e = self.swap(self.e);
            },
            0x34 => {
                // SWAP H
                self.h = self.swap(self.h);
            },
            0x35 => {
                // SWAP L
                self.l = self.swap(self.l);
            },
            0x36 => {
                // SWAP (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                value = self.swap(value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x37 => {
                // SWAP A
                self.a = self.swap(self.a);
            },
            0x38 => {
                // SRL B
                self.b = self.srl(self.b);
            },
            0x39 => {
                // SRL C
                self.c = self.srl(self.c);
            },
            0x3a => {
                // SRL D
                self.d = self.srl(self.d);
            },
            0x3b => {
                // SRL E
                self.e = self.srl(self.e);
            },
            0x3c => {
                // SRL H
                self.h = self.srl(self.h);
            },
            0x3d => {
                // SRL L
                self.l = self.srl(self.l);
            },
            0x3e => {
                // SRL (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                value = self.srl(value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x3f => {
                // SRL A
                self.a = self.srl(self.a);
            },
            0x40 => {
                // BIT 0, B
                self.bit(0, self.b);
            },
            0x41 => {
                // BIT 0, C
                self.bit(0, self.c);
            },
            0x42 => {
                // BIT 0, D
                self.bit(0, self.d);
            },
            0x43 => {
                // BIT 0, E
                self.bit(0, self.e);
            },
            0x44 => {
                // BIT 0, H
                self.bit(0, self.h);
            },
            0x45 => {
                // BIT 0, L
                self.bit(0, self.l);
            },
            0x46 => {
                // BIT 0, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.bit(0, value);
            },
            0x47 => {
                // BIT 0, A
                self.bit(0, self.a);
            },
            0x48 => {
                // BIT 1, B
                self.bit(1, self.b);
            },
            0x49 => {
                // BIT 1, C
                self.bit(1, self.c);
            },
            0x4a => {
                // BIT 1, D
                self.bit(1, self.d);
            },
            0x4b => {
                // BIT 1, E
                self.bit(1, self.e);
            },
            0x4c => {
                // BIT 1, H
                self.bit(1, self.h);
            },
            0x4d => {
                // BIT 1, L
                self.bit(1, self.l);
            },
            0x4e => {
                // BIT 1, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.bit(1, value);
            },
            0x4f => {
                // BIT 1, A
                self.bit(1, self.a);
            },
            0x50 => {
                // BIT 2, B
                self.bit(2, self.b);
            },
            0x51 => {
                // BIT 2, C
                self.bit(2, self.c);
            },
            0x52 => {
                // BIT 2, D
                self.bit(2, self.d);
            },
            0x53 => {
                // BIT 2, E
                self.bit(2, self.e);
            },
            0x54 => {
                // BIT 2, H
                self.bit(2, self.h);
            },
            0x55 => {
                // BIT 2, L
                self.bit(2, self.l);
            },
            0x56 => {
                // BIT 2, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.bit(2, value);
            },
            0x57 => {
                // BIT 2, A
                self.bit(2, self.a);
            },
            0x58 => {
                // BIT 3, B
                self.bit(3, self.b);
            },
            0x59 => {
                // BIT 3, C
                self.bit(3, self.c);
            },
            0x5a => {
                // BIT 3, D
                self.bit(3, self.d);
            },
            0x5b => {
                // BIT 3, E
                self.bit(3, self.e);
            },
            0x5c => {
                // BIT 3, H
                self.bit(3, self.h);
            },
            0x5d => {
                // BIT 3, L
                self.bit(3, self.l);
            },
            0x5e => {
                // BIT 3, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.bit(3, value);
            },
            0x5f => {
                // BIT 3, A
                self.bit(3, self.a);
            },
            0x60 => {
                // BIT 4, B
                self.bit(4, self.b);
            },
            0x61 => {
                // BIT 4, C
                self.bit(4, self.c);
            },
            0x62 => {
                // BIT 4, D
                self.bit(4, self.d);
            },
            0x63 => {
                // BIT 4, E
                self.bit(4, self.e);
            },
            0x64 => {
                // BIT 4, H
                self.bit(4, self.h);
            },
            0x65 => {
                // BIT 4, L
                self.bit(4, self.l);
            },
            0x66 => {
                // BIT 4, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.bit(4, value);
            },
            0x67 => {
                // BIT 4, A
                self.bit(4, self.a);
            },
            0x68 => {
                // BIT 5, B
                self.bit(5, self.b);
            },
            0x69 => {
                // BIT 5, C
                self.bit(5, self.c);
            },
            0x6a => {
                // BIT 5, D
                self.bit(5, self.d);
            },
            0x6b => {
                // BIT 5, E
                self.bit(5, self.e);
            },
            0x6c => {
                // BIT 5, H
                self.bit(5, self.h);
            },
            0x6d => {
                // BIT 5, L
                self.bit(5, self.l);
            },
            0x6e => {
                // BIT 5, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.bit(5, value);
            },
            0x6f => {
                // BIT 5, A
                self.bit(5, self.a);
            },
            0x70 => {
                // BIT 6, B
                self.bit(6, self.b);
            },
            0x71 => {
                // BIT 6, C
                self.bit(6, self.c);
            },
            0x72 => {
                // BIT 6, D
                self.bit(6, self.d);
            },
            0x73 => {
                // BIT 6, E
                self.bit(6, self.e);
            },
            0x74 => {
                // BIT 6, H
                self.bit(6, self.h);
            },
            0x75 => {
                // BIT 6, L
                self.bit(6, self.l);
            },
            0x76 => {
                // BIT 6, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.bit(6, value);
            },
            0x77 => {
                // BIT 6, A
                self.bit(6, self.a);
            },
            0x78 => {
                // BIT 7, B
                self.bit(7, self.b);
            },
            0x79 => {
                // BIT 7, C
                self.bit(7, self.c);
            },
            0x7a => {
                // BIT 7, D
                self.bit(7, self.d);
            },
            0x7b => {
                // BIT 7, E
                self.bit(7, self.e);
            },
            0x7c => {
                // BIT 7, H
                self.bit(7, self.h);
            },
            0x7d => {
                // BIT 7, L
                self.bit(7, self.l);
            },
            0x7e => {
                // BIT 7, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.bit(7, value);
            },
            0x7f => {
                // BIT 7, A
                self.bit(7, self.a);
            },
            0x80 => {
                // RES 0, B
                Cpu::res(0, &mut self.b);
            },
            0x81 => {
                // RES 0, C
                Cpu::res(0, &mut self.c);
            },
            0x82 => {
                // RES 0, D
                Cpu::res(0, &mut self.d);
            },
            0x83 => {
                // RES 0, E
                Cpu::res(0, &mut self.e);
            },
            0x84 => {
                // RES 0, H
                Cpu::res(0, &mut self.h);
            },
            0x85 => {
                // RES 0, L
                Cpu::res(0, &mut self.l);
            },
            0x86 => {
                // RES 0, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::res(0, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x87 => {
                // RES 0, A
                Cpu::res(0, &mut self.a);
            },
            0x88 => {
                // RES 1, B
                Cpu::res(1, &mut self.b);
            },
            0x89 => {
                // RES 1, C
                Cpu::res(1, &mut self.c);
            },
            0x8a => {
                // RES 1, D
                Cpu::res(1, &mut self.d);
            },
            0x8b => {
                // RES 1, E
                Cpu::res(1, &mut self.e);
            },
            0x8c => {
                // RES 1, H
                Cpu::res(1, &mut self.h);
            },
            0x8d => {
                // RES 1, L
                Cpu::res(1, &mut self.l);
            },
            0x8e => {
                // RES 1, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::res(1, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x8f => {
                // RES 1, A
                Cpu::res(1, &mut self.a);
            },
            0x90 => {
                // RES 2, B
                Cpu::res(2, &mut self.b);
            },
            0x91 => {
                // RES 2, C
                Cpu::res(2, &mut self.c);
            },
            0x92 => {
                // RES 2, D
                Cpu::res(2, &mut self.d);
            },
            0x93 => {
                // RES 2, E
                Cpu::res(2, &mut self.e);
            },
            0x94 => {
                // RES 2, H
                Cpu::res(2, &mut self.h);
            },
            0x95 => {
                // RES 2, L
                Cpu::res(2, &mut self.l);
            },
            0x96 => {
                // RES 2, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::res(2, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x97 => {
                // RES 2, A
                Cpu::res(2, &mut self.a);
            },
            0x98 => {
                // RES 3, B
                Cpu::res(3, &mut self.b);
            },
            0x99 => {
                // RES 3, C
                Cpu::res(3, &mut self.c);
            },
            0x9a => {
                // RES 3, D
                Cpu::res(3, &mut self.d);
            },
            0x9b => {
                // RES 3, E
                Cpu::res(3, &mut self.e);
            },
            0x9c => {
                // RES 3, H
                Cpu::res(3, &mut self.h);
            },
            0x9d => {
                // RES 3, L
                Cpu::res(3, &mut self.l);
            },
            0x9e => {
                // RES 3, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::res(3, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x9f => {
                // RES 3, A
                Cpu::res(3, &mut self.a);
            },
            0xa0 => {
                // RES 4, B
                Cpu::res(4, &mut self.b);
            },
            0xa1 => {
                // RES 4, C
                Cpu::res(4, &mut self.c);
            },
            0xa2 => {
                // RES 4, D
                Cpu::res(4, &mut self.d);
            },
            0xa3 => {
                // RES 4, E
                Cpu::res(4, &mut self.e);
            },
            0xa4 => {
                // RES 4, H
                Cpu::res(4, &mut self.h);
            },
            0xa5 => {
                // RES 4, L
                Cpu::res(4, &mut self.l);
            },
            0xa6 => {
                // RES 4, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::res(4, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xa7 => {
                // RES 4, A
                Cpu::res(4, &mut self.a);
            },
            0xa8 => {
                // RES 5, B
                Cpu::res(5, &mut self.b);
            },
            0xa9 => {
                // RES 5, C
                Cpu::res(5, &mut self.c);
            },
            0xaa => {
                // RES 5, D
                Cpu::res(5, &mut self.d);
            },
            0xab => {
                // RES 5, E
                Cpu::res(5, &mut self.e);
            },
            0xac => {
                // RES 5, H
                Cpu::res(5, &mut self.h);
            },
            0xad => {
                // RES 5, L
                Cpu::res(5, &mut self.l);
            },
            0xae => {
                // RES 5, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::res(5, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xaf => {
                // RES 5, A
                Cpu::res(5, &mut self.a);
            },
            0xb0 => {
                // RES 6, B
                Cpu::res(6, &mut self.b);
            },
            0xb1 => {
                // RES 6, C
                Cpu::res(6, &mut self.c);
            },
            0xb2 => {
                // RES 6, D
                Cpu::res(6, &mut self.d);
            },
            0xb3 => {
                // RES 6, E
                Cpu::res(6, &mut self.e);
            },
            0xb4 => {
                // RES 6, H
                Cpu::res(6, &mut self.h);
            },
            0xb5 => {
                // RES 6, L
                Cpu::res(6, &mut self.l);
            },
            0xb6 => {
                // RES 6, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::res(6, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xb7 => {
                // RES 6, A
                Cpu::res(6, &mut self.a);
            },
            0xb8 => {
                // RES 7, B
                Cpu::res(7, &mut self.b);
            },
            0xb9 => {
                // RES 7, C
                Cpu::res(7, &mut self.c);
            },
            0xba => {
                // RES 7, D
                Cpu::res(7, &mut self.d);
            },
            0xbb => {
                // RES 7, E
                Cpu::res(7, &mut self.e);
            },
            0xbc => {
                // RES 7, H
                Cpu::res(7, &mut self.h);
            },
            0xbd => {
                // RES 7, L
                Cpu::res(7, &mut self.l);
            },
            0xbe => {
                // RES 7, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::res(7, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xbf => {
                // RES 7, A
                Cpu::res(7, &mut self.a);
            },
            0xc0 => {
                // SET 0, B
                Cpu::set(0, &mut self.b);
            },
            0xc1 => {
                // SET 0, C
                Cpu::set(0, &mut self.c);
            },
            0xc2 => {
                // SET 0, D
                Cpu::set(0, &mut self.d);
            },
            0xc3 => {
                // SET 0, E
                Cpu::set(0, &mut self.e);
            },
            0xc4 => {
                // SET 0, H
                Cpu::set(0, &mut self.h);
            },
            0xc5 => {
                // SET 0, L
                Cpu::set(0, &mut self.l);
            },
            0xc6 => {
                // SET 0, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::set(0, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xc7 => {
                // SET 0, A
                Cpu::set(0, &mut self.a);
            },
            0xc8 => {
                // SET 1, B
                Cpu::set(1, &mut self.b);
            },
            0xc9 => {
                // SET 1, C
                Cpu::set(1, &mut self.c);
            },
            0xca => {
                // SET 1, D
                Cpu::set(1, &mut self.d);
            },
            0xcb => {
                // SET 1, E
                Cpu::set(1, &mut self.e);
            },
            0xcc => {
                // SET 1, H
                Cpu::set(1, &mut self.h);
            },
            0xcd => {
                // SET 1, L
                Cpu::set(1, &mut self.l);
            },
            0xce => {
                // SET 1, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::set(1, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xcf => {
                // SET 1, A
                Cpu::set(1, &mut self.a);
            },
            0xd0 => {
                // SET 2, B
                Cpu::set(2, &mut self.b);
            },
            0xd1 => {
                // SET 2, C
                Cpu::set(2, &mut self.c);
            },
            0xd2 => {
                // SET 2, D
                Cpu::set(2, &mut self.d);
            },
            0xd3 => {
                // SET 2, E
                Cpu::set(2, &mut self.e);
            },
            0xd4 => {
                // SET 2, H
                Cpu::set(2, &mut self.h);
            },
            0xd5 => {
                // SET 2, L
                Cpu::set(2, &mut self.l);
            },
            0xd6 => {
                // SET 2, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::set(2, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xd7 => {
                // SET 2, A
                Cpu::set(2, &mut self.a);
            },
            0xd8 => {
                // SET 3, B
                Cpu::set(3, &mut self.b);
            },
            0xd9 => {
                // SET 3, C
                Cpu::set(3, &mut self.c);
            },
            0xda => {
                // SET 3, D
                Cpu::set(3, &mut self.d);
            },
            0xdb => {
                // SET 3, E
                Cpu::set(3, &mut self.e);
            },
            0xdc => {
                // SET 3, H
                Cpu::set(3, &mut self.h);
            },
            0xdd => {
                // SET 3, L
                Cpu::set(3, &mut self.l);
            },
            0xde => {
                // SET 3, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::set(3, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xdf => {
                // SET 3, A
                Cpu::set(3, &mut self.a);
            },
            0xe0 => {
                // SET 4, B
                Cpu::set(4, &mut self.b);
            },
            0xe1 => {
                // SET 4, C
                Cpu::set(4, &mut self.c);
            },
            0xe2 => {
                // SET 4, D
                Cpu::set(4, &mut self.d);
            },
            0xe3 => {
                // SET 4, E
                Cpu::set(4, &mut self.e);
            },
            0xe4 => {
                // SET 4, H
                Cpu::set(4, &mut self.h);
            },
            0xe5 => {
                // SET 4, L
                Cpu::set(4, &mut self.l);
            },
            0xe6 => {
                // SET 4, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::set(4, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xe7 => {
                // SET 4, A
                Cpu::set(4, &mut self.a);
            },
            0xe8 => {
                // SET 5, B
                Cpu::set(5, &mut self.b);
            },
            0xe9 => {
                // SET 5, C
                Cpu::set(5, &mut self.c);
            },
            0xea => {
                // SET 5, D
                Cpu::set(5, &mut self.d);
            },
            0xeb => {
                // SET 5, E
                Cpu::set(5, &mut self.e);
            },
            0xec => {
                // SET 5, H
                Cpu::set(5, &mut self.h);
            },
            0xed => {
                // SET 5, L
                Cpu::set(5, &mut self.l);
            },
            0xee => {
                // SET 5, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::set(5, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xef => {
                // SET 5, A
                Cpu::set(5, &mut self.a);
            },
            0xf0 => {
                // SET 6, B
                Cpu::set(6, &mut self.b);
            },
            0xf1 => {
                // SET 6, C
                Cpu::set(6, &mut self.c);
            },
            0xf2 => {
                // SET 6, D
                Cpu::set(6, &mut self.d);
            },
            0xf3 => {
                // SET 6, E
                Cpu::set(6, &mut self.e);
            },
            0xf4 => {
                // SET 6, H
                Cpu::set(6, &mut self.h);
            },
            0xf5 => {
                // SET 6, L
                Cpu::set(6, &mut self.l);
            },
            0xf6 => {
                // SET 6, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::set(6, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xf7 => {
                // SET 6, A
                Cpu::set(6, &mut self.a);
            },
            0xf8 => {
                // SET 7, B
                Cpu::set(7, &mut self.b);
            },
            0xf9 => {
                // SET 7, C
                Cpu::set(7, &mut self.c);
            },
            0xfa => {
                // SET 7, D
                Cpu::set(7, &mut self.d);
            },
            0xfb => {
                // SET 7, E
                Cpu::set(7, &mut self.e);
            },
            0xfc => {
                // SET 7, H
                Cpu::set(7, &mut self.h);
            },
            0xfd => {
                // SET 7, L
                Cpu::set(7, &mut self.l);
            },
            0xfe => {
                // SET 7, (HL)
                let mut value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::set(7, &mut value);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0xff => {
                // SET 7, A
                Cpu::set(7, &mut self.a);
            },
        }
        (CB_INSTRUCTION_TIMINGS[usize::from(instruction)], memory_write_results)
    }

    fn execute(&mut self, memory: &mut Memory, instruction: u8) -> (u8, Vec<MemoryWriteResult>) {
        let mut memory_write_results: Vec<MemoryWriteResult> = Vec::new();
        let mut branch_taken: bool = false;
        match instruction {
            0x00 => {
                // NOP
            },
            0x01 => {
                // LD BC, u16
                let lower: u8 = self.fetch(memory);
                let upper: u8 = self.fetch(memory);
                Cpu::ld_word(&mut self.c, &mut self.b, lower, upper);
            },
            0x02 => {
                // LD (BC), A
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.c, self.b), self.a));
            },
            0x03 => {
                // INC BC
                Cpu::inc_word(&mut self.c, &mut self.b);
            },
            0x04 => {
                // INC B
                self.b = self.inc_byte(self.b);
            },
            0x05 => {
                // DEC B
                self.b = self.dec_byte(self.b);
            },
            0x06 => {
                // LD B, u8
                let value: u8 = self.fetch(memory);
                Cpu::ld_byte(&mut self.b, value);
            },
            0x07 => {
                // RLCA
                self.a = self.rlc(self.a);
                self.zero = false;
            },
            0x08 => {
                // LD (u16), SP
                let lower = self.fetch(memory);
                let upper = self.fetch(memory);
                let address = bit_logic::compose_bytes(lower, upper);
                memory_write_results.append(&mut memory.write_to_memory(address, self.sp as u8));
                memory_write_results.append(&mut memory.write_to_memory(address + 1, (self.sp >> 8) as u8));
            },
            0x09 => {
                // ADD HL, BC
                let (lower, upper) = self.add_word(self.l, self.h, self.c, self.b);
                self.l = lower;
                self.h = upper;
            },
            0x0a => {
                // LD A, (BC)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.c, self.b));
                Cpu::ld_byte(&mut self.a, value);
            },
            0x0b => {
                // DEC BC
                Cpu::dec_word(&mut self.c, &mut self.b);
            },
            0x0c => {
                // INC C
                self.c = self.inc_byte(self.c);
            },
            0x0d => {
                // DEC C
                self.c = self.dec_byte(self.c);
            },
            0x0e => {
                // LD C, u8
                let value: u8 = self.fetch(memory);
                Cpu::ld_byte(&mut self.c, value);
            },
            0x0f => {
                // RRCA
                self.a = self.rrc(self.a);
                self.zero = false;
            },
            0x10 => {
                // STOP
            },
            0x11 => {
                // LD DE, u16
                let lower: u8 = self.fetch(memory);
                let upper: u8 = self.fetch(memory);
                Cpu::ld_word(&mut self.e, &mut self.d, lower, upper);
            },
            0x12 => {
                // LD (DE), A
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.e, self.d), self.a));
            },
            0x13 => {
                // INC DE
                Cpu::inc_word(&mut self.e, &mut self.d);
            },
            0x14 => {
                // INC D
                self.d = self.inc_byte(self.d);
            },
            0x15 => {
                // DEC D
                self.d = self.dec_byte(self.d);
            },
            0x16 => {
                // LD D, u8
                let value: u8 = self.fetch(memory);
                Cpu::ld_byte(&mut self.d, value);
            },
            0x17 => {
                // RLA
                self.a = self.rl(self.a);
                self.zero = false;
            },
            0x18 => {
                // JR i8
                self.jr(memory);
            },
            0x19 => {
                // ADD HL, DE
                let (lower, upper) = self.add_word(self.l, self.h, self.e, self.d);
                self.l = lower;
                self.h = upper;
            },
            0x1a => {
                // LD A, (DE)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.e, self.d));
                Cpu::ld_byte(&mut self.a, value);
            },
            0x1b => {
                // DEC DE
                Cpu::dec_word(&mut self.e, &mut self.d);
            },
            0x1c => {
                // INC E
                self.e = self.inc_byte(self.e);
            },
            0x1d => {
                // DEC E
                self.e = self.dec_byte(self.e);
            },
            0x1e => {
                // LD E, u8
                let value: u8 = self.fetch(memory);
                Cpu::ld_byte(&mut self.e, value);
            },
            0x1f => {
                // RRA
                self.a = self.rr(self.a);
                self.zero = false;
            },
            0x20 => {
                // JR NZ, i8
                if !self.zero {
                    self.jr(memory);
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            },
            0x21 => {
                // LD HL, u16
                let lower: u8 = self.fetch(memory);
                let upper: u8 = self.fetch(memory);
                Cpu::ld_word(&mut self.l, &mut self.h, lower, upper);
            },
            0x22 => {
                // LD (HL+), A
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), self.a));
                Cpu::inc_word(&mut self.l, &mut self.h);
            },
            0x23 => {
                // INC HL
                Cpu::inc_word(&mut self.l, &mut self.h);
            },
            0x24 => {
                // INC H
                self.h = self.inc_byte(self.h);
            },
            0x25 => {
                // DEC H
                self.h = self.dec_byte(self.h);
            },
            0x26 => {
                // LD H, u8
                let value: u8 = self.fetch(memory);
                Cpu::ld_byte(&mut self.h, value);
            },
            0x27 => {
                // DAA
                let mut a: u8 = self.a;
                let mut adjust: u8 = if self.carry { 0x60 } else { 0x00 };

                if self.half_carry { adjust |= 0x06; }

                if !self.subtract {
                    if a & 0x0f > 0x09 { adjust |= 0x06; }
                    if a > 0x99 { adjust |= 0x60; }
                    a = a.wrapping_add(adjust);
                } else {
                    a = a.wrapping_sub(adjust);
                }

                self.zero = a == 0;
                self.half_carry = false;
                self.carry = adjust >= 0x60;
                self.a = a;
            },
            0x28 => {
                // JR Z, i8
                if self.zero {
                    self.jr(memory);
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            },
            0x29 => {
                // ADD HL, HL
                let (lower, upper) = self.add_word(self.l, self.h, self.l, self.h);
                self.l = lower;
                self.h = upper;
            },
            0x2a => {
                // LD A, (HL+)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::ld_byte(&mut self.a, value);
                Cpu::inc_word(&mut self.l, &mut self.h);
            },
            0x2b => {
                // DEC HL
                Cpu::dec_word(&mut self.l, &mut self.h);
            },
            0x2c => {
                // INC L
                self.l = self.inc_byte(self.l);
            },
            0x2d => {
                // DEC L
                self.l = self.dec_byte(self.l);
            },
            0x2e => {
                // LD L, u8
                let value: u8 = self.fetch(memory);
                Cpu::ld_byte(&mut self.l, value);
            },
            0x2f => {
                // CPL
                self.a = !self.a;
                self.subtract = true;
                self.half_carry = true;
            },
            0x30 => {
                // JR NC, i8
                if !self.carry {
                    self.jr(memory);
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            },
            0x31 => {
                // LD SP, u16
                let lower: u8 = self.fetch(memory);
                let upper: u8 = self.fetch(memory);
                self.sp = bit_logic::compose_bytes(lower, upper);
            },
            0x32 => {
                // LD (HL-), A
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), self.a));
                Cpu::dec_word(&mut self.l, &mut self.h);
            },
            0x33 => {
                // INC SP
                self.sp = self.sp.wrapping_add(1);
            },
            0x34 => {
                // INC (HL)
                let address: u16 = bit_logic::compose_bytes(self.l, self.h);
                let mut new_value: u8 = memory.read_from_memory(address);
                new_value = self.inc_byte(new_value);
                memory_write_results.append(&mut memory.write_to_memory(address, new_value));
            },
            0x35 => {
                // DEC (HL)
                let address: u16 = bit_logic::compose_bytes(self.l, self.h);
                let mut new_value: u8 = memory.read_from_memory(address);
                new_value = self.dec_byte(new_value);
                memory_write_results.append(&mut memory.write_to_memory(address, new_value));
            },
            0x36 => {
                // LD (HL), u8
                let value: u8 = self.fetch(memory);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), value));
            },
            0x37 => {
                // SCF
                self.subtract = false;
                self.half_carry = false;
                self.carry = true;
            },
            0x38 => {
                // JR C, i8
                if self.carry {
                    self.jr(memory);
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            },
            0x39 => {
                // ADD HL, SP
                let (sp_lower, sp_upper) = bit_logic::decompose_bytes(self.sp);
                let (lower, upper) = self.add_word(self.l, self.h, sp_lower, sp_upper);
                self.l = lower;
                self.h = upper;
            },
            0x3a => {
                // LD A, (HL-)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::ld_byte(&mut self.a, value);
                Cpu::dec_word(&mut self.l, &mut self.h);
            },
            0x3b => {
                // DEC SP
                self.sp = self.sp.wrapping_sub(1);
            },
            0x3c => {
                // INC A
                self.a = self.inc_byte(self.a);
            },
            0x3d => {
                // DEC A
                self.a = self.dec_byte(self.a);
            },
            0x3e => {
                // LD A, u8
                let value: u8 = self.fetch(memory);
                Cpu::ld_byte(&mut self.a, value);
            },
            0x3f => {
                // CCF
                self.subtract = false;
                self.half_carry = false;
                self.carry = !self.carry;
            },
            0x40 => {
                // LD B, B
                let value: u8 = self.b;
                Cpu::ld_byte(&mut self.b, value);
            },
            0x41 => {
                // LD B, C
                Cpu::ld_byte(&mut self.b, self.c);
            },
            0x42 => {
                // LD B, D
                Cpu::ld_byte(&mut self.b, self.d);
            },
            0x43 => {
                // LD B, E
                Cpu::ld_byte(&mut self.b, self.e);
            },
            0x44 => {
                // LD B, H
                Cpu::ld_byte(&mut self.b, self.h);
            },
            0x45 => {
                // LD B, L
                Cpu::ld_byte(&mut self.b, self.l);
            },
            0x46 => {
                // LD B, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::ld_byte(&mut self.b, value);
            },
            0x47 => {
                // LD B, A
                Cpu::ld_byte(&mut self.b, self.a);
            },
            0x48 => {
                // LD C, B
                Cpu::ld_byte(&mut self.c, self.b);
            },
            0x49 => {
                // LD C, C
                let value: u8 = self.c;
                Cpu::ld_byte(&mut self.c, value);
            },
            0x4a => {
                // LD C, D
                Cpu::ld_byte(&mut self.c, self.d);
            },
            0x4b => {
                // LD C, E
                Cpu::ld_byte(&mut self.c, self.e);
            },
            0x4c => {
                // LD C, H
                Cpu::ld_byte(&mut self.c, self.h);
            },
            0x4d => {
                // LD C, L
                Cpu::ld_byte(&mut self.c, self.l);
            },
            0x4e => {
                // LD C, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::ld_byte(&mut self.c, value);
            },
            0x4f => {
                // LD C, A
                Cpu::ld_byte(&mut self.c, self.a);                
            },
            0x50 => {
                // LD D, B
                Cpu::ld_byte(&mut self.d, self.b);
            },
            0x51 => {
                // LD D, C
                Cpu::ld_byte(&mut self.d, self.c);
            },
            0x52 => {
                // LD D, D
                let value: u8 = self.d;
                Cpu::ld_byte(&mut self.d, value);
            },
            0x53 => {
                // LD D, E
                Cpu::ld_byte(&mut self.d, self.e);
            },
            0x54 => {
                // LD D, H
                Cpu::ld_byte(&mut self.d, self.h);
            },
            0x55 => {
                // LD D, L
                Cpu::ld_byte(&mut self.d, self.l);
            },
            0x56 => {
                // LD D, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::ld_byte(&mut self.d, value);
            },
            0x57 => {
                // LD D, A
                Cpu::ld_byte(&mut self.d, self.a);
            },
            0x58 => {
                // LD E, B
                Cpu::ld_byte(&mut self.e, self.b);
            },
            0x59 => {
                // LD E, C
                Cpu::ld_byte(&mut self.e, self.c);
            },
            0x5a => {
                // LD E, D
                Cpu::ld_byte(&mut self.e, self.d);
            },
            0x5b => {
                // LD E, E
                let value: u8 = self.e;
                Cpu::ld_byte(&mut self.e, value);
            },
            0x5c => {
                // LD E, H
                Cpu::ld_byte(&mut self.e, self.h);
            },
            0x5d => {
                // LD E, L
                Cpu::ld_byte(&mut self.e, self.l);
            },
            0x5e => {
                // LD E, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::ld_byte(&mut self.e, value);
            },
            0x5f => {
                // LD E, A
                Cpu::ld_byte(&mut self.e, self.a);
            },
            0x60 => {
                // LD H, B
                Cpu::ld_byte(&mut self.h, self.b);
            },
            0x61 => {
                // LD H, C
                Cpu::ld_byte(&mut self.h, self.c);
            },
            0x62 => {
                // LD H, D
                Cpu::ld_byte(&mut self.h, self.d);
            },
            0x63 => {
                // LD H, E
                Cpu::ld_byte(&mut self.h, self.e);
            },
            0x64 => {
                // LD H, H
                let value: u8 = self.h;
                Cpu::ld_byte(&mut self.h, value);
            },
            0x65 => {
                // LD H, L
                Cpu::ld_byte(&mut self.h, self.l);
            },
            0x66 => {
                // LD H, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::ld_byte(&mut self.h, value);
            },
            0x67 => {
                // LD H, A
                Cpu::ld_byte(&mut self.h, self.a);
            },
            0x68 => {
                // LD L, B
                Cpu::ld_byte(&mut self.l, self.b);
            },
            0x69 => {
                // LD L, C
                Cpu::ld_byte(&mut self.l, self.c);
            },
            0x6a => {
                // LD L, D
                Cpu::ld_byte(&mut self.l, self.d);
            },
            0x6b => {
                // LD L, E
                Cpu::ld_byte(&mut self.l, self.e);
            },
            0x6c => {
                // LD L, H
                Cpu::ld_byte(&mut self.l, self.h);
            },
            0x6d => {
                // LD L, L
                let value: u8 = self.l;
                Cpu::ld_byte(&mut self.l, value);
            },
            0x6e => {
                // LD L, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::ld_byte(&mut self.l, value);
            },
            0x6f => {
                // LD L, A
                Cpu::ld_byte(&mut self.l, self.a);
            },
            0x70 => {
                // LD (HL), B
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), self.b));
            },
            0x71 => {
                // LD (HL), C
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), self.c));
            },
            0x72 => {
                // LD (HL), D
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), self.d));
            },
            0x73 => {
                // LD (HL), E
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), self.e));
            },
            0x74 => {
                // LD (HL), H
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), self.h));
            },
            0x75 => {
                // LD (HL), L
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), self.l));
            },
            0x76 => {
                // HALT
                self.halted = true;
            },
            0x77 => {
                // LD (HL), A
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(self.l, self.h), self.a));
            },
            0x78 => {
                // LD A, B
                Cpu::ld_byte(&mut self.a, self.b);
            },
            0x79 => {
                // LD A, C
                Cpu::ld_byte(&mut self.a, self.c);
            },
            0x7a => {
                // LD A, D
                Cpu::ld_byte(&mut self.a, self.d);
            },
            0x7b => {
                // LD A, E
                Cpu::ld_byte(&mut self.a, self.e);
            },
            0x7c => {
                // LD A, H
                Cpu::ld_byte(&mut self.a, self.h);
            },
            0x7d => {
                // LD A, L
                Cpu::ld_byte(&mut self.a, self.l);
            },
            0x7e => {
                // LD A, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                Cpu::ld_byte(&mut self.a, value);
            },
            0x7f => {
                // LD A, A
                let value: u8 = self.a;
                Cpu::ld_byte(&mut self.a, value);
            },
            0x80 => {
                // ADD A, B
                self.a = self.add_byte(self.a, self.b);
            },
            0x81 => {
                // ADD A, C
                self.a = self.add_byte(self.a, self.c);
            },
            0x82 => {
                // ADD A, D
                self.a = self.add_byte(self.a, self.d);
            },
            0x83 => {
                // ADD A, E
                self.a = self.add_byte(self.a, self.e);
            },
            0x84 => {
                // ADD A, H
                self.a = self.add_byte(self.a, self.h);
            },
            0x85 => {
                // ADD A, L
                self.a = self.add_byte(self.a, self.l);
            },
            0x86 => {
                // ADD A, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.a = self.add_byte(self.a, value);
            },
            0x87 => {
                // ADD A, A
                self.a = self.add_byte(self.a, self.a);
            },
            0x88 => {
                // ADC A, B
                self.a = self.adc_byte(self.a, self.b);
            },
            0x89 => {
                // ADC A, C
                self.a = self.adc_byte(self.a, self.c);
            },
            0x8a => {
                // ADC A, D
                self.a = self.adc_byte(self.a, self.d);
            },
            0x8b => {
                // ADC A, E
                self.a = self.adc_byte(self.a, self.e);
            },
            0x8c => {
                // ADC A, H
                self.a = self.adc_byte(self.a, self.h);
            },
            0x8d => {
                // ADC A, L
                self.a = self.adc_byte(self.a, self.l);
            },
            0x8e => {
                // ADC A, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.a = self.adc_byte(self.a, value);
            },
            0x8f => {
                // ADC A, A
                self.a = self.adc_byte(self.a, self.a);
            },
            0x90 => {
                // SUB A, B
                self.a = self.sub_byte(self.a, self.b);
            },
            0x91 => {
                // SUB A, C
                self.a = self.sub_byte(self.a, self.c);
            },
            0x92 => {
                // SUB A, D
                self.a = self.sub_byte(self.a, self.d);
            },
            0x93 => {
                // SUB A, E
                self.a = self.sub_byte(self.a, self.e);
            },
            0x94 => {
                // SUB A, H
                self.a = self.sub_byte(self.a, self.h);
            },
            0x95 => {
                // SUB A, L
                self.a = self.sub_byte(self.a, self.l);                
            },
            0x96 => {
                // SUB A, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.a = self.sub_byte(self.a, value);
            },
            0x97 => {
                // SUB A, A
                self.a = self.sub_byte(self.a, self.a);
            },
            0x98 => {
                // SBC A, B
                self.a = self.sbc_byte(self.a, self.b);
            },
            0x99 => {
                // SBC A, C
                self.a = self.sbc_byte(self.a, self.c);
            },
            0x9a => {
                // SBC A, D
                self.a = self.sbc_byte(self.a, self.d);
            },
            0x9b => {
                // SBC A, E
                self.a = self.sbc_byte(self.a, self.e);
            },
            0x9c => {
                // SBC A, H
                self.a = self.sbc_byte(self.a, self.h);
            },
            0x9d => {
                // SBC A, L
                self.a = self.sbc_byte(self.a, self.l);
            },
            0x9e => {
                // SBC A, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.a = self.sbc_byte(self.a, value);
            },
            0x9f => {
                // SBC A, A
                self.a = self.sbc_byte(self.a, self.a);
            },
            0xa0 => {
                // AND A, B
                self.a = self.and_byte(self.a, self.b);
            },
            0xa1 => {
                // AND A, C
                self.a = self.and_byte(self.a, self.c);
            },
            0xa2 => {
                // AND A, D
                self.a = self.and_byte(self.a, self.d);
            },
            0xa3 => {
                // AND A, E
                self.a = self.and_byte(self.a, self.e);
            },
            0xa4 => {
                // AND A, H
                self.a = self.and_byte(self.a, self.h);
            },
            0xa5 => {
                // AND A, L
                self.a = self.and_byte(self.a, self.l);
            },
            0xa6 => {
                // AND A, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.a = self.and_byte(self.a, value);
            },
            0xa7 => {
                // AND A, A
                self.a = self.and_byte(self.a, self.a);
            },
            0xa8 => {
                // XOR A, B
                self.a = self.xor_byte(self.a, self.b);
            },
            0xa9 => {
                // XOR A, C
                self.a = self.xor_byte(self.a, self.c);
            },
            0xaa => {
                // XOR A, D
                self.a = self.xor_byte(self.a, self.d);
            },
            0xab => {
                // XOR A, E
                self.a = self.xor_byte(self.a, self.e);
            },
            0xac => {
                // XOR A, H
                self.a = self.xor_byte(self.a, self.h);
            },
            0xad => {
                // XOR A, L
                self.a = self.xor_byte(self.a, self.l);
            },
            0xae => {
                // XOR A, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.a = self.xor_byte(self.a, value);
            },
            0xaf => {
                // XOR A, A
                self.a = self.xor_byte(self.a, self.a);
            },
            0xb0 => {
                // OR A, B
                self.a = self.or_byte(self.a, self.b);
            },
            0xb1 => {
                // OR A, C
                self.a = self.or_byte(self.a, self.c);
            },
            0xb2 => {
                // OR A, D
                self.a = self.or_byte(self.a, self.d);
            },
            0xb3 => {
                // OR A, E
                self.a = self.or_byte(self.a, self.e);
            },
            0xb4 => {
                // OR A, H
                self.a = self.or_byte(self.a, self.h);
            },
            0xb5 => {
                // OR A, L
                self.a = self.or_byte(self.a, self.l);
            },
            0xb6 => {
                // OR A, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.a = self.or_byte(self.a, value);
            },
            0xb7 => {
                // OR A, A
                self.a = self.or_byte(self.a, self.a);
            },
            0xb8 => {
                // CP A, B
                self.cp_byte(self.a, self.b);
            },
            0xb9 => {
                // CP A, C
                self.cp_byte(self.a, self.c);
            },
            0xba => {
                // CP A, D
                self.cp_byte(self.a, self.d);
            },
            0xbb => {
                // CP A, E
                self.cp_byte(self.a, self.e);
            },
            0xbc => {
                // CP A, H
                self.cp_byte(self.a, self.h);
            },
            0xbd => {
                // CP A, L
                self.cp_byte(self.a, self.l);
            },
            0xbe => {
                // CP A, (HL)
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(self.l, self.h));
                self.cp_byte(self.a, value);
            },
            0xbf => {
                // CP A, A
                self.cp_byte(self.a, self.a);
            },
            0xc0 => {
                // RET NZ
                if !self.zero {
                    self.ret(memory);
                    branch_taken = true;
                }
            },
            0xc1 => {
                // POP BC
                self.c = self.pop(memory);
                self.b = self.pop(memory);
            },
            0xc2 => {
                // JP NZ, u16
                if !self.zero {
                    self.jp_from_pc(memory);
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(2);
                }
            },
            0xc3 => {
                // JP u16
                self.jp_from_pc(memory);
            },
            0xc4 => {
                // CALL NZ, u16
                if !self.zero {
                    memory_write_results.append(&mut self.call(memory));
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(2);
                }
            },
            0xc5 => {
                // PUSH BC
                self.push(memory, self.b);
                self.push(memory, self.c);
            },
            0xc6 => {
                // ADD A, u8
                let value: u8 = self.fetch(memory);
                self.a = self.add_byte(self.a, value);
            },
            0xc7 => {
                // RST 00h
                memory_write_results.append(&mut self.rst(memory, 0x0));
            },
            0xc8 => {
                // RET Z
                if self.zero {
                    self.ret(memory);
                    branch_taken = true;
                }
            },
            0xc9 => {
                // RET
                self.ret(memory);
            },
            0xca => {
                // JP Z, u16
                if self.zero {
                    self.jp_from_pc(memory);
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(2);
                }
            },
            0xcb => {
                // Prefix CB
                let cb_instruction: u8 = self.fetch(memory);
                return self.execute_cb(memory, cb_instruction);
            },
            0xcc => {
                // CALL Z, u16
                if self.zero {
                    memory_write_results.append(&mut self.call(memory));
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(2);
                }
            },
            0xcd => {
                // CALL u16
                memory_write_results.append(&mut self.call(memory));
            },
            0xce => {
                // ADC A, u8
                let value: u8 = self.fetch(memory);
                self.a = self.adc_byte(self.a, value);
            },
            0xcf => {
                // RST 08h
                memory_write_results.append(&mut self.rst(memory, 0x8));
            },
            0xd0 => {
                // RET NC
                if !self.carry {
                    self.ret(memory);
                    branch_taken = true;
                }
            },
            0xd1 => {
                // POP DE
                self.e = self.pop(memory);
                self.d = self.pop(memory);
            },
            0xd2 => {
                // JP NC, u16
                if !self.carry {
                    self.jp_from_pc(memory);
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(2);
                }
            },
            0xd3 => {
                // Blank Instruction
            },
            0xd4 => {
                // CALL NC, u16
                if !self.carry {
                    memory_write_results.append(&mut self.call(memory));
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(2);
                }
            },
            0xd5 => {
                // PUSH DE
                self.push(memory, self.d);
                self.push(memory, self.e);
            },
            0xd6 => {
                // SUB A, u8
                let value: u8 = self.fetch(memory);
                self.a = self.sub_byte(self.a, value);
            },
            0xd7 => {
                // RST 10h
                memory_write_results.append(&mut self.rst(memory, 0x10));
            },
            0xd8 => {
                // RET C
                if self.carry {
                    self.ret(memory);
                    branch_taken = true;
                }
            },
            0xd9 => {
                // RETI
                self.ret(memory);
                self.interrupts_enabled = true;
            },
            0xda => {
                // JP C, u16
                if self.carry {
                    self.jp_from_pc(memory);
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(2);
                }
            },
            0xdb => {
                // Blank Instruction
            },
            0xdc => {
                // CALL C, u16
                if self.carry {
                    memory_write_results.append(&mut self.call(memory));
                    branch_taken = true;
                } else {
                    self.pc = self.pc.wrapping_add(2);
                }
            },
            0xdd => {
                // Blank Instruction
            },
            0xde => {
                // SBC A, u8
                let value: u8 = self.fetch(memory);
                self.a = self.sbc_byte(self.a, value);
            },
            0xdf => {
                // RST 18h
                memory_write_results.append(&mut self.rst(memory, 0x18));
            },
            0xe0 => {
                // LD (FF00 + u8), A
                let value: u8 = self.fetch(memory);
                memory_write_results.append(&mut memory.write_to_memory(0xff00 + (value as u16), self.a));
            },
            0xe1 => {
                // POP HL
                self.l = self.pop(memory);
                self.h = self.pop(memory);
            },
            0xe2 => {
                // LD (FF00 + C), A
                memory_write_results.append(&mut memory.write_to_memory(0xff00 + (self.c as u16), self.a));
            },
            0xe3 | 0xe4 => {
                // Blank Instruction
            },
            0xe5 => {
                // PUSH HL
                self.push(memory, self.h);
                self.push(memory, self.l);
            },
            0xe6 => {
                // AND A, u8
                let value: u8 = self.fetch(memory);
                self.a = self.and_byte(self.a, value);
            },
            0xe7 => {
                // RST 20h
                memory_write_results.append(&mut self.rst(memory, 0x20));
            },
            0xe8 => {
                // ADD SP, i8
                let sp: u16 = self.sp;
                //let value: u16 = ((self.fetch(memory) as i8) as i16) as u16;
                let value: u16 = (self.fetch(memory) as i8) as u16;
                self.sp = sp.wrapping_add(value);
                self.zero = false;
                self.subtract = false;
                self.half_carry = (sp & 0xf) + (value & 0xf) > 0xf;
                self.carry = (sp & 0xff) + (value & 0xff) > 0xff;
            },
            0xe9 => {
                // JP HL
                self.jp_from_bytes(self.l, self.h);
            },
            0xea => {
                // LD (u16), A
                let lower: u8 = self.fetch(memory);
                let upper: u8 = self.fetch(memory);
                memory_write_results.append(&mut memory.write_to_memory(bit_logic::compose_bytes(lower, upper), self.a));
            },
            0xeb | 0xec | 0xed => {
                // Blank Instruction
            },
            0xee => {
                // XOR A, u8
                let value: u8 = self.fetch(memory);
                self.a = self.xor_byte(self.a, value);
            },
            0xef => {
                // RST 28h
                memory_write_results.append(&mut self.rst(memory, 0x28));
            },
            0xf0 => {
                // LD A, (FF00 + u8)
                let offset: u8 = self.fetch(memory);
                let value: u8 = memory.read_from_memory(0xff00 + (offset as u16));
                Cpu::ld_byte(&mut self.a, value);
            },
            0xf1 => {
                // POP AF
                let popped_f: u8 = self.pop(memory);
                self.set_f(popped_f);
                self.a = self.pop(memory);
            },
            0xf2 => {
                // LD A, (FF00 + C)
                let value: u8 = memory.read_from_memory(0xff00 + (self.c as u16));
                Cpu::ld_byte(&mut self.a, value);
            },
            0xf3 => {
                // DI
                self.pending_interrupt_enable = false;
                self.one_instruction_passed = false;
                self.interrupts_enabled = false;
            },
            0xf4 => {
                // Blank Instruction
            },
            0xf5 => {
                // PUSH AF
                self.push(memory, self.a);
                self.push(memory, self.get_f());
            },
            0xf6 => {
                // OR A, u8
                let value: u8 = self.fetch(memory);
                self.a = self.or_byte(self.a, value);
            },
            0xf7 => {
                // RST 30h
                memory_write_results.append(&mut self.rst(memory, 0x30));
            },
            0xf8 => {
                // LD HL, SP + i8
                //let value: u16 = ((self.fetch(memory) as i8) as i16) as u16;
                let value: u16 = (self.fetch(memory) as i8) as u16;
                let (lower, upper) = bit_logic::decompose_bytes(self.sp.wrapping_add(value));
                self.l = lower;
                self.h = upper;
                self.zero = false;
                self.subtract = false;
                self.half_carry = (self.sp & 0xf) + (value & 0xf) > 0xf;
                self.carry = (self.sp & 0xff) + (value & 0xff) > 0xff;
            },
            0xf9 => {
                // LD SP, HL
                self.sp = bit_logic::compose_bytes(self.l, self.h);
            },
            0xfa => {
                // LD A, (u16)
                let lower: u8 = self.fetch(memory);
                let upper: u8 = self.fetch(memory);
                let value: u8 = memory.read_from_memory(bit_logic::compose_bytes(lower, upper));
                Cpu::ld_byte(&mut self.a, value);
            },
            0xfb => {
                // EI
                self.pending_interrupt_enable = true;
            },
            0xfc | 0xfd => {
                // Blank Instruction
            },
            0xfe => {
                // CP A, u8
                let value: u8 = self.fetch(memory);
                self.cp_byte(self.a, value);
            },
            0xff => {
                // RST 38h
                memory_write_results.append(&mut self.rst(memory, 0x38));
            },
        }
        if branch_taken {
            (BRANCH_INSTRUCTION_TIMINGS[usize::from(instruction)], memory_write_results)
        } else {
            (INSTRUCTION_TIMINGS[usize::from(instruction)], memory_write_results)
        }
    }
}