use crate::{ bit_logic, TIME_BETWEEN_AUDIO_SAMPLING};
use super::Memory;

const WAVE_FORM: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1,],
    [1, 0, 0, 0, 0, 0, 0, 1,],
    [1, 0, 0, 0, 0, 1, 1, 1,],
    [0, 1, 1, 1, 1, 1, 1, 0,],
];

const SOUND_CHANNEL_4_DIVISOR: [u8; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

pub(crate) trait SoundChannel {
    fn reset(&mut self, memory: &Memory, length: u8);
    fn update(&mut self, memory: &mut Memory);
    fn get_amplitude(&mut self, memory: &Memory) -> f32;
}

pub(crate) struct Spu {
    pub(crate) audio_data: Vec<f32>,
    pub(crate) audio_fill_timer: u8, // CYCLES_PER_SECOND / SAMPLE_RATE (44100)
    pub(crate) sound_channel_1: SoundChannel1,
    pub(crate) sound_channel_2: SoundChannel2,
    pub(crate) sound_channel_3: SoundChannel3,
    pub(crate) sound_channel_4: SoundChannel4,
}

impl Spu {
    pub(crate) fn new() -> Spu {
        Spu {
            audio_data: Vec::new(),
            audio_fill_timer: TIME_BETWEEN_AUDIO_SAMPLING,
            sound_channel_1: SoundChannel1::new(),
            sound_channel_2: SoundChannel2::new(),
            sound_channel_3: SoundChannel3::new(),
            sound_channel_4: SoundChannel4::new(),
        }
    }
}

pub(crate) struct SoundChannel1 {
    enabled: bool,
    amplitude: i16,
    frequency: i16,
    frequency_timer: u16,
    frame_sequence: u8,
    frame_sequence_timer: u16,
    wave_duty_position: u8,
    envelope_enabled: bool,
    envelope_sweeps: u8,
    length: u8,
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_shadow: i16,
}

impl SoundChannel1 {
    fn new() -> SoundChannel1 {
        SoundChannel1 {
            enabled: false,
            amplitude: 0,
            frequency: 0,
            frequency_timer: 0,
            frame_sequence: 0,
            frame_sequence_timer: 8192,
            wave_duty_position: 0,
            envelope_enabled: false,
            envelope_sweeps: 0,
            length: 0,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_shadow: 0,
        }
    }
}

impl SoundChannel for SoundChannel1 {
    fn reset(&mut self, memory: &Memory, length: u8) {
        let nr10: u8 = memory.read_from_memory(0xff10);
        let nr12: u8 = memory.read_from_memory(0xff12);
        let nr13: u8 = memory.read_from_memory(0xff13);
        let nr14: u8 = memory.read_from_memory(0xff14);
        if self.length == 0 { self.length = 64 - length; }
        self.enabled = true;
        self.amplitude = (nr12 >> 4) as i16;
        self.envelope_sweeps = nr12 & 0b111;
        self.envelope_enabled = true;

        let new_frequency_timer = (nr14 as u16) & 0b111 << 8 | (nr13 as u16);
        self.frequency_timer = (2048 - new_frequency_timer) * 4;

        self.sweep_period = (nr10 >> 4) & 0b111;
        let sweep_shift = nr10 & 0b111;
        let sweep_negate = if bit_logic::check_bit(nr10, 3) { -1 } else { 1 };
        self.sweep_enabled = self.sweep_period != 0 && sweep_shift != 0;
        self.sweep_shadow = ((nr14 as i16) & 0b111) << 8 | (nr13 as i16);
        if sweep_shift != 0 {
            if (self.sweep_shadow as i32) + ((self.sweep_shadow as i32) >> sweep_shift) * sweep_negate > 2047 {
                self.sweep_enabled = false;
                self.enabled = false;
            }
        }
        if nr12 >> 3 == 0 { self.enabled = false; }
    }

    fn update(&mut self, memory: &mut Memory) {
        let nr10 = memory.read_from_memory(0xff10);
        let nr11 = memory.read_from_memory(0xff11);
        let nr12 = memory.read_from_memory(0xff12);
        let nr13 = memory.read_from_memory(0xff13);
        let nr14 = memory.read_from_memory(0xff14);

        self.frame_sequence_timer = self.frame_sequence_timer.wrapping_sub(1);
        if self.frame_sequence_timer == 0 {
            self.frame_sequence_timer = 8192;
            self.frame_sequence += 1;
            self.frame_sequence &= 8;

            if self.frame_sequence % 2 == 0 && bit_logic::check_bit(nr14, 6)  && self.length != 0 {
                self.length = self.length.wrapping_sub(1);
                if self.length == 0 { self.enabled = false; }
            }
        }

        if (self.frame_sequence == 2 || self.frame_sequence == 6) && self.frame_sequence_timer == 8192 && ((nr10 >> 4) & 0b111) != 0 && (nr10 & 0b111) != 0 {
            self.sweep_period -= 1;
            if self.sweep_period <= 0 {
                self.sweep_period = (nr10 >> 4) & 0b111;
                if self.sweep_period == 0 { self.sweep_period = 8; }
                if ((nr10 >> 4) & 0b111) != 0 && self.sweep_enabled && (nr10 & 0b111) != 0 {
                    let sweep_shift = nr10 & 0b111;
                    let sweep_negate = if bit_logic::check_bit(nr10, 3) { -1 } else { 1 };
                    let new_frequency = self.sweep_shadow + (self.sweep_shadow >> sweep_shift) * sweep_negate;
                    if new_frequency < 2048 && sweep_shift != 0 {
                        self.sweep_shadow = new_frequency;
                        memory.write_to_memory(0xff13, (self.sweep_shadow & 0xff) as u8);
                        memory.write_to_memory(0xff14, ((self.sweep_shadow >> 8) & 0b111) as u8);
                        if self.sweep_shadow + (self.sweep_shadow >> sweep_shift) * sweep_negate > 2047 {
                            self.enabled = false;
                            self.sweep_enabled = false;
                        }
                    }
                    if new_frequency > 2047 {
                        self.enabled = false;
                        self.sweep_enabled = false;
                    }
                    if self.sweep_shadow + (self.sweep_shadow >> sweep_shift) * sweep_negate > 2047 {
                        self.enabled = false;
                        self.sweep_enabled = false;
                    }
                }
            }
        }

        if self.frequency_timer == 0 {
            let new_frequency_timer = (((nr14 as u16) & 0b111) << 8) | (nr13 as u16);
            self.frequency_timer = (2048 - new_frequency_timer) * 4;
            self.wave_duty_position += 1;
            self.wave_duty_position %= 8;
        } else {
            self.frequency_timer = self.frequency_timer.wrapping_sub(1);
        }

        let duty = nr11 >> 6;
        if WAVE_FORM[duty as usize][self.wave_duty_position as usize] == 1 {
            self.frequency = self.amplitude;
        } else {
            self.frequency = 0;
        }
        
        if self.frame_sequence == 7 && self.frame_sequence_timer == 8192 && self.envelope_enabled && (nr12 & 0b111) != 0 {
            self.envelope_sweeps = self.envelope_sweeps.wrapping_sub(1);
            if self.envelope_sweeps == 0 {
                self.envelope_sweeps = nr12 & 0b111;
                let new_amplitude = self.amplitude + if bit_logic::check_bit(nr12, 3) { 1 } else { -1 };
                if new_amplitude >= 0 && new_amplitude <= 15 {
                    self.amplitude = new_amplitude;
                    self.frequency = self.amplitude;
                } else {
                    self.envelope_enabled = false;
                }
            }
        }
    }

    fn get_amplitude(&mut self, _memory: &Memory) -> f32 {
        if self.enabled {
            self.frequency as f32 / 100.0
        } else {
            0.0
        }
    }
}

pub(crate) struct SoundChannel2 {
    enabled: bool,
    amplitude: i16,
    frequency_timer: u16,
    frame_sequence: u8,
    frame_sequence_timer: u16,
    wave_duty_position: u8,
    envelope_enabled: bool,
    envelope_sweeps: u8,
    length: u8,
}

impl SoundChannel2 {
    fn new() -> SoundChannel2 {
        SoundChannel2 {
            enabled: false,
            amplitude: 0,
            frequency_timer: 0,
            frame_sequence: 8,
            frame_sequence_timer: 8192,
            wave_duty_position: 0,
            envelope_enabled: false,
            envelope_sweeps: 0,
            length: 0,
        }
    }
}

impl SoundChannel for SoundChannel2 {
    fn reset(&mut self, memory: &Memory, length: u8) {
        let nr22: u8 = memory.read_from_memory(0xff17);
        if self.length == 0 { self.length = 64 - length; }
        self.enabled = true;
        self.amplitude = (nr22 >> 4) as i16;
        self.envelope_sweeps = nr22 & 0b111;
        self.envelope_enabled = true;
        if nr22 >> 3 == 0 { self.enabled = false; }
    }

    fn update(&mut self, memory: &mut Memory) {
        let nr22 = memory.read_from_memory(0xff17);
        let nr23 = memory.read_from_memory(0xff18);
        let nr24 = memory.read_from_memory(0xff19);
        if self.frequency_timer == 0 {
            let new_frequency_timer = (((nr24 as u16) & 0b111) << 8) | (nr23 as u16);
            self.frequency_timer = (2048 - new_frequency_timer) * 4;
            self.wave_duty_position += 1;
            self.wave_duty_position %= 8;
        } else {
            self.frequency_timer = self.frequency_timer.wrapping_sub(1);
        }

        self.frame_sequence_timer = self.frame_sequence_timer.wrapping_sub(1);
        if self.frame_sequence_timer == 0 {
            self.frame_sequence_timer = 8192;
            self.frame_sequence += 1;
            self.frame_sequence &= 8;

            if self.frame_sequence % 2 == 0 && bit_logic::check_bit(nr24, 6)  && self.length != 0 {
                self.length = self.length.wrapping_sub(1);
                if self.length == 0 { self.enabled = false; }
            }

            if self.frame_sequence == 7 && self.envelope_enabled && (nr22 & 0b111) != 0 {
                self.envelope_sweeps = self.envelope_sweeps.wrapping_sub(1);
                if self.envelope_sweeps == 0 {
                    self.envelope_sweeps = nr22 & 0b111;
                    let new_amplitude = self.amplitude + if bit_logic::check_bit(nr22, 3) { 1 } else { -1 };
                    if new_amplitude >= 0 && new_amplitude <= 15 {
                        self.amplitude = new_amplitude;
                    } else {
                        self.envelope_enabled = false;
                    }
                }
            }
        }
    }

    fn get_amplitude(&mut self, memory: &Memory) -> f32 {
        if self.enabled {
            let duty = memory.read_from_memory(0xff16) >> 6;
            if WAVE_FORM[duty as usize][self.wave_duty_position as usize] == 1 {
                (self.amplitude as f32) / 100.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

pub(crate) struct SoundChannel3 {
    enabled: bool,
    frequency_timer: u16,
    frame_sequence: u8,
    frame_sequence_timer: u16,
    wave_index: u8,
    length: u16,
}

impl SoundChannel3 {
    fn new() -> SoundChannel3 {
        SoundChannel3 {
            enabled: false,
            frequency_timer: 0,
            frame_sequence: 8,
            frame_sequence_timer: 8192,
            wave_index: 0,
            length: 0,
        }
    }
}

impl SoundChannel for SoundChannel3 {
    fn reset(&mut self, memory: &Memory, length: u8) {
        let nr33 = memory.read_from_memory(0xff1d);
        let nr34 = memory.read_from_memory(0xff1e);
        let new_frequency = ((nr34 as u16) & 0b111) << 8 | (nr33 as u16);
        self.frequency_timer = (2048 - new_frequency) * 2;
        if self.length == 0 { self.length = 256 - (length as u16); }
        self.enabled = true;
        self.wave_index = 0;
        if memory.read_from_memory(0xff1a) >> 6 == 0 { self.enabled = false; }
    }

    fn update(&mut self, memory: &mut Memory) {
        let nr33 = memory.read_from_memory(0xff1d);
        let nr34 = memory.read_from_memory(0xff1e);
        if self.frequency_timer == 0 {
            let new_frequency_timer = (((nr34 as u16) & 0b111) << 8) | (nr33 as u16);
            self.frequency_timer = (2048 - new_frequency_timer) * 2;
            self.wave_index += 1;
            self.wave_index %= 32;
        } else {
            self.frequency_timer = self.frequency_timer.wrapping_sub(1);
        }

        self.frame_sequence_timer = self.frame_sequence_timer.wrapping_sub(1);
        if self.frame_sequence_timer == 0 {
            self.frame_sequence_timer = 8192;
            self.frame_sequence += 1;
            self.frame_sequence &= 8;

            if self.frame_sequence % 2 == 0 && bit_logic::check_bit(nr34, 6)  && self.length != 0 {
                self.length = self.length.wrapping_sub(1);
                if self.length == 0 { self.enabled = false; }
            }
        }
    }

    fn get_amplitude(&mut self, memory: &Memory) -> f32 {
        if self.enabled {
            let mut wave = memory.read_from_memory(0xff30 + ((self.wave_index as u16) / 2));
            wave = if self.wave_index % 2 != 0 {
                wave & 0xf
            } else {
                wave >> 4
            };
            let volume = (memory.read_from_memory(0xff1c) >> 5) & 0b11;
            wave = if volume != 0 {
                wave >> (volume - 1)
            } else {
                wave >> 4
            };
            if memory.read_from_memory(0xff1a) >> 7 != 0 {
                (wave as f32) / 100.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

pub(crate) struct SoundChannel4 {
    enabled: bool,
    amplitude: i16,
    frequency_timer: u16,
    frame_sequence: u8,
    frame_sequence_timer: u16,
    envelope_enabled: bool,
    envelope_sweeps: u8,
    length: u8,
    lfsr: u16,
}

impl SoundChannel4 {
    fn new() -> SoundChannel4 {
        SoundChannel4 {
            enabled: false,
            amplitude: 0,
            frequency_timer: 0,
            frame_sequence: 7,
            frame_sequence_timer: 8192,
            envelope_enabled: false,
            envelope_sweeps: 0,
            length: 0,
            lfsr: 0,
        }
    }
}

impl SoundChannel for SoundChannel4 {
    fn reset(&mut self, memory: &Memory, length: u8) {
        let nr42 = memory.read_from_memory(0xff21);
        let nr43 = memory.read_from_memory(0xff22);
        if self.length == 0 { self.length = 64 - length; }
        self.enabled = true;
        self.frequency_timer = (SOUND_CHANNEL_4_DIVISOR[(nr43 as usize) & 0b111] as u16) << ((nr43 as u16) >> 4);
        self.lfsr = 0x7fff;
        self.amplitude = nr42 as i16 >> 4;
        self.envelope_sweeps = nr42 & 0b111;
        self.envelope_enabled = true;
        if nr42 >> 3 == 0 { self.enabled = false; }
    }

    fn update(&mut self, memory: &mut Memory) {
        let nr42 = memory.read_from_memory(0xff21);
        let nr43 = memory.read_from_memory(0xff22);
        let nr44 = memory.read_from_memory(0xff23);

        self.frame_sequence_timer = self.frame_sequence_timer.wrapping_sub(1);
        if self.frame_sequence_timer == 0 {
            self.frame_sequence_timer = 8192;
            self.frame_sequence += 1;
            self.frame_sequence &= 8;

            if self.frame_sequence % 2 == 0 && bit_logic::check_bit(nr44, 6)  && self.length != 0 {
                self.length = self.length.wrapping_sub(1);
                if self.length == 0 { self.enabled = false; }
            }

            if self.frame_sequence == 7 && self.envelope_enabled && (nr42 & 0b111) != 0 {
                self.envelope_sweeps = self.envelope_sweeps.wrapping_sub(1);
                if self.envelope_sweeps == 0 {
                    self.envelope_sweeps = nr42 & 0b111;
                    if self.envelope_sweeps != 0 {
                        let new_amplitude = self.amplitude + if bit_logic::check_bit(nr42, 3) { 1 } else { -1 };
                        if new_amplitude >= 0 && new_amplitude <= 15 {
                            self.amplitude = new_amplitude;
                        } else {
                            self.envelope_enabled = false;
                        }
                    }
                }
            }
        }

        if self.frequency_timer == 0 {
            self.frequency_timer = (SOUND_CHANNEL_4_DIVISOR[(nr43 as usize) & 0b111] as u16) << ((nr43 as u16) >> 4);
            let xor_rs = (self.lfsr & 1) ^ ((self.lfsr & 0b10) >> 1);
            self.lfsr >>= 1;
            self.lfsr |= xor_rs << 14;
            if bit_logic::check_bit(nr43, 3) {
                self.lfsr |= xor_rs << 6;
                self.lfsr &= 0x7f;
            }
        } else {
            self.frequency_timer = self.frequency_timer.wrapping_sub(1);
        }
    }

    fn get_amplitude(&mut self, _memory: &Memory) -> f32 {
        if self.enabled {
            if (self.lfsr & 1) != 0 {
                0.0
            } else {
                (self.amplitude as f32) / 100.0
            }
        } else {
            0.0
        }
    }
}