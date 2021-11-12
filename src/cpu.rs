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
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                value = self.rlc(value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
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
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                value = self.rrc(value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
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
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                value = self.rl(value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
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
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                value = self.rr(value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
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
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                value = self.sla(value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
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
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                value = self.sra(value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
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
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                value = self.swap(value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
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
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                value = self.srl(value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
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
                let value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                self.bit(7, value);
            },
            0x7f => {
                // BIT 7, A
                self.bit(7, self.a);
            },
            0x80 => {
                // RES 0, B
                CPU::res(0, &mut self.b);
            },
            0x81 => {
                // RES 0, C
                CPU::res(0, &mut self.c);
            },
            0x82 => {
                // RES 0, D
                CPU::res(0, &mut self.d);
            },
            0x83 => {
                // RES 0, E
                CPU::res(0, &mut self.e);
            },
            0x84 => {
                // RES 0, H
                CPU::res(0, &mut self.h);
            },
            0x85 => {
                // RES 0, L
                CPU::res(0, &mut self.l);
            },
            0x86 => {
                // RES 0, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::res(0, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0x87 => {
                // RES 0, A
                CPU::res(0, &mut self.a);
            },
            0x88 => {
                // RES 1, B
                CPU::res(1, &mut self.b);
            },
            0x89 => {
                // RES 1, C
                CPU::res(1, &mut self.c);
            },
            0x8a => {
                // RES 1, D
                CPU::res(1, &mut self.d);
            },
            0x8b => {
                // RES 1, E
                CPU::res(1, &mut self.e);
            },
            0x8c => {
                // RES 1, H
                CPU::res(1, &mut self.h);
            },
            0x8d => {
                // RES 1, L
                CPU::res(1, &mut self.l);
            },
            0x8e => {
                // RES 1, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::res(1, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0x8f => {
                // RES 1, A
                CPU::res(1, &mut self.a);
            },
            0x90 => {
                // RES 2, B
                CPU::res(2, &mut self.b);
            },
            0x91 => {
                // RES 2, C
                CPU::res(2, &mut self.c);
            },
            0x92 => {
                // RES 2, D
                CPU::res(2, &mut self.d);
            },
            0x93 => {
                // RES 2, E
                CPU::res(2, &mut self.e);
            },
            0x94 => {
                // RES 2, H
                CPU::res(2, &mut self.h);
            },
            0x95 => {
                // RES 2, L
                CPU::res(2, &mut self.l);
            },
            0x96 => {
                // RES 2, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::res(2, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0x97 => {
                // RES 2, A
                CPU::res(2, &mut self.a);
            },
            0x98 => {
                // RES 3, B
                CPU::res(3, &mut self.b);
            },
            0x99 => {
                // RES 3, C
                CPU::res(3, &mut self.c);
            },
            0x9a => {
                // RES 3, D
                CPU::res(3, &mut self.d);
            },
            0x9b => {
                // RES 3, E
                CPU::res(3, &mut self.e);
            },
            0x9c => {
                // RES 3, H
                CPU::res(3, &mut self.h);
            },
            0x9d => {
                // RES 3, L
                CPU::res(3, &mut self.l);
            },
            0x9e => {
                // RES 3, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::res(3, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0x9f => {
                // RES 3, A
                CPU::res(3, &mut self.a);
            },
            0xa0 => {
                // RES 4, B
                CPU::res(4, &mut self.b);
            },
            0xa1 => {
                // RES 4, C
                CPU::res(4, &mut self.c);
            },
            0xa2 => {
                // RES 4, D
                CPU::res(4, &mut self.d);
            },
            0xa3 => {
                // RES 4, E
                CPU::res(4, &mut self.e);
            },
            0xa4 => {
                // RES 4, H
                CPU::res(4, &mut self.h);
            },
            0xa5 => {
                // RES 4, L
                CPU::res(4, &mut self.l);
            },
            0xa6 => {
                // RES 4, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::res(4, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xa7 => {
                // RES 4, A
                CPU::res(4, &mut self.a);
            },
            0xa8 => {
                // RES 5, B
                CPU::res(5, &mut self.b);
            },
            0xa9 => {
                // RES 5, C
                CPU::res(5, &mut self.c);
            },
            0xaa => {
                // RES 5, D
                CPU::res(5, &mut self.d);
            },
            0xab => {
                // RES 5, E
                CPU::res(5, &mut self.e);
            },
            0xac => {
                // RES 5, H
                CPU::res(5, &mut self.h);
            },
            0xad => {
                // RES 5, L
                CPU::res(5, &mut self.l);
            },
            0xae => {
                // RES 5, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::res(5, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xaf => {
                // RES 5, A
                CPU::res(5, &mut self.a);
            },
            0xb0 => {
                // RES 6, B
                CPU::res(6, &mut self.b);
            },
            0xb1 => {
                // RES 6, C
                CPU::res(6, &mut self.c);
            },
            0xb2 => {
                // RES 6, D
                CPU::res(6, &mut self.d);
            },
            0xb3 => {
                // RES 6, E
                CPU::res(6, &mut self.e);
            },
            0xb4 => {
                // RES 6, H
                CPU::res(6, &mut self.h);
            },
            0xb5 => {
                // RES 6, L
                CPU::res(6, &mut self.l);
            },
            0xb6 => {
                // RES 6, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::res(6, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xb7 => {
                // RES 6, A
                CPU::res(6, &mut self.a);
            },
            0xb8 => {
                // RES 7, B
                CPU::res(7, &mut self.b);
            },
            0xb9 => {
                // RES 7, C
                CPU::res(7, &mut self.c);
            },
            0xba => {
                // RES 7, D
                CPU::res(7, &mut self.d);
            },
            0xbb => {
                // RES 7, E
                CPU::res(7, &mut self.e);
            },
            0xbc => {
                // RES 7, H
                CPU::res(7, &mut self.h);
            },
            0xbd => {
                // RES 7, L
                CPU::res(7, &mut self.l);
            },
            0xbe => {
                // RES 7, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::res(7, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xbf => {
                // RES 7, A
                CPU::res(7, &mut self.a);
            },
            0xc0 => {
                // SET 0, B
                CPU::set(0, &mut self.b);
            },
            0xc1 => {
                // SET 0, C
                CPU::set(0, &mut self.c);
            },
            0xc2 => {
                // SET 0, D
                CPU::set(0, &mut self.d);
            },
            0xc3 => {
                // SET 0, E
                CPU::set(0, &mut self.e);
            },
            0xc4 => {
                // SET 0, H
                CPU::set(0, &mut self.h);
            },
            0xc5 => {
                // SET 0, L
                CPU::set(0, &mut self.l);
            },
            0xc6 => {
                // SET 0, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::set(0, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xc7 => {
                // SET 0, A
                CPU::set(0, &mut self.a);
            },
            0xc8 => {
                // SET 1, B
                CPU::set(1, &mut self.b);
            },
            0xc9 => {
                // SET 1, C
                CPU::set(1, &mut self.c);
            },
            0xca => {
                // SET 1, D
                CPU::set(1, &mut self.d);
            },
            0xcb => {
                // SET 1, E
                CPU::set(1, &mut self.e);
            },
            0xcc => {
                // SET 1, H
                CPU::set(1, &mut self.h);
            },
            0xcd => {
                // SET 1, L
                CPU::set(1, &mut self.l);
            },
            0xce => {
                // SET 1, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::set(1, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xcf => {
                // SET 1, A
                CPU::set(1, &mut self.a);
            },
            0xd0 => {
                // SET 2, B
                CPU::set(2, &mut self.b);
            },
            0xd1 => {
                // SET 2, C
                CPU::set(2, &mut self.c);
            },
            0xd2 => {
                // SET 2, D
                CPU::set(2, &mut self.d);
            },
            0xd3 => {
                // SET 2, E
                CPU::set(2, &mut self.e);
            },
            0xd4 => {
                // SET 2, H
                CPU::set(2, &mut self.h);
            },
            0xd5 => {
                // SET 2, L
                CPU::set(2, &mut self.l);
            },
            0xd6 => {
                // SET 2, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::set(2, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xd7 => {
                // SET 2, A
                CPU::set(2, &mut self.a);
            },
            0xd8 => {
                // SET 3, B
                CPU::set(3, &mut self.b);
            },
            0xd9 => {
                // SET 3, C
                CPU::set(3, &mut self.c);
            },
            0xda => {
                // SET 3, D
                CPU::set(3, &mut self.d);
            },
            0xdb => {
                // SET 3, E
                CPU::set(3, &mut self.e);
            },
            0xdc => {
                // SET 3, H
                CPU::set(3, &mut self.h);
            },
            0xdd => {
                // SET 3, L
                CPU::set(3, &mut self.l);
            },
            0xde => {
                // SET 3, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::set(3, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xdf => {
                // SET 3, A
                CPU::set(3, &mut self.a);
            },
            0xe0 => {
                // SET 4, B
                CPU::set(4, &mut self.b);
            },
            0xe1 => {
                // SET 4, C
                CPU::set(4, &mut self.c);
            },
            0xe2 => {
                // SET 4, D
                CPU::set(4, &mut self.d);
            },
            0xe3 => {
                // SET 4, E
                CPU::set(4, &mut self.e);
            },
            0xe4 => {
                // SET 4, H
                CPU::set(4, &mut self.h);
            },
            0xe5 => {
                // SET 4, L
                CPU::set(4, &mut self.l);
            },
            0xe6 => {
                // SET 4, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::set(4, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xe7 => {
                // SET 4, A
                CPU::set(4, &mut self.a);
            },
            0xe8 => {
                // SET 5, B
                CPU::set(5, &mut self.b);
            },
            0xe9 => {
                // SET 5, C
                CPU::set(5, &mut self.c);
            },
            0xea => {
                // SET 5, D
                CPU::set(5, &mut self.d);
            },
            0xeb => {
                // SET 5, E
                CPU::set(5, &mut self.e);
            },
            0xec => {
                // SET 5, H
                CPU::set(5, &mut self.h);
            },
            0xed => {
                // SET 5, L
                CPU::set(5, &mut self.l);
            },
            0xee => {
                // SET 5, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::set(5, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xef => {
                // SET 5, A
                CPU::set(5, &mut self.a);
            },
            0xf0 => {
                // SET 6, B
                CPU::set(6, &mut self.b);
            },
            0xf1 => {
                // SET 6, C
                CPU::set(6, &mut self.c);
            },
            0xf2 => {
                // SET 6, D
                CPU::set(6, &mut self.d);
            },
            0xf3 => {
                // SET 6, E
                CPU::set(6, &mut self.e);
            },
            0xf4 => {
                // SET 6, H
                CPU::set(6, &mut self.h);
            },
            0xf5 => {
                // SET 6, L
                CPU::set(6, &mut self.l);
            },
            0xf6 => {
                // SET 6, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::set(6, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xf7 => {
                // SET 6, A
                CPU::set(6, &mut self.a);
            },
            0xf8 => {
                // SET 7, B
                CPU::set(7, &mut self.b);
            },
            0xf9 => {
                // SET 7, C
                CPU::set(7, &mut self.c);
            },
            0xfa => {
                // SET 7, D
                CPU::set(7, &mut self.e);
            },
            0xfb => {
                // SET 7, E
                CPU::set(7, &mut self.e);
            },
            0xfc => {
                // SET 7, H
                CPU::set(7, &mut self.h);
            },
            0xfd => {
                // SET 7, L
                CPU::set(7, &mut self.l);
            },
            0xfe => {
                // SET 7, (HL)
                let mut value: u8 = self.read_from_address(bit_logic::compose_bytes(self.l, self.h));
                CPU::set(7, &mut value);
                self.write_to_address(bit_logic::compose_bytes(self.l, self.h), value);
            },
            0xff => {
                // SET 7, A
                CPU::set(7, &mut self.a);
            },
        }
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