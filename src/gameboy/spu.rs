use super::Gameboy;
use super::Memory;

const WAVE_FORM: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1,],
    [1, 0, 0, 0, 0, 0, 0, 1,],
    [1, 0, 0, 0, 0, 1, 1, 1,],
    [0, 1, 1, 1, 1, 1, 1, 0,],
];

impl Gameboy {
    pub(crate) fn update_audio(&mut self, cycles: u8) {
        let volume: f32 = 0.5;
        for _ in 0..cycles {
            self.spu.sound_channel_1.update(&self.memory);
            self.spu.sound_channel_2.update(&self.memory);
            self.spu.sound_channel_3.update(&self.memory);
            self.spu.sound_channel_4.update(&self.memory);

            if self.spu.audio_fill_timer == 0 {
                self.spu.audio_fill_timer = 95;
                let mut results = 0.0;
                results += self.spu.sound_channel_1.get_amplitude() * volume;
                results += self.spu.sound_channel_2.get_amplitude(&self.memory) * volume;
                results += self.spu.sound_channel_3.get_amplitude() * volume;
                results += self.spu.sound_channel_4.get_amplitude() * volume;
                self.spu.audio_data.push(results); // Left Channel
                self.spu.audio_data.push(results); // Right Channel
            } else {
                self.spu.audio_fill_timer = self.spu.audio_fill_timer.wrapping_sub(1);
            }
        }
    }
}

pub(crate) struct Spu {
    pub(crate) audio_data: Vec<f32>,
    audio_fill_timer: u8, // CYCLES_PER_SECOND / AUDIO_SAMPLE_RATE (44100)
    sound_channel_1: SoundChannel1,
    sound_channel_2: SoundChannel2,
    sound_channel_3: SoundChannel3,
    sound_channel_4: SoundChannel4,
}

impl Spu {
    pub(crate) fn new() -> Spu {
        Spu {
            audio_data: Vec::new(),
            audio_fill_timer: 95,
            sound_channel_1: SoundChannel1::new(),
            sound_channel_2: SoundChannel2::new(),
            sound_channel_3: SoundChannel3::new(),
            sound_channel_4: SoundChannel4::new(),
        }
    }
}

struct SoundChannel1 {
    amplitude: i16,
    frequency_timer: u16,
    frame_sequence: u8,
    frame_sequence_timer: u16,
    wave_duty_position: u8,
}

impl SoundChannel1 {
    fn new() -> SoundChannel1 {
        SoundChannel1 {
            amplitude: 0,
            frequency_timer: 0,
            frame_sequence: 0,
            frame_sequence_timer: 8192,
            wave_duty_position: 0,
        }
    }

    fn update(&mut self, memory: &Memory) {

    }

    fn get_amplitude(&mut self) -> f32 { 0.0 }
}

struct SoundChannel2 {
    amplitude: i16,
    frequency_timer: u16,
    frame_sequence: u8,
    frame_sequence_timer: u16,
    wave_duty_position: u8,
    envelope_enabled: bool,
    envelope_length: u8,
}

impl SoundChannel2 {
    fn new() -> SoundChannel2 {
        SoundChannel2 {
            amplitude: 0,
            frequency_timer: 0,
            frame_sequence: 8,
            frame_sequence_timer: 8192,
            wave_duty_position: 0,
            envelope_enabled: false,
            envelope_length: 0,
        }
    }

    fn update(&mut self, memory: &Memory) {
        let nr23 = memory.read_from_memory(0xff18);
        let nr24 = memory.read_from_memory(0xff19);
        if self.frequency_timer == 0 {
            let new_frequency_timer = (((nr24 as u16) & 0b111) << 8) | (nr23 as u16);
            self.frequency_timer = (2048 - new_frequency_timer) * 4;
            self.wave_duty_position += 1;
            self.wave_duty_position %= 8;
        } else {
            self.frequency_timer = self.frame_sequence_timer.wrapping_sub(1);
        }

        self.frame_sequence_timer = self.frame_sequence_timer.wrapping_sub(1);
        if self.frame_sequence_timer == 0 {
            self.frame_sequence_timer = 8192;
            self.frame_sequence += 1;
            self.frame_sequence &= 8;

            if self.frame_sequence % 2 == 0 {

            }
        }
    }

    fn get_amplitude(&mut self, memory: &Memory) -> f32 {
        let duty = memory.read_from_memory(0xff16) >> 6;
        if WAVE_FORM[duty as usize][self.wave_duty_position as usize] == 1 {
            (self.amplitude as f32) / 100.0
        } else {
            0.0
        }
    }
}

struct SoundChannel3 {
    frame_sequence: u8,
    frame_sequence_timer: u16,
}

impl SoundChannel3 {
    fn new() -> SoundChannel3 {
        SoundChannel3 {
            frame_sequence: 8,
            frame_sequence_timer: 8192,
        }
    }

    fn update(&mut self, memory: &Memory) {

    }

    fn get_amplitude(&mut self) -> f32 { 0.0 }
}

struct SoundChannel4 {
    frame_sequence: u8,
    frame_sequence_timer: u16,
}

impl SoundChannel4 {
    fn new() -> SoundChannel4 {
        SoundChannel4 {
            frame_sequence: 7,
            frame_sequence_timer: 8192,
        }
    }

    fn update(&mut self, memory: &Memory) {

    }

    fn get_amplitude(&mut self) -> f32 { 0.0 }
}