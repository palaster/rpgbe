use super::{WIDTH, HEIGHT, TIME_BETWEEN_AUDIO_SAMPLING};

// CPU
// GPU
pub const SCREEN_DATA_SIZE: u32 = (WIDTH as u32) * (HEIGHT as u32) * 3;
pub(crate) const SCANLINE_COUNTER_START: u16 = 456;
// Memory
// SPU
// Timer

#[derive(Debug)]
pub(crate) struct Gameboy {
    // CPU
    pub(crate) a: u8,
    pub(crate) b: u8,
    pub(crate) c: u8,
    pub(crate) d: u8,
    pub(crate) e: u8,
    pub(crate) h: u8,
    pub(crate) l: u8,
    pub(crate) sp: u16,
    pub(crate) pc: u16,
    pub(crate) zero: bool,
    pub(crate) subtract: bool,
    pub(crate) half_carry: bool,
    pub(crate) carry: bool,
    pub(crate) halted: bool,
    pub(crate) interrupts_enabled: bool,
    pub(crate) pending_interrupt_enable: bool,
    pub(crate) one_instruction_passed: bool,

    // GPU
    pub(crate) scanline_counter: i32,
    pub(crate) screen_data: [u8; SCREEN_DATA_SIZE as usize],
    pub(crate) scanline_bg: [bool; WIDTH as usize],

    // Memory
    pub(crate) gamepad_state: u8,
    pub(crate) rom_banking: bool,
    pub(crate) enable_ram: bool,
    pub(crate) mbc1: bool,
    pub(crate) mbc2: bool,
    pub(crate) current_rom_bank: u8,
    pub(crate) current_ram_bank: u8,
    pub(crate) ram_banks: Vec<u8>,
    pub(crate) cartridge: Vec<u8>,
    pub(crate) rom: Vec<u8>,

    // SPU
    pub(crate) audio_data: Vec<f32>,
    pub(crate) audio_fill_timer: u8, // CYCLES_PER_SECOND / SAMPLE_RATE (44100)
    pub(crate) sound_channel_1: SoundChannel1,
    pub(crate) sound_channel_2: SoundChannel2,
    pub(crate) sound_channel_3: SoundChannel3,
    pub(crate) sound_channel_4: SoundChannel4,

    // Timer
    pub(crate) timer_counter: i32,
    pub(crate) divider_counter: i32,
}

impl Gameboy {
    pub fn new() -> Gameboy {
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

        Gameboy {
            // CPU
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
            // GPU
            scanline_counter: SCANLINE_COUNTER_START as i32,
            screen_data: [0; SCREEN_DATA_SIZE as usize],
            scanline_bg: [false; WIDTH as usize],
            // Memory
            gamepad_state: 0xff,
            rom_banking: false,
            enable_ram: false,
            mbc1: false,
            mbc2: false,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_banks: vec![0; 0x8000],
            cartridge: Vec::new(),
            rom: rom_vec,
            // SPU
            audio_data: Vec::new(),
            audio_fill_timer: TIME_BETWEEN_AUDIO_SAMPLING,
            sound_channel_1: SoundChannel1::new(),
            sound_channel_2: SoundChannel2::new(),
            sound_channel_3: SoundChannel3::new(),
            sound_channel_4: SoundChannel4::new(),
            // Timer
            timer_counter: 0,
            divider_counter: 0,
        }
    }
}

#[derive(Debug)]
pub(crate) struct SoundChannel1 {
    pub(crate) enabled: bool,
    pub(crate) amplitude: i16,
    pub(crate) frequency: i16,
    pub(crate) frequency_timer: u16,
    pub(crate) frame_sequence: u8,
    pub(crate) frame_sequence_timer: u16,
    pub(crate) wave_duty_position: u8,
    pub(crate) envelope_enabled: bool,
    pub(crate) envelope_sweeps: u8,
    pub(crate) length: u8,
    pub(crate) sweep_enabled: bool,
    pub(crate) sweep_period: u8,
    pub(crate) sweep_shadow: i16,
}

#[derive(Debug)]
pub(crate) struct SoundChannel2 {
    pub(crate) enabled: bool,
    pub(crate) amplitude: i16,
    pub(crate) frequency_timer: u16,
    pub(crate) frame_sequence: u8,
    pub(crate) frame_sequence_timer: u16,
    pub(crate) wave_duty_position: u8,
    pub(crate) envelope_enabled: bool,
    pub(crate) envelope_sweeps: u8,
    pub(crate) length: u8,
}

#[derive(Debug)]
pub(crate) struct SoundChannel3 {
    pub(crate) enabled: bool,
    pub(crate) frequency_timer: u16,
    pub(crate) frame_sequence: u8,
    pub(crate) frame_sequence_timer: u16,
    pub(crate) wave_index: u8,
    pub(crate) length: u16,
}

#[derive(Debug)]
pub(crate) struct SoundChannel4 {
    pub(crate) enabled: bool,
    pub(crate) amplitude: i16,
    pub(crate) frequency_timer: u16,
    pub(crate) frame_sequence: u8,
    pub(crate) frame_sequence_timer: u16,
    pub(crate) envelope_enabled: bool,
    pub(crate) envelope_sweeps: u8,
    pub(crate) length: u8,
    pub(crate) lfsr: u16,
}