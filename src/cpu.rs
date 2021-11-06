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

    fn fetch(&self) -> u8 {
        let memory = MEMORY.lock().unwrap();
        memory.read_from_memory(self.pc.into())
    }

    fn read_from_address(&self, address: u16) -> u8 {
        let memory = MEMORY.lock().unwrap();
        memory.read_from_memory(address.into())
    }

    fn write_to_address(&self, address: u16, value: u8) {
        let mut memory = MEMORY.lock().unwrap();
        memory.write_to_memory(address.into(), value);
    }

    pub fn update(&mut self) -> f64 {
        let instruction = self.fetch();
        self.pc += 1;
        self.execute(instruction)
    }

    fn pop(&mut self, lower: &mut u8, upper: &mut u8) {
        let memory = MEMORY.lock().unwrap();
        *lower = memory.read_from_memory(self.sp.into());
        self.sp += 1;
        *upper = memory.read_from_memory(self.sp.into());
        self.sp += 1;
    }
    
    fn push(&mut self, lower: u8, upper: u8) {
        let mut memory = MEMORY.lock().unwrap();
        self.sp -= 1;
        memory.write_to_memory(self.sp.into(), upper);
        self.sp -= 1;
        memory.write_to_memory(self.sp.into(), lower);
    }

    fn rlc(&mut self, reg: &mut u8) {
        let carry: bool = bit_logic::check_bit(reg, &7);
        let truncated: u8 = bit_logic::bit_value(reg, &7);
        let result: u8 = (*reg << 1) | truncated;
        *reg = result;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = carry;
    }
    
    fn rrc(&mut self, reg: &mut u8) {
        let carry: bool = bit_logic::check_bit(reg, &0);
        let truncated: u8 = bit_logic::bit_value(reg, &0);
        let result: u8 = (*reg >> 1) | (truncated << 7);
        *reg = result;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = carry;
    }
    
    fn rl(&mut self, reg: &mut u8) {
        let carry: bool = self.carry;
        let will_carry: bool = bit_logic::check_bit(reg, &7);
        let mut result: u8 = *reg << 1;
        result |= carry as u8;
        *reg = result;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = will_carry;
    }
    
    fn rr(&mut self, reg: &mut u8) {
        let carry: bool = self.carry;
        let will_carry: bool = bit_logic::check_bit(reg, &0);
        let mut result: u8 = *reg >> 1;
        result |= (carry as u8) << 7;
        *reg = result;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = will_carry;
    }
    
    fn sla(&mut self, reg: &mut u8) {
        
        let carry: bool = bit_logic::check_bit(reg, &7);
        let result: u8 = *reg << 1;
        *reg = result;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = carry;
    }
    
    fn sra(&mut self, reg: &mut u8) {
        let carry: bool = bit_logic::check_bit(reg, &0);
        let top: bool = bit_logic::check_bit(reg, &7);
        let mut result: u8 = *reg >> 1;
        result = bit_logic::set_bit_to(&top, &result, &7);
        *reg = result;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = carry;
    }
    
    fn srl(&mut self, reg: &mut u8) {
        let least_bit_set: bool = bit_logic::check_bit(reg, &0);
        let result: u8 = *reg >> 1;
        *reg = result;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = least_bit_set;
    }
    
    fn swap(&mut self, reg: &mut u8) {
        
        let lower: u8 = *reg & 0x0f;
        let upper: u8 = (*reg & 0xf0) >> 4;
        let result: u8 = (lower << 4) | upper;
        *reg = result;
        self.zero = result == 0;
        self.subtract = false;
        self.half_carry = false;
        self.carry = false;
    }
    
    fn bit(&mut self, bit: &u8, reg: &u8) {
        self.zero = !bit_logic::check_bit(reg, bit);
        self.subtract = false;
        self.half_carry = true;
    }
    
    fn set(&self, bit: &u8, reg: &mut u8) { *reg = bit_logic::set_bit(reg, bit); }
    
    fn res(&self, bit: &u8, reg: &mut u8) { *reg = bit_logic::reset_bit(reg, bit); }

    fn execute(&mut self, instruction: u8) -> f64 {
        match instruction {
            0x00 => {
                // NOP
            },
            0x01 => {
                // LD BC, u16
            },
            0x02 => {
                // LD (BC), A
            },
            0x03 => {
                // INC BC
            },
            0x04 => {
                // INC B
            },
            0x05 => {
                // DEC B
            },
            0x06 => {
                // LD B, u8
            },
            0x07 => {
                // RLCA
            },
            0x08 => {
                // LD (u16), SP
            },
            0x09 => {
                // ADD HL, BC
            },
            0x0a => {
                // LD A, (BC)
            },
            0x0b => {
                // DEC BC
            },
            0x0c => {
                // INC C
            },
            0x0d => {
                // DEC C
            },
            0x0e => {
                // LD C, u8
            },
            0x0f => {
                // RRCA
            },
            0x10 => {
                // STOP
            },
            0x11 => {
                // LD DE, u16
            },
            0x12 => {
                // LD (DE), A
            },
            0x13 => {
                // INC DE
            },
            0x14 => {
                // INC D
            },
            0x15 => {
                // DEC D
            },
            0x16 => {
                // LD D, u8
            },
            0x17 => {
                // RLA
            },
            0x18 => {
                // JR i8
            },
            0x19 => {
                // ADD HL, DE
            },
            0x1a => {
                // LD A, (DE)
            },
            0x1b => {
                // DEC DE
            },
            0x1c => {
                // INC E
            },
            0x1d => {
                // DEC E
            },
            0x1e => {
                // LD E, u8
            },
            0x1f => {
                // RRA
            },
            0x20 => {
                // JR NZ, i8
            },
            0x21 => {
                // LD HL, u16
            },
            0x22 => {
                // LD (HL+), A
            },
            0x23 => {
                // INC HL
            },
            0x24 => {
                // INC H
            },
            0x25 => {
                // DEC H
            },
            0x26 => {
                // LD H, u8
            },
            0x27 => {
                // DAA
            },
            0x28 => {
                // JR Z, i8
            },
            0x29 => {
                // ADD HL, HL
            },
            0x2a => {
                // LD A, (HL+)
            },
            0x2b => {
                // DEC HL
            },
            0x2c => {
                // INC L
            },
            0x2d => {
                // DEC L
            },
            0x2e => {
                // LD L, u8
            },
            0x2f => {
                // CPL
            },
            0x30 => {
                // JR NC, i8
            },
            0x31 => {
                // LD SP, u16
            },
            0x32 => {
                // LD (HL-), A
            },
            0x33 => {
                // INC SP
            },
            0x34 => {
                // INC (HL)
            },
            0x35 => {
                // DEC (HL)
            },
            0x36 => {
                // LD (HL), u8
            },
            0x37 => {
                // SCF
            },
            0x38 => {
                // JR C, i8
            },
            0x39 => {
                // ADD HL, SP
            },
            0x3a => {
                // LD A, (HL-)
            },
            0x3b => {
                // DEC SP
            },
            0x3c => {
                // INC A
            },
            0x3d => {
                // DEC A
            },
            0x3e => {
                // LD A, u8
            },
            0x3f => {
                // CCF
            },
            0x40 => {
                // LD B, B
            },
            0x41 => {
                // LD B, C
            },
            0x42 => {
                // LD B, D
            },
            0x43 => {
                // LD B, E
            },
            0x44 => {
                // LD B, H
            },
            0x45 => {
                // LD B, L
            },
            0x46 => {
                // LD B, (HL)
            },
            0x47 => {
                // LD B, A
            },
            0x48 => {
                // LD C, B
            },
            0x49 => {
                // LD C, C
            },
            0x4a => {
                // LD C, D
            },
            0x4b => {
                // LD C, E
            },
            0x4c => {
                // LD C, H
            },
            0x4d => {
                // LD C, L
            },
            0x4e => {
                // LD C, (HL)
            },
            0x4f => {
                // LD C, A
            },
            0x50 => {
                // LD D, B
            },
            0x51 => {
                // LD D, C
            },
            0x52 => {
                // LD D, D
            },
            0x53 => {
                // LD D, E
            },
            0x54 => {
                // LD D, H
            },
            0x55 => {
                // LD D, L
            },
            0x56 => {
                // LD D, (HL)
            },
            0x57 => {
                // LD D, A
            },
            0x58 => {
                // LD E, B
            },
            0x59 => {
                // LD E, C
            },
            0x5a => {
                // LD E, D
            },
            0x5b => {
                // LD E, E
            },
            0x5c => {
                // LD E, H
            },
            0x5d => {
                // LD E, L
            },
            0x5e => {
                // LD E, (HL)
            },
            0x5f => {
                // LD E, A
            },
            0x60 => {
                // LD H, B
            },
            0x61 => {
                // LD H, C
            },
            0x62 => {
                // LD H, D
            },
            0x63 => {
                // LD H, E
            },
            0x64 => {
                // LD H, H
            },
            0x65 => {
                // LD H, L
            },
            0x66 => {
                // LD H, (HL)
            },
            0x67 => {
                // LD H, A
            },
            0x68 => {
                // LD L, B
            },
            0x69 => {
                // LD L, C
            },
            0x6a => {
                // LD L, D
            },
            0x6b => {
                // LD L, E
            },
            0x6c => {
                // LD L, H
            },
            0x6d => {
                // LD L, L
            },
            0x6e => {
                // LD L, (HL)
            },
            0x6f => {
                // LD L, A
            },
            0x70 => {
                // LD (HL), B
            },
            0x71 => {
                // LD (HL), C
            },
            0x72 => {
                // LD (HL), D
            },
            0x73 => {
                // LD (HL), E
            },
            0x74 => {
                // LD (HL), H
            },
            0x75 => {
                // LD (HL), L
            },
            0x76 => {
                // HALT
            },
            0x77 => {
                // LD (HL), A
            },
            0x78 => {
                // LD A, B
            },
            0x79 => {
                // LD A, C
            },
            0x7a => {
                // LD A, D
            },
            0x7b => {
                // LD A, E
            },
            0x7c => {
                // LD A, H
            },
            0x7d => {
                // LD A, L
            },
            0x7e => {
                // LD A, (HL)
            },
            0x7f => {
                // LD A, A
            },
            0x80 => {
                // ADD A, B
            },
            0x81 => {
                // ADD A, C
            },
            0x82 => {
                // ADD A, D
            },
            0x83 => {
                // ADD A, E
            },
            0x84 => {
                // ADD A, H
            },
            0x85 => {
                // ADD A, L
            },
            0x86 => {
                // ADD A, (HL)
            },
            0x87 => {
                // ADD A, A
            },
            0x88 => {
                // ADC A, B
            },
            0x89 => {
                // ADC A, C
            },
            0x8a => {
                // ADC A, D
            },
            0x8b => {
                // ADC A, E
            },
            0x8c => {
                // ADC A, H
            },
            0x8d => {
                // ADC A, L
            },
            0x8e => {
                // ADC A, (HL)
            },
            0x8f => {
                // ADC A, A
            },
            0x90 => {
                // SUB A, B
            },
            0x91 => {
                // SUB A, C
            },
            0x92 => {
                // SUB A, D
            },
            0x93 => {
                // SUB A, E
            },
            0x94 => {
                // SUB A, H
            },
            0x95 => {
                // SUB A, L
            },
            0x96 => {
                // SUB A, (HL)
            },
            0x97 => {
                // SUB A, A
            },
            0x98 => {
                // SBC A, B
            },
            0x99 => {
                // SBC A, C
            },
            0x9a => {
                // SBC A, D
            },
            0x9b => {
                // SBC A, E
            },
            0x9c => {
                // SBC A, H
            },
            0x9d => {
                // SBC A, L
            },
            0x9e => {
                // SBC A, (HL)
            },
            0x9f => {
                // SBC A, A
            },
            0xa0 => {
                // AND A, B
            },
            0xa1 => {
                // AND A, C
            },
            0xa2 => {
                // AND A, D
            },
            0xa3 => {
                // AND A, E
            },
            0xa4 => {
                // AND A, H
            },
            0xa5 => {
                // AND A, L
            },
            0xa6 => {
                // AND A, (HL)
            },
            0xa7 => {
                // AND A, A
            },
            0xa8 => {
                // XOR A, B
            },
            0xa9 => {
                // XOR A, C
            },
            0xaa => {
                // XOR A, D
            },
            0xab => {
                // XOR A, E
            },
            0xac => {
                // XOR A, H
            },
            0xad => {
                // XOR A, L
            },
            0xae => {
                // XOR A, (HL)
            },
            0xaf => {
                // XOR A, A
            },
            0xb0 => {
                // OR A, B
            },
            0xb1 => {
                // OR A, C
            },
            0xb2 => {
                // OR A, D
            },
            0xb3 => {
                // OR A, E
            },
            0xb4 => {
                // OR A, H
            },
            0xb5 => {
                // OR A, L
            },
            0xb6 => {
                // OR A, (HL)
            },
            0xb7 => {
                // OR A, A
            },
            0xb8 => {
                // CP A, B
            },
            0xb9 => {
                // CP A, C
            },
            0xba => {
                // CP A, D
            },
            0xbb => {
                // CP A, E
            },
            0xbc => {
                // CP A, H
            },
            0xbd => {
                // CP A, L
            },
            0xbe => {
                // CP A, (HL)
            },
            0xbf => {
                // CP A, A
            },
            0xc0 => {
                // RET NZ
            },
            0xc1 => {
                // POP BC
            },
            0xc2 => {
                // JP NZ, u16
            },
            0xc3 => {
                // JP u16
            },
            0xc4 => {
                // CALL NZ, u16
            },
            0xc5 => {
                // PUSH BC
            },
            0xc6 => {
                // ADD A, u8
            },
            0xc7 => {
                // RST 00h
            },
            0xc8 => {
                // RET Z
            },
            0xc9 => {
                // RET
            },
            0xca => {
                // JP Z, u16
            },
            0xcb => {
                // Prefix CB
            },
            0xcc => {
                // CALL Z, u16
            },
            0xcd => {
                // CALL u16
            },
            0xce => {
                // ADC A, u8
            },
            0xcf => {
                // RST 08h
            },
            0xd0 => {
                // RET NC
            },
            0xd1 => {
                // POP DE
            },
            0xd2 => {
                // JP NC, u16
            },
            0xd3 => {
                // Blank Instruction
            },
            0xd4 => {
                // CALL NC, u16
            },
            0xd5 => {
                // PUSH DE
            },
            0xd6 => {
                // SUB A, u8
            },
            0xd7 => {
                // RST 10h
            },
            0xd8 => {
                // RET C
            },
            0xd9 => {
                // RETI
            },
            0xda => {
                // JP C, u16
            },
            0xdb => {
                // Blank Instruction
            },
            0xdc => {
                // CALL C, u16
            },
            0xdd => {
                // Blank Instruction
            },
            0xde => {
                // SBC A, u8
            },
            0xdf => {
                // RST 18h
            },
            0xe0 => {
                // LD (FF00 + u8), A
            },
            0xe1 => {
                // POP HL
            },
            0xe2 => {
                // LD (FF00 + C), A
            },
            0xe3 | 0xe4 => {
                // Blank Instruction
            },
            0xe5 => {
                // PUSH HL
            },
            0xe6 => {
                // AND A, u8
            },
            0xe7 => {
                // RST 20h
            },
            0xe8 => {
                // ADD SP, i8
            },
            0xe9 => {
                // JP HL
            },
            0xea => {
                // LD (u16), A
            },
            0xeb | 0xec | 0xed => {
                // Blank Instruction
            },
            0xee => {
                // XOR A, u8
            },
            0xef => {
                // RST 28h
            },
            0xf0 => {
                // LD A, (FF00 + u8)
            },
            0xf1 => {
                // POP AF
            },
            0xf2 => {
                // LD A, (FF00 + C)
            },
            0xf3 => {
                // DI
            },
            0xf4 => {
                // Blank Instruction
            },
            0xf5 => {
                // PUSH AF
            },
            0xf6 => {
                // OR A, u8
            },
            0xf7 => {
                // RST 30h
            },
            0xf8 => {
                // LD HL, SP + i8
            },
            0xf9 => {
                // LD SP, HL
            },
            0xfa => {
                // LD A, (u16)
            },
            0xfb => {
                // EI
            },
            0xfc | 0xfd => {
                // Blank Instruction
            },
            0xfe => {
                // CP A, u8
            },
            0xff => {
                // RST 38h
            },
        }
        INSTRUCTION_TIMINGS[usize::from(instruction)] as f64
    }
}