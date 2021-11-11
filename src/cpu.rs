use crate::memory::Memory;
use crate::bit_logic;

use std::sync::{Arc, Mutex};

static INSTRUCTION_TIMINGS: [u8; 256] = [
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
static BRANCH_INSTRUCTION_TIMINGS: [u8; 256] = [
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
static CB_INSTRUCTION_TIMINGS: [u8; 256] = [
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

pub struct CPU {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
    halted: bool,
    interrupts_enabled: bool,
    pending_interrupt_enable: bool,
    memory: Arc<Mutex<Memory>>,
}

impl CPU {
    pub fn new(memory: Arc<Mutex<Memory>>) -> CPU {
        CPU {
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
            memory: memory,
        }
    }

    pub fn is_halted(&self) -> bool { self.halted }

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

    fn fetch(&mut self) -> u8 {
        let memory = self.memory.lock().unwrap();
        let value = memory.read_from_memory(self.pc);
        self.pc = u16::wrapping_add(self.pc, 1);
        value
    }

    fn read_from_address(&self, address: u16) -> u8 {
        let memory = self.memory.lock().unwrap();
        memory.read_from_memory(address)
    }

    fn write_to_address(&mut self, address: u16, value: u8) {
        let mut memory = self.memory.lock().unwrap();
        memory.write_to_memory(address, value);
    }

    fn pop(&mut self) -> u8 {
        let memory = self.memory.lock().unwrap();
        let value: u8 = memory.read_from_memory(self.sp);
        self.sp = u16::wrapping_add(self.sp, 1);
        value
    }
    
    fn push(&mut self, value: u8) {
        let mut memory = self.memory.lock().unwrap();
        self.sp = u16::wrapping_sub(self.sp, 1);
        memory.write_to_memory(self.sp, value);
    }

    fn rlc(&mut self, value: u8) -> u8 {
        let carry: bool = bit_logic::check_bit(value, 7);
        let truncated: u8 = bit_logic::bit_value(value, 7);
        let result: u8 = (value << 1) | truncated;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = carry;
        result
    }
    
    fn rrc(&mut self, value: u8) -> u8 {
        let carry: bool = bit_logic::check_bit(value, 0);
        let truncated: u8 = bit_logic::bit_value(value, 0);
        let result: u8 = (value >> 1) | (truncated << 7);
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = carry;
        result
    }
    
    fn rl(&mut self, value: u8) -> u8 {
        let carry: bool = self.carry;
        let will_carry: bool = bit_logic::check_bit(value, 7);
        let mut result: u8 = value << 1;
        result |= carry as u8;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = will_carry;
        result
    }
    
    fn rr(&mut self, value: u8) -> u8 {
        let carry: bool = self.carry;
        let will_carry: bool = bit_logic::check_bit(value, 0);
        let mut result: u8 = value >> 1;
        result |= (carry as u8) << 7;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = will_carry;
        result
    }
    
    fn sla(&mut self, value: u8) -> u8 {
        let carry: bool = bit_logic::check_bit(value, 7);
        let result: u8 = value << 1;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = carry;
        result
    }
    
    fn sra(&mut self, value: u8) -> u8 {
        let carry: bool = bit_logic::check_bit(value, 0);
        let top: bool = bit_logic::check_bit(value, 7);
        let mut result: u8 = value >> 1;
        result = bit_logic::set_bit_to(top, result, 7);
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = carry;
        result
    }
    
    fn srl(&mut self, value: u8) -> u8 {
        let least_bit_set: bool = bit_logic::check_bit(value, 0);
        let result: u8 = value >> 1;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = least_bit_set;
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
        word += 1;
        *upper = (word >> 8) as u8;
        *lower = word as u8;
    }

    fn dec_word(lower: &mut u8, upper: &mut u8) {
        let mut word: u16 = bit_logic::compose_bytes(*lower, *upper);
        word -= 1;
        *upper = (word >> 8) as u8;
        *lower = word as u8;
    }

    fn inc_byte(&mut self, value: u8) -> u8 {
        let result = value + 1;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = (result & 0xf) == 0;
        result
    }

    fn dec_byte(&mut self, value: u8) -> u8 {
        let result = value - 1;
        self.zero = result == 0;
        self.subtract = true;
        self.half_carry = (result & 0xf) == 0xf;
        result
    }

    fn add_word(&mut self, lower_value: u8, upper_value: u8, lower_addend: u8, upper_addend: u8) -> (u8, u8) {
        let value_word: u16 = bit_logic::compose_bytes(lower_value, upper_value);
        let addend_word: u16 = bit_logic::compose_bytes(lower_addend, upper_addend);
        let result: u32 = (value_word as u32) + (addend_word as u32);
        self.subtract = false;
        self.half_carry = (value_word & 0xfff) + (addend_word & 0xfff) > 0xfff;
        self.carry = (result & 0x10000) != 0;
        (result as u8, (result >> 8) as u8)
    }

    fn add_byte(&mut self, value: u8, addend: u8) -> u8 {
        let first: u8 = value;
        let second: u8 = addend;
        let result: u16 = (first as u16) + (second as u16);
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = (first & 0xf) + (second & 0xf) > 0xf;
        self.carry = (result & 0x100) != 0;
        result as u8
    }

    fn adc_byte(&mut self, value: u8, addend: u8) -> u8 {
        let first: u8 = value;
        let second: u8 = addend;
        let carry: u8 = self.carry as u8;
        let result: u16 = (first as u16) + (second as u16) + (carry as u16);
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = ((first & 0xf) + (second & 0xf) + carry) > 0xf;
        self.carry = result > 0xff;
        result as u8
    }

    fn sub_byte(&mut self, value: u8, subtrahend: u8) -> u8 {
        let first: u8 = value;
        let second: u8 = subtrahend;
        let result: u8 = first - second;
        self.zero = result == 0;
        self.subtract = true;
        self.half_carry = ((first & 0xf) - (second & 0xf)) < 0;
        self.carry = first < second;
        result
    }

    fn sbc_byte(&mut self, value: u8, subtrahend: u8) -> u8 {
        let first: u8 = value;
        let second: u8 = subtrahend;
        let carry: u8 = self.carry as u8;
        let result: i16 = (first as i16) + (second as i16) + (carry as i16);
        self.zero = result == 0;
        self.subtract = true;
        self.half_carry = ((first & 0xf) - (second & 0xf) - carry) < 0;
        self.carry = result < 0;
        result as u8
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
        let result: u8 = first - second;
        self.zero = result == 0;
        self.subtract = true;
        self.half_carry = ((first & 0xf) - (second & 0xf)) < 0;
        self.carry = first < second;
    }

    fn ret(&mut self) {
        let lower: u8 = self.pop();
        let upper: u8 = self.pop();
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

    fn jp_from_pc(&mut self) {
        let lower: u8 = self.fetch();
        let upper: u8 = self.fetch();
        self.jp_from_bytes(lower, upper);
    }

    fn call(&mut self) {
        let lower_new: u8 = self.fetch();
        let upper_new: u8 = self.fetch();
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        self.jp_from_bytes(lower_new, upper_new);
    }

    fn rst(&mut self, value: u8) {
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        self.jp_from_word((0x0 + value) as u16);
    }

    fn jr(&mut self) {
        let value: i32 = self.fetch() as i32;
        self.jp_from_word((self.pc as i32 + value) as u16);
    }

    pub fn update(&mut self) -> f64 {
        let instruction = self.fetch();
        self.execute(instruction)
    }

    fn execute_cb(&mut self, instruction: u8) -> f64 {
        CB_INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
    }

    fn execute(&mut self, instruction: u8) -> f64 {
        let mut branch_taken: bool = false;
        match instruction {
            0x00 => {
                // NOP
            },
            0x01 => {
                // LD BC, u16
                let lower: u8 = self.fetch();
                let upper: u8 = self.fetch();
                CPU::ld_word(&mut self.c, &mut self.b, lower, upper);
            },
            0x02 => {
                // LD (BC), A
                self.write_to_address(bit_logic::compose_bytes(self.c, self.b), self.a);
            },
            0x03 => {
                // INC BC
                CPU::inc_word(&mut self.c, &mut self.b);
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
                let value = self.fetch();
                CPU::ld_byte(&mut self.b, value);
            },
            0x07 => {
                // RLCA
                self.a = self.rlc(self.a);
                self.zero = false;
            },
            0x08 => {
                // LD (u16), SP
                let lower = self.fetch();
                let upper = self.fetch();
                let address = bit_logic::compose_bytes(lower, upper);
                self.write_to_address(address, self.sp as u8);
                self.write_to_address(address + 1, (self.sp >> 8) as u8);
            },
            0x09 => {
                // ADD HL, BC
                let (lower, upper) = self.add_word(self.l, self.h, self.c, self.b);
                self.l = lower;
                self.h = upper;
            },
            0x0a => {
                // LD A, (BC)
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.c, self.b));
                CPU::ld_byte(&mut self.a, value);
            },
            0x0b => {
                // DEC BC
                CPU::dec_word(&mut self.c, &mut self.b);
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
                let value = self.fetch();
                CPU::ld_byte(&mut self.c, value);
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
                let lower: u8 = self.fetch();
                let upper: u8 = self.fetch();
                CPU::ld_word(&mut self.e, &mut self.d, lower, upper);
            },
            0x12 => {
                // LD (DE), A
                self.write_to_address(bit_logic::compose_bytes(self.e, self.d), self.a);
            },
            0x13 => {
                // INC DE
                CPU::inc_word(&mut self.e, &mut self.d);
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
                let value: u8 = self.fetch();
                CPU::ld_byte(&mut self.d, value);
            },
            0x17 => {
                // RLA
                self.a = self.rl(self.a);
                self.zero = false;
            },
            0x18 => {
                // JR i8
                self.jr();
            },
            0x19 => {
                // ADD HL, DE
                let (lower, upper) = self.add_word(self.l, self.h, self.e, self.d);
                self.l = lower;
                self.h = upper;
            },
            0x1a => {
                // LD A, (DE)
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.e, self.d));
                CPU::ld_byte(&mut self.a, value);
            },
            0x1b => {
                // DEC DE
                CPU::dec_word(&mut self.e, &mut self.d);
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
                let value: u8 = self.fetch();
                CPU::ld_byte(&mut self.e, value);
            },
            0x1f => {
                // RRA
                self.a = self.rr(self.a);
                self.zero = false;
            },
            0x20 => {
                // JR NZ, i8
                if !self.zero {
                    self.jr();
                    branch_taken = true;
                } else {
                    self.pc += 1;
                }
            },
            0x21 => {
                // LD HL, u16
                let lower: u8 = self.fetch();
                let upper: u8 = self.fetch();
                CPU::ld_word(&mut self.l, &mut self.h, lower, upper);
            },
            0x22 => {
                // LD (HL+), A
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), self.a);
                CPU::inc_word(&mut self.l, &mut self.h);
            },
            0x23 => {
                // INC HL
                CPU::inc_word(&mut self.l, &mut self.h);
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
                let value: u8 = self.fetch();
                CPU::ld_byte(&mut self.h, value);
            },
            0x27 => {
                // DAA
                let mut correction: u16 = if self.carry { 0x60 } else { 0x00 };

                if self.half_carry || (!self.subtract && ((self.a & 0x0f) > 9)) {
                    correction |= 0x06;
                }
                if self.carry || (!self.subtract && (self.a > 0x99)) {
                    correction |= 0x60;
                }

                self.a = if self.subtract {
                    (self.a as u16 - correction) as u8
                } else {
                    (self.a as u16 + correction) as u8
                };

                if ((correction << 2) & 0x100) != 0 {
                    self.carry = true;
                }
                self.zero = self.a == 0;
                self.half_carry = false;
            },
            0x28 => {
                // JR Z, i8
                if self.zero {
                    self.jr();
                    branch_taken = true;
                } else {
                    self.pc += 1;
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::ld_byte(&mut self.a, value);
                CPU::inc_word(&mut self.l, &mut self.h);
            },
            0x2b => {
                // DEC HL
                CPU::dec_word(&mut self.l, &mut self.h);
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
                let value: u8 = self.fetch();
                CPU::ld_byte(&mut self.l, value);
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
                    self.jr();
                    branch_taken = true;
                } else {
                    self.pc += 1;
                }
            },
            0x31 => {
                // LD SP, u16
                let lower: u8 = self.fetch();
                let upper: u8 = self.fetch();
                self.sp = bit_logic::compose_bytes(lower, upper);
            },
            0x32 => {
                // LD (HL-), A
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), self.a);
                CPU::dec_word(&mut self.l, &mut self.h);
            },
            0x33 => {
                // INC SP
                self.sp += 1;
            },
            0x34 => {
                // INC (HL)
                let address: u16 = bit_logic::compose_bytes(self.l, self.h);
                let new_value: u8 = self.read_from_address(address);
                let new_value: u8 = self.inc_byte(new_value);
                self.write_to_address(address, new_value);
            },
            0x35 => {
                // DEC (HL)
                let address: u16 = bit_logic::compose_bytes(self.l, self.h);
                let new_value: u8 = self.read_from_address(address);
                let new_value: u8 = self.dec_byte(new_value);
                self.write_to_address(address, new_value);
            },
            0x36 => {
                // LD (HL), u8
                let value: u8 = self.fetch();
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
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
                    self.jr();
                    branch_taken = true;
                } else {
                    self.pc += 1;
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::ld_byte(&mut self.a, value);
                CPU::dec_word(&mut self.l, &mut self.h);
            },
            0x3b => {
                // DEC SP
                self.sp -= 1;
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
                let value: u8 = self.fetch();
                CPU::ld_byte(&mut self.a, value);
            },
            0x3f => {
                // CCF
                self.subtract = false;
                self.half_carry = false;
                self.carry = !self.carry;
            },
            0x40 => {
                // LD B, B
                let value = self.b;
                CPU::ld_byte(&mut self.b, value);
            },
            0x41 => {
                // LD B, C
                CPU::ld_byte(&mut self.b, self.c);
            },
            0x42 => {
                // LD B, D
                CPU::ld_byte(&mut self.b, self.d);
            },
            0x43 => {
                // LD B, E
                CPU::ld_byte(&mut self.b, self.e);
            },
            0x44 => {
                // LD B, H
                CPU::ld_byte(&mut self.b, self.h);
            },
            0x45 => {
                // LD B, L
                CPU::ld_byte(&mut self.b, self.l);
            },
            0x46 => {
                // LD B, (HL)
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::ld_byte(&mut self.b, value);
            },
            0x47 => {
                // LD B, A
                CPU::ld_byte(&mut self.b, self.a);
            },
            0x48 => {
                // LD C, B
                CPU::ld_byte(&mut self.c, self.b);
            },
            0x49 => {
                // LD C, C
                let value = self.c;
                CPU::ld_byte(&mut self.c, value);
            },
            0x4a => {
                // LD C, D
                CPU::ld_byte(&mut self.c, self.d);
            },
            0x4b => {
                // LD C, E
                CPU::ld_byte(&mut self.c, self.e);
            },
            0x4c => {
                // LD C, H
                CPU::ld_byte(&mut self.c, self.h);
            },
            0x4d => {
                // LD C, L
                CPU::ld_byte(&mut self.c, self.l);
            },
            0x4e => {
                // LD C, (HL)
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::ld_byte(&mut self.c, value);
            },
            0x4f => {
                // LD C, A
                CPU::ld_byte(&mut self.c, self.a);                
            },
            0x50 => {
                // LD D, B
                CPU::ld_byte(&mut self.d, self.b);
            },
            0x51 => {
                // LD D, C
                CPU::ld_byte(&mut self.d, self.c);
            },
            0x52 => {
                // LD D, D
                let value = self.d;
                CPU::ld_byte(&mut self.d, value);
            },
            0x53 => {
                // LD D, E
                CPU::ld_byte(&mut self.d, self.e);
            },
            0x54 => {
                // LD D, H
                CPU::ld_byte(&mut self.d, self.h);
            },
            0x55 => {
                // LD D, L
                CPU::ld_byte(&mut self.d, self.l);
            },
            0x56 => {
                // LD D, (HL)
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::ld_byte(&mut self.d, value);
            },
            0x57 => {
                // LD D, A
                CPU::ld_byte(&mut self.d, self.a);
            },
            0x58 => {
                // LD E, B
                CPU::ld_byte(&mut self.e, self.b);
            },
            0x59 => {
                // LD E, C
                CPU::ld_byte(&mut self.e, self.c);
            },
            0x5a => {
                // LD E, D
                CPU::ld_byte(&mut self.e, self.d);
            },
            0x5b => {
                // LD E, E
                let value = self.e;
                CPU::ld_byte(&mut self.e, value);
            },
            0x5c => {
                // LD E, H
                CPU::ld_byte(&mut self.e, self.h);
            },
            0x5d => {
                // LD E, L
                CPU::ld_byte(&mut self.e, self.l);
            },
            0x5e => {
                // LD E, (HL)
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::ld_byte(&mut self.e, value);
            },
            0x5f => {
                // LD E, A
                CPU::ld_byte(&mut self.e, self.a);
            },
            0x60 => {
                // LD H, B
                CPU::ld_byte(&mut self.h, self.b);
            },
            0x61 => {
                // LD H, C
                CPU::ld_byte(&mut self.h, self.c);
            },
            0x62 => {
                // LD H, D
                CPU::ld_byte(&mut self.h, self.d);
            },
            0x63 => {
                // LD H, E
                CPU::ld_byte(&mut self.h, self.e);
            },
            0x64 => {
                // LD H, H
                let value = self.h;
                CPU::ld_byte(&mut self.h, value);
            },
            0x65 => {
                // LD H, L
                CPU::ld_byte(&mut self.h, self.l);
            },
            0x66 => {
                // LD H, (HL)
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::ld_byte(&mut self.h, value);
            },
            0x67 => {
                // LD H, A
                CPU::ld_byte(&mut self.h, self.a);
            },
            0x68 => {
                // LD L, B
                CPU::ld_byte(&mut self.l, self.b);
            },
            0x69 => {
                // LD L, C
                CPU::ld_byte(&mut self.l, self.c);
            },
            0x6a => {
                // LD L, D
                CPU::ld_byte(&mut self.l, self.d);
            },
            0x6b => {
                // LD L, E
                CPU::ld_byte(&mut self.l, self.e);
            },
            0x6c => {
                // LD L, H
                CPU::ld_byte(&mut self.l, self.h);
            },
            0x6d => {
                // LD L, L
                let value = self.l;
                CPU::ld_byte(&mut self.l, value);
            },
            0x6e => {
                // LD L, (HL)
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::ld_byte(&mut self.l, value);
            },
            0x6f => {
                // LD L, A
                CPU::ld_byte(&mut self.l, self.a);
            },
            0x70 => {
                // LD (HL), B
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), self.b);
            },
            0x71 => {
                // LD (HL), C
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), self.c);
            },
            0x72 => {
                // LD (HL), D
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), self.d);
            },
            0x73 => {
                // LD (HL), E
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), self.e);
            },
            0x74 => {
                // LD (HL), H
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), self.h);
            },
            0x75 => {
                // LD (HL), L
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), self.l);
            },
            0x76 => {
                // HALT
                self.halted = true;
            },
            0x77 => {
                // LD (HL), A
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), self.a);
            },
            0x78 => {
                // LD A, B
                CPU::ld_byte(&mut self.a, self.b);
            },
            0x79 => {
                // LD A, C
                CPU::ld_byte(&mut self.a, self.c);
            },
            0x7a => {
                // LD A, D
                CPU::ld_byte(&mut self.a, self.d);
            },
            0x7b => {
                // LD A, E
                CPU::ld_byte(&mut self.a, self.e);
            },
            0x7c => {
                // LD A, H
                CPU::ld_byte(&mut self.a, self.h);
            },
            0x7d => {
                // LD A, L
                CPU::ld_byte(&mut self.a, self.l);
            },
            0x7e => {
                // LD A, (HL)
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::ld_byte(&mut self.a, value);
            },
            0x7f => {
                // LD A, A
                let value = self.a;
                CPU::ld_byte(&mut self.a, value);
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                self.cp_byte(self.a, value);
            },
            0xbf => {
                // CP A, A
                self.cp_byte(self.a, self.a);
            },
            0xc0 => {
                // RET NZ
                if !self.zero {
                    self.ret();
                    branch_taken = true;
                }
            },
            0xc1 => {
                // POP BC
                self.c = self.pop();
                self.b = self.pop();
            },
            0xc2 => {
                // JP NZ, u16
                if !self.zero {
                    self.jp_from_pc();
                    branch_taken = true;
                } else {
                    self.pc += 2;
                }
            },
            0xc3 => {
                // JP u16
                self.jp_from_pc();
            },
            0xc4 => {
                // CALL NZ, u16
                if !self.zero {
                    self.call();
                    branch_taken = true;
                } else {
                    self.pc += 2;
                }
            },
            0xc5 => {
                // PUSH BC
                self.push(self.b);
                self.push(self.c);
            },
            0xc6 => {
                // ADD A, u8
                let value: u8 = self.fetch();
                self.a = self.add_byte(self.a, value);
            },
            0xc7 => {
                // RST 00h
                self.rst(0x0);
            },
            0xc8 => {
                // RET Z
                if self.zero {
                    self.ret();
                    branch_taken = true;
                }
            },
            0xc9 => {
                // RET
                self.ret();
            },
            0xca => {
                // JP Z, u16
                if self.zero {
                    self.jp_from_pc();
                    branch_taken = true;
                } else {
                    self.pc += 2;
                }
            },
            0xcb => {
                // Prefix CB
                return self.execute_cb(instruction);
            },
            0xcc => {
                // CALL Z, u16
                if self.zero {
                    self.call();
                    branch_taken = true;
                } else {
                    self.pc += 2;
                }
            },
            0xcd => {
                // CALL u16
                self.call();
            },
            0xce => {
                // ADC A, u8
                let value: u8 = self.fetch();
                self.a = self.adc_byte(self.a, value);
            },
            0xcf => {
                // RST 08h
                self.rst(0x8);
            },
            0xd0 => {
                // RET NC
                if !self.carry {
                    self.ret();
                    branch_taken = true;
                }
            },
            0xd1 => {
                // POP DE
                self.e = self.pop();
                self.d = self.pop();
            },
            0xd2 => {
                // JP NC, u16
                if !self.carry {
                    self.jp_from_pc();
                    branch_taken = true;
                } else {
                    self.pc += 2;
                }
            },
            0xd3 => {
                // Blank Instruction
            },
            0xd4 => {
                // CALL NC, u16
                if !self.carry {
                    self.call();
                    branch_taken = true;
                } else {
                    self.pc += 2;
                }
            },
            0xd5 => {
                // PUSH DE
                self.push(self.d);
                self.push(self.e);
            },
            0xd6 => {
                // SUB A, u8
                let value: u8 = self.fetch();
                self.a = self.sub_byte(self.a, value);
            },
            0xd7 => {
                // RST 10h
                self.rst(0x10);
            },
            0xd8 => {
                // RET C
                if self.carry {
                    self.ret();
                    branch_taken = true;
                }
            },
            0xd9 => {
                // RETI
                self.ret();
                self.interrupts_enabled = false;
            },
            0xda => {
                // JP C, u16
                if self.carry {
                    self.jp_from_pc();
                    branch_taken = true;
                } else {
                    self.pc += 2;
                }
            },
            0xdb => {
                // Blank Instruction
            },
            0xdc => {
                // CALL C, u16
                if self.carry {
                    self.call();
                    branch_taken = true;
                } else {
                    self.pc += 2;
                }
            },
            0xdd => {
                // Blank Instruction
            },
            0xde => {
                // SBC A, u8
                let value: u8 = self.fetch();
                self.a = self.sbc_byte(self.a, value);
            },
            0xdf => {
                // RST 18h
                self.rst(0x18);
            },
            0xe0 => {
                // LD (FF00 + u8), A
                let value: u8 = self.fetch();
                self.write_to_address(0xff00 + (value as u16), self.a);
            },
            0xe1 => {
                // POP HL
                self.l = self.pop();
                self.h = self.pop();
            },
            0xe2 => {
                // LD (FF00 + C), A
                self.write_to_address(0xff00 + (self.c as u16), self.a);
            },
            0xe3 | 0xe4 => {
                // Blank Instruction
            },
            0xe5 => {
                // PUSH HL
                self.push(self.h);
                self.push(self.l);
            },
            0xe6 => {
                // AND A, u8
                let value: u8 = self.fetch();
                self.a = self.and_byte(self.a, value);
            },
            0xe7 => {
                // RST 20h
                self.rst(0x20);
            },
            0xe8 => {
                // ADD SP, i8
                let sp: i32 = self.sp as i32;
                let value: i32 = self.fetch() as i32;
                let result: i32 = sp + value;
                self.sp = result as u16;
                self.zero = false;
                self.subtract = false;
                self.half_carry = (((sp ^ value ^ (result & 0xFFFF)) & 0x10) == 0x10);
                self.carry = (((sp ^ value ^ (result & 0xFFFF)) & 0x100) == 0x100);
            },
            0xe9 => {
                // JP HL
                self.jp_from_bytes(self.l, self.h);
            },
            0xea => {
                // LD (u16), A
                let lower: u8 = self.fetch();
                let upper: u8 = self.fetch();
                self.write_to_address(bit_logic::compose_bytes(lower, upper), self.a);
            },
            0xeb | 0xec | 0xed => {
                // Blank Instruction
            },
            0xee => {
                // XOR A, u8
                let value: u8 = self.fetch();
                self.a = self.xor_byte(self.a, value);
            },
            0xef => {
                // RST 28h
                self.rst(0x28);
            },
            0xf0 => {
                // LD A, (FF00 + u8)
                let offset: u8 = self.fetch();
                let value: u8 = self.read_from_address(0xff00 + (offset as u16));
                CPU::ld_byte(&mut self.a, value);
            },
            0xf1 => {
                // POP AF
                let popped_f: u8 = self.pop();
                self.set_f(popped_f);
                self.a = self.pop();
            },
            0xf2 => {
                // LD A, (FF00 + C)
                let value: u8 = self.read_from_address(0xff00 + (self.c as u16));
                CPU::ld_byte(&mut self.a, value);
            },
            0xf3 => {
                // DI
                self.pending_interrupt_enable = false;
                self.interrupts_enabled = false;
            },
            0xf4 => {
                // Blank Instruction
            },
            0xf5 => {
                // PUSH AF
                self.push(self.a);
                self.push(self.get_f());
            },
            0xf6 => {
                // OR A, u8
                let value: u8 = self.fetch();
                self.a = self.or_byte(self.a, value);
            },
            0xf7 => {
                // RST 30h
                self.rst(0x30);
            },
            0xf8 => {
                // LD HL, SP + i8
                let value: i32 = self.fetch() as i32;
                let result: i32 = self.sp as i32 + value;
                let (lower, upper) = bit_logic::decompose_bytes(result as u16);
                self.l = lower;
                self.h = upper;
                self.zero = false;
                self.subtract = false;
                self.half_carry = (((self.sp as i32 ^ value ^ (result & 0xFFFF)) & 0x10) == 0x10);
                self.carry = (((self.sp as i32 ^ value ^ (result & 0xFFFF)) & 0x100) == 0x100);
            },
            0xf9 => {
                // LD SP, HL
                self.sp = bit_logic::compose_bytes(self.l, self.h);
            },
            0xfa => {
                // LD A, (u16)
                let lower: u8 = self.fetch();
                let upper: u8 = self.fetch();
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(lower, upper));
                CPU::ld_byte(&mut self.a, value);
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
                let value: u8 = self.fetch();
                self.cp_byte(self.a, value);
            },
            0xff => {
                // RST 38h
                self.rst(0x38);
            },
        }
        if branch_taken {
            BRANCH_INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
        } else {
            INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
        }
    }
}