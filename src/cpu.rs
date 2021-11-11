use super::MEMORY;
//use crate::memory::Memory;
use crate::bit_logic;

static INSTRUCTION_TIMINGS: [u8; 256] = [0; 256];
static BRANCH_INSTRUCTION_TIMINGS: [u8; 256] = [0; 256];
static CB_INSTRUCTION_TIMINGS: [u8; 256] = [0; 256];

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
}

impl CPU {
    pub fn new() -> CPU {
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
        }
    }

    pub fn is_halted(&self) -> bool { self.halted }

    fn fetch(&mut self) -> u8 {
        let memory = MEMORY.lock().unwrap();
        let value = memory.read_from_memory(self.pc.into());
        self.pc += 1;
        value
    }

    fn read_from_address(address: u16) -> u8 {
        let memory = MEMORY.lock().unwrap();
        memory.read_from_memory(address.into())
    }

    fn write_to_address(address: u16, value: u8) {
        let mut memory = MEMORY.lock().unwrap();
        memory.write_to_memory(address.into(), value);
    }

    fn pop(&mut self) -> u8 {
        let memory = MEMORY.lock().unwrap();
        let value: u8 = memory.read_from_memory(self.sp.into());
        self.sp += 1;
        value
    }
    
    fn push(&mut self, value: u8) {
        let mut memory = MEMORY.lock().unwrap();
        self.sp -= 1;
        memory.write_to_memory(self.sp.into(), value);
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
        self.push(self.pc as u8);
        self.push((self.pc >> 8) as u8);
        self.jp_from_bytes(lower_new, upper_new);
    }

    fn rst(&mut self, value: u8) {
        self.push(self.pc as u8);
        self.push((self.pc >> 8) as u8);
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

    fn execute(&mut self, instruction: u8) -> f64 {
        match instruction {
            0x00 => {
                // NOP
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x01 => {
                // LD BC, u16
                let lower: u8 = self.fetch();
                let upper: u8 = self.fetch();
                CPU::ld_word(&mut self.c, &mut self.b, lower, upper);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x02 => {
                // LD (BC), A
                CPU::write_to_address(bit_logic::compose_bytes(self.c, self.b), self.a);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x03 => {
                // INC BC
                CPU::inc_word(&mut self.c, &mut self.b);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x04 => {
                // INC B
                self.b = self.inc_byte(self.b);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x05 => {
                // DEC B
                self.b = self.dec_byte(self.b);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x06 => {
                // LD B, u8
                let value = self.fetch();
                CPU::ld_byte(&mut self.b, value);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x07 => {
                // RLCA
                self.a = self.rlc(self.a);
                self.zero = false;
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x08 => {
                // LD (u16), SP
                let lower = self.fetch();
                let upper = self.fetch();
                let address = bit_logic::compose_bytes(lower, upper);
                CPU::write_to_address(address, self.sp as u8);
                CPU::write_to_address(address + 1, (self.sp >> 8) as u8);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x09 => {
                // ADD HL, BC
                let (lower, upper) = self.add_word(self.l, self.h, self.c, self.b);
                self.l = lower;
                self.h = upper;
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x0a => {
                // LD A, (BC)
                CPU::ld_byte(&mut self.a, CPU::read_from_address(bit_logic::compose_bytes(self.c, self.b)));
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x0b => {
                // DEC BC
                CPU::dec_word(&mut self.c, &mut self.b);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x0c => {
                // INC C
                self.c = self.inc_byte(self.c);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x0d => {
                // DEC C
                self.c = self.dec_byte(self.c);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x0e => {
                // LD C, u8
                let value = self.fetch();
                CPU::ld_byte(&mut self.c, value);
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x0f => {
                // RRCA
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x10 => {
                // STOP
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x11 => {
                // LD DE, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x12 => {
                // LD (DE), A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x13 => {
                // INC DE
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x14 => {
                // INC D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x15 => {
                // DEC D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x16 => {
                // LD D, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x17 => {
                // RLA
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x18 => {
                // JR i8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x19 => {
                // ADD HL, DE
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x1a => {
                // LD A, (DE)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x1b => {
                // DEC DE
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x1c => {
                // INC E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x1d => {
                // DEC E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x1e => {
                // LD E, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x1f => {
                // RRA
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x20 => {
                // JR NZ, i8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x21 => {
                // LD HL, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x22 => {
                // LD (HL+), A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x23 => {
                // INC HL
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x24 => {
                // INC H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x25 => {
                // DEC H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x26 => {
                // LD H, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x27 => {
                // DAA
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x28 => {
                // JR Z, i8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x29 => {
                // ADD HL, HL
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x2a => {
                // LD A, (HL+)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x2b => {
                // DEC HL
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x2c => {
                // INC L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x2d => {
                // DEC L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x2e => {
                // LD L, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x2f => {
                // CPL
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x30 => {
                // JR NC, i8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x31 => {
                // LD SP, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x32 => {
                // LD (HL-), A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x33 => {
                // INC SP
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x34 => {
                // INC (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x35 => {
                // DEC (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x36 => {
                // LD (HL), u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x37 => {
                // SCF
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x38 => {
                // JR C, i8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x39 => {
                // ADD HL, SP
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x3a => {
                // LD A, (HL-)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x3b => {
                // DEC SP
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x3c => {
                // INC A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x3d => {
                // DEC A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x3e => {
                // LD A, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x3f => {
                // CCF
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x40 => {
                // LD B, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x41 => {
                // LD B, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x42 => {
                // LD B, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x43 => {
                // LD B, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x44 => {
                // LD B, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x45 => {
                // LD B, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x46 => {
                // LD B, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x47 => {
                // LD B, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x48 => {
                // LD C, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x49 => {
                // LD C, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x4a => {
                // LD C, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x4b => {
                // LD C, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x4c => {
                // LD C, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x4d => {
                // LD C, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x4e => {
                // LD C, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x4f => {
                // LD C, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x50 => {
                // LD D, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x51 => {
                // LD D, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x52 => {
                // LD D, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x53 => {
                // LD D, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x54 => {
                // LD D, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x55 => {
                // LD D, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x56 => {
                // LD D, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x57 => {
                // LD D, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x58 => {
                // LD E, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x59 => {
                // LD E, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x5a => {
                // LD E, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x5b => {
                // LD E, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x5c => {
                // LD E, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x5d => {
                // LD E, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x5e => {
                // LD E, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x5f => {
                // LD E, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x60 => {
                // LD H, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x61 => {
                // LD H, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x62 => {
                // LD H, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x63 => {
                // LD H, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x64 => {
                // LD H, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x65 => {
                // LD H, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x66 => {
                // LD H, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x67 => {
                // LD H, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x68 => {
                // LD L, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x69 => {
                // LD L, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x6a => {
                // LD L, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x6b => {
                // LD L, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x6c => {
                // LD L, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x6d => {
                // LD L, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x6e => {
                // LD L, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x6f => {
                // LD L, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x70 => {
                // LD (HL), B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x71 => {
                // LD (HL), C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x72 => {
                // LD (HL), D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x73 => {
                // LD (HL), E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x74 => {
                // LD (HL), H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x75 => {
                // LD (HL), L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x76 => {
                // HALT
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x77 => {
                // LD (HL), A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x78 => {
                // LD A, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x79 => {
                // LD A, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x7a => {
                // LD A, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x7b => {
                // LD A, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x7c => {
                // LD A, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x7d => {
                // LD A, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x7e => {
                // LD A, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x7f => {
                // LD A, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x80 => {
                // ADD A, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x81 => {
                // ADD A, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x82 => {
                // ADD A, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x83 => {
                // ADD A, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x84 => {
                // ADD A, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x85 => {
                // ADD A, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x86 => {
                // ADD A, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x87 => {
                // ADD A, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x88 => {
                // ADC A, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x89 => {
                // ADC A, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x8a => {
                // ADC A, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x8b => {
                // ADC A, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x8c => {
                // ADC A, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x8d => {
                // ADC A, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x8e => {
                // ADC A, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x8f => {
                // ADC A, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x90 => {
                // SUB A, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x91 => {
                // SUB A, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x92 => {
                // SUB A, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x93 => {
                // SUB A, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x94 => {
                // SUB A, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x95 => {
                // SUB A, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x96 => {
                // SUB A, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x97 => {
                // SUB A, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x98 => {
                // SBC A, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x99 => {
                // SBC A, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x9a => {
                // SBC A, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x9b => {
                // SBC A, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x9c => {
                // SBC A, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x9d => {
                // SBC A, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x9e => {
                // SBC A, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0x9f => {
                // SBC A, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xa0 => {
                // AND A, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xa1 => {
                // AND A, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xa2 => {
                // AND A, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xa3 => {
                // AND A, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xa4 => {
                // AND A, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xa5 => {
                // AND A, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xa6 => {
                // AND A, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xa7 => {
                // AND A, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xa8 => {
                // XOR A, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xa9 => {
                // XOR A, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xaa => {
                // XOR A, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xab => {
                // XOR A, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xac => {
                // XOR A, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xad => {
                // XOR A, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xae => {
                // XOR A, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xaf => {
                // XOR A, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xb0 => {
                // OR A, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xb1 => {
                // OR A, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xb2 => {
                // OR A, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xb3 => {
                // OR A, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xb4 => {
                // OR A, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xb5 => {
                // OR A, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xb6 => {
                // OR A, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xb7 => {
                // OR A, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xb8 => {
                // CP A, B
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xb9 => {
                // CP A, C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xba => {
                // CP A, D
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xbb => {
                // CP A, E
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xbc => {
                // CP A, H
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xbd => {
                // CP A, L
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xbe => {
                // CP A, (HL)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xbf => {
                // CP A, A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xc0 => {
                // RET NZ
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xc1 => {
                // POP BC
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xc2 => {
                // JP NZ, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xc3 => {
                // JP u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xc4 => {
                // CALL NZ, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xc5 => {
                // PUSH BC
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xc6 => {
                // ADD A, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xc7 => {
                // RST 00h
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xc8 => {
                // RET Z
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xc9 => {
                // RET
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xca => {
                // JP Z, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xcb => {
                // Prefix CB
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xcc => {
                // CALL Z, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xcd => {
                // CALL u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xce => {
                // ADC A, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xcf => {
                // RST 08h
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xd0 => {
                // RET NC
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xd1 => {
                // POP DE
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xd2 => {
                // JP NC, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xd3 => {
                // Blank Instruction
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xd4 => {
                // CALL NC, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xd5 => {
                // PUSH DE
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xd6 => {
                // SUB A, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xd7 => {
                // RST 10h
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xd8 => {
                // RET C
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xd9 => {
                // RETI
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xda => {
                // JP C, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xdb => {
                // Blank Instruction
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xdc => {
                // CALL C, u16
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xdd => {
                // Blank Instruction
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xde => {
                // SBC A, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xdf => {
                // RST 18h
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xe0 => {
                // LD (FF00 + u8), A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xe1 => {
                // POP HL
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xe2 => {
                // LD (FF00 + C), A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xe3 | 0xe4 => {
                // Blank Instruction
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xe5 => {
                // PUSH HL
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xe6 => {
                // AND A, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xe7 => {
                // RST 20h
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xe8 => {
                // ADD SP, i8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xe9 => {
                // JP HL
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xea => {
                // LD (u16), A
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xeb | 0xec | 0xed => {
                // Blank Instruction
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xee => {
                // XOR A, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xef => {
                // RST 28h
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xf0 => {
                // LD A, (FF00 + u8)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xf1 => {
                // POP AF
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xf2 => {
                // LD A, (FF00 + C)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xf3 => {
                // DI
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xf4 => {
                // Blank Instruction
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xf5 => {
                // PUSH AF
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xf6 => {
                // OR A, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xf7 => {
                // RST 30h
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xf8 => {
                // LD HL, SP + i8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xf9 => {
                // LD SP, HL
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xfa => {
                // LD A, (u16)
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xfb => {
                // EI
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xfc | 0xfd => {
                // Blank Instruction
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xfe => {
                // CP A, u8
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
            0xff => {
                // RST 38h
                INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
            },
        }
    }
}