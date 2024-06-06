use super::TIME_BETWEEN_AUDIO_SAMPLING;
use super::bit_logic;
use super::gameboy::{Gameboy, SoundChannel1, SoundChannel2, SoundChannel3, SoundChannel4};

const WAVE_FORM: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1,],
    [1, 0, 0, 0, 0, 0, 0, 1,],
    [1, 0, 0, 0, 0, 1, 1, 1,],
    [0, 1, 1, 1, 1, 1, 1, 0,],
];

const SOUND_CHANNEL_4_DIVISOR: [u8; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

impl Gameboy {
    pub(crate) fn update_audio(&mut self, cycles: u8) {
        let nr10 = self.read_from_memory(0xff10);
        let nr11 = self.read_from_memory(0xff11);
        let nr12 = self.read_from_memory(0xff12);
        let nr21 = self.read_from_memory(0xff16);
        let nr22 = self.read_from_memory(0xff17);
        let nr23 = self.read_from_memory(0xff18);
        let nr24 = self.read_from_memory(0xff19);
        let nr30 = self.read_from_memory(0xff1a);
        let nr32 = self.read_from_memory(0xff1c);
        let nr33 = self.read_from_memory(0xff1d);
        let nr34 = self.read_from_memory(0xff1e);
        let nr42 = self.read_from_memory(0xff21);
        let nr43 = self.read_from_memory(0xff22);
        let nr44 = self.read_from_memory(0xff23);
        let mut nr50: u8;
        let mut channel_1: f32;
        let mut channel_2: f32;
        let mut channel_3: f32;
        let mut channel_4: f32;
        let mut nr51: u8;
        let mut left_results: f32;
        let mut right_results: f32;

        for _ in 0..cycles {
            self.update_sound_channel_1(&nr10, &nr11, &nr12);
            self.update_sound_channel_2(&nr22, &nr23, &nr24);
            self.update_sound_channel_3(&nr33, &nr34);
            self.update_sound_channel_4(&nr42, &nr43, &nr44);

            if self.audio_fill_timer == 0 {
                self.audio_fill_timer = TIME_BETWEEN_AUDIO_SAMPLING;
                let (_enable_left_vin, left_volume, _enable_right_vin, right_volume) = {
                    nr50 = self.read_from_memory(0xff24);
                    (
                        nr50 & 0x80 != 0,
                        (nr50 & 0x70) >> 4,
                        nr50 & 0x8 != 0,
                        nr50 & 0x7
                    )
                };
                channel_1 = self.get_amplitude_sound_channel_1();
                channel_2 = self.get_amplitude_sound_channel_2(&nr21);
                channel_3 = self.get_amplitude_sound_channel_3(&nr30, &nr32);
                channel_4 = self.get_amplitude_sound_channel_4();
                nr51 = self.read_from_memory(0xff25);
                if nr51 != 0 {
                    left_results = 0.0;
                    left_results += if bit_logic::check_bit(nr51, 4) { channel_1 } else { 0.0 };
                    left_results += if bit_logic::check_bit(nr51, 5) { channel_2 } else { 0.0 };
                    left_results += if bit_logic::check_bit(nr51, 6) { channel_3 } else { 0.0 };
                    left_results += if bit_logic::check_bit(nr51, 7) { channel_4 } else { 0.0 };
                    left_results *= left_volume as f32 / 7.0;
                    self.audio_data.push(left_results);
                    right_results = 0.0;
                    right_results += if bit_logic::check_bit(nr51, 0) { channel_1 } else { 0.0 };
                    right_results += if bit_logic::check_bit(nr51, 1) { channel_2 } else { 0.0 };
                    right_results += if bit_logic::check_bit(nr51, 2) { channel_3 } else { 0.0 };
                    right_results += if bit_logic::check_bit(nr51, 3) { channel_4 } else { 0.0 };
                    right_results *= right_volume as f32 / 7.0;
                    self.audio_data.push(right_results);
                } else {
                    self.audio_data.push(0.0);
                    self.audio_data.push(0.0);
                }
            } else {
                self.audio_fill_timer -= 1;
            }
        }
    }

    pub(crate) fn reset_sound_channel_1(&mut self, length: u8) {
        let nr10: u8 = self.read_from_memory(0xff10);
        let nr12: u8 = self.read_from_memory(0xff12);
        let nr13: u8 = self.read_from_memory(0xff13);
        let nr14: u8 = self.read_from_memory(0xff14);
        if self.sound_channel_1.length == 0 { self.sound_channel_1.length = 64 - length; }
        self.sound_channel_1.enabled = true;
        self.sound_channel_1.amplitude = (nr12 >> 4) as i16;
        self.sound_channel_1.envelope_sweeps = nr12 & 0b111;
        self.sound_channel_1.envelope_enabled = true;

        let new_frequency_timer = (nr14 as u16) & 0b111 << 8 | (nr13 as u16);
        self.sound_channel_1.frequency_timer = 8192 - (new_frequency_timer * 4);

        self.sound_channel_1.sweep_period = (nr10 >> 4) & 0b111;
        let sweep_shift = nr10 & 0b111;
        let sweep_negate = if bit_logic::check_bit(nr10, 3) { -1 } else { 1 };
        self.sound_channel_1.sweep_enabled = self.sound_channel_1.sweep_period != 0 && sweep_shift != 0;
        self.sound_channel_1.sweep_shadow = ((nr14 as i16) & 0b111) << 8 | (nr13 as i16);
        if sweep_shift != 0 && (self.sound_channel_1.sweep_shadow as i32) + ((self.sound_channel_1.sweep_shadow as i32) >> sweep_shift) * sweep_negate > 2047 {
            self.sound_channel_1.sweep_enabled = false;
            self.sound_channel_1.enabled = false;
        }
        if nr12 >> 3 == 0 { self.sound_channel_1.enabled = false; }
    }

    fn update_sound_channel_1(&mut self, nr10: &u8, nr11: &u8, nr12: &u8) {
        let duty = nr11 >> 6;
        let nr13 = self.read_from_memory(0xff13);
        let nr14 = self.read_from_memory(0xff14);

        self.sound_channel_1.frame_sequence_timer -= 1;
        if self.sound_channel_1.frame_sequence_timer == 0 {
            // TODO: self.sound_channel_1.frame_sequence_timer = (8192 + 1) & 8; THIS EQUALS 0
            self.sound_channel_1.frame_sequence_timer = 8192;
            if self.sound_channel_1.frame_sequence % 2 == 0 && bit_logic::check_bit(nr14, 6) && self.sound_channel_1.length != 0 {
                self.sound_channel_1.length -= 1;
                if self.sound_channel_1.length == 0 { self.sound_channel_1.enabled = false; }
            }
        }

        let nr10_and_b111 = nr10 & 0b111;
        let nr10_shift_right_4_and_b111 = (nr10 >> 4) & 0b111;
        if (self.sound_channel_1.frame_sequence == 2 || self.sound_channel_1.frame_sequence == 6) && self.sound_channel_1.frame_sequence_timer == 8192 && nr10_shift_right_4_and_b111 != 0 && nr10_and_b111 != 0 {
            self.sound_channel_1.sweep_period -= 1;
            if self.sound_channel_1.sweep_period == 0 {
                self.sound_channel_1.sweep_period = nr10_shift_right_4_and_b111;
                if self.sound_channel_1.sweep_period == 0 { self.sound_channel_1.sweep_period = 8; }
                if nr10_shift_right_4_and_b111 != 0 && self.sound_channel_1.sweep_enabled && nr10_and_b111 != 0 {
                    let sweep_shift = nr10_and_b111;
                    let sweep_negate = if bit_logic::check_bit(*nr10, 3) { -1 } else { 1 };
                    let new_frequency = self.sound_channel_1.sweep_shadow + (self.sound_channel_1.sweep_shadow >> sweep_shift) * sweep_negate;
                    if new_frequency < 2048 && sweep_shift != 0 {
                        self.sound_channel_1.sweep_shadow = new_frequency;
                        self.write_to_memory(0xff13, (self.sound_channel_1.sweep_shadow & 0xff) as u8);
                        self.write_to_memory(0xff14, ((self.sound_channel_1.sweep_shadow >> 8) & 0b111) as u8);
                        if self.sound_channel_1.sweep_shadow + (self.sound_channel_1.sweep_shadow >> sweep_shift) * sweep_negate > 2047 {
                            self.sound_channel_1.enabled = false;
                            self.sound_channel_1.sweep_enabled = false;
                        }
                    }
                    if new_frequency > 2047 {
                        self.sound_channel_1.enabled = false;
                        self.sound_channel_1.sweep_enabled = false;
                    }
                    if self.sound_channel_1.sweep_shadow + (self.sound_channel_1.sweep_shadow >> sweep_shift) * sweep_negate > 2047 {
                        self.sound_channel_1.enabled = false;
                        self.sound_channel_1.sweep_enabled = false;
                    }
                }
            }
        }

        if self.sound_channel_1.frequency_timer == 0 {
            self.sound_channel_1.frequency_timer = (2048 - ((((nr14 as u16) & 0b111) << 8) | (nr13 as u16))) * 4;
            self.sound_channel_1.wave_duty_position = (self.sound_channel_1.wave_duty_position + 1) % 8;
        } else {
            self.sound_channel_1.frequency_timer -= 1;
        }

        self.sound_channel_1.frequency = if WAVE_FORM[duty as usize][self.sound_channel_1.wave_duty_position as usize] == 1 {
            self.sound_channel_1.amplitude
        } else {
            0
        };
        
        if self.sound_channel_1.frame_sequence == 7 && self.sound_channel_1.frame_sequence_timer == 8192 && self.sound_channel_1.envelope_enabled && (nr12 & 0b111) != 0 {
            self.sound_channel_1.envelope_sweeps -= 1;
            if self.sound_channel_1.envelope_sweeps == 0 {
                self.sound_channel_1.envelope_sweeps = nr12 & 0b111;
                let new_amplitude = self.sound_channel_1.amplitude + if bit_logic::check_bit(*nr12, 3) { 1 } else { -1 };
                if new_amplitude > 0 && new_amplitude <= 15 {
                    self.sound_channel_1.amplitude = new_amplitude;
                    self.sound_channel_1.frequency = self.sound_channel_1.amplitude;
                } else {
                    self.sound_channel_1.envelope_enabled = false;
                }
            }
        }
    }

    fn get_amplitude_sound_channel_1(&self) -> f32 {
        if self.sound_channel_1.enabled {
            self.sound_channel_1.frequency as f32 / 100.0
        } else {
            0.0
        }
    }

    pub(crate) fn reset_sound_channel_2(&mut self, length: u8) {
        let nr22: u8 = self.read_from_memory(0xff17);
        if self.sound_channel_2.length == 0 { self.sound_channel_2.length = 64 - length; }
        self.sound_channel_2.enabled = true;
        self.sound_channel_2.amplitude = (nr22 >> 4) as i16;
        self.sound_channel_2.envelope_sweeps = nr22 & 0b111;
        self.sound_channel_2.envelope_enabled = true;
        if nr22 >> 3 == 0 { self.sound_channel_2.enabled = false; }
    }

    fn update_sound_channel_2(&mut self, nr22: &u8, nr23: &u8, nr24: &u8) {
        if self.sound_channel_2.frequency_timer == 0 {
            let new_frequency_timer = (((*nr24 as u16) & 0b111) << 8) | (*nr23 as u16);
            self.sound_channel_2.frequency_timer = (2048 - new_frequency_timer) * 4;
            self.sound_channel_2.wave_duty_position = (self.sound_channel_2.wave_duty_position + 1) % 8;
        } else {
            self.sound_channel_2.frequency_timer -= 1;
        }

        self.sound_channel_2.frame_sequence_timer -= 1;
        if self.sound_channel_2.frame_sequence_timer == 0 {
            self.sound_channel_2.frame_sequence_timer = 8192;
            self.sound_channel_2.frame_sequence = (self.sound_channel_2.frame_sequence + 1) & 8;

            if self.sound_channel_2.frame_sequence % 2 == 0 && bit_logic::check_bit(*nr24, 6) && self.sound_channel_2.length != 0 {
                self.sound_channel_2.length -= 1;
                if self.sound_channel_2.length == 0 { self.sound_channel_2.enabled = false; }
            }

            if self.sound_channel_2.frame_sequence == 7 && self.sound_channel_2.envelope_enabled && (nr22 & 0b111) != 0 {
                self.sound_channel_2.envelope_sweeps -= 1;
                if self.sound_channel_2.envelope_sweeps == 0 {
                    self.sound_channel_2.envelope_sweeps = nr22 & 0b111;
                    let new_amplitude = self.sound_channel_2.amplitude + if bit_logic::check_bit(*nr22, 3) { 1 } else { -1 };
                    if new_amplitude > 0 && new_amplitude <= 15 {
                        self.sound_channel_2.amplitude = new_amplitude;
                    } else {
                        self.sound_channel_2.envelope_enabled = false;
                    }
                }
            }
        }
    }

    fn get_amplitude_sound_channel_2(&self, nr21: &u8) -> f32 {
        if self.sound_channel_2.enabled {
            if WAVE_FORM[(nr21 >> 6) as usize][self.sound_channel_2.wave_duty_position as usize] == 1 {
                (self.sound_channel_2.amplitude as f32) / 100.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    pub(crate) fn reset_sound_channel_3(&mut self, length: u8) {
        let nr33 = self.read_from_memory(0xff1d);
        let nr34 = self.read_from_memory(0xff1e);
        let new_frequency = ((nr34 as u16) & 0b111) << 8 | (nr33 as u16);
        self.sound_channel_3.frequency_timer = (2048 - new_frequency) * 2;
        if self.sound_channel_3.length == 0 { self.sound_channel_3.length = 256 - (length as u16); }
        self.sound_channel_3.enabled = true;
        self.sound_channel_3.wave_index = 0;
        if self.read_from_memory(0xff1a) >> 6 == 0 { self.sound_channel_3.enabled = false; }
    }

    fn update_sound_channel_3(&mut self, nr33: &u8, nr34: &u8) {
        if self.sound_channel_3.frequency_timer == 0 {
            let new_frequency_timer = (((*nr34 as u16) & 0b111) << 8) | (*nr33 as u16);
            self.sound_channel_3.frequency_timer = (2048 - new_frequency_timer) * 2;
            self.sound_channel_3.wave_index = (self.sound_channel_3.wave_index + 1) % 32;
        } else {
            self.sound_channel_3.frequency_timer -= 1;
        }

        self.sound_channel_3.frame_sequence_timer -= 1;
        if self.sound_channel_3.frame_sequence_timer == 0 {
            self.sound_channel_3.frame_sequence_timer = 8192;
            self.sound_channel_3.frame_sequence = (self.sound_channel_3.frame_sequence + 1) & 8;

            if self.sound_channel_3.frame_sequence % 2 == 0 && bit_logic::check_bit(*nr34, 6)  && self.sound_channel_3.length != 0 {
                self.sound_channel_3.length -= 1;
                if self.sound_channel_3.length == 0 { self.sound_channel_3.enabled = false; }
            }
        }
    }

    fn get_amplitude_sound_channel_3(&self, nr30: &u8, nr32: &u8) -> f32 {
        if self.sound_channel_3.enabled {
            let mut wave = self.read_from_memory(0xff30 + ((self.sound_channel_3.wave_index as u16) / 2));
            wave = if self.sound_channel_3.wave_index % 2 != 0 {
                wave & 0xf
            } else {
                wave >> 4
            };
            let volume = (nr32 >> 5) & 0b11;
            wave >>= if volume != 0 {
                volume - 1
            } else {
                4
            };
            if nr30 >> 7 != 0 {
                (wave as f32) / 100.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    pub(crate) fn reset_sound_channel_4(&mut self, length: u8) {
        let nr42 = self.read_from_memory(0xff21);
        let nr43 = self.read_from_memory(0xff22);
        if self.sound_channel_4.length == 0 { self.sound_channel_4.length = 64 - length; }
        self.sound_channel_4.enabled = true;
        self.sound_channel_4.frequency_timer = (SOUND_CHANNEL_4_DIVISOR[(nr43 as usize) & 0b111] as u16) << ((nr43 as u16) >> 4);
        self.sound_channel_4.lfsr = 0x7fff;
        self.sound_channel_4.amplitude = nr42 as i16 >> 4;
        self.sound_channel_4.envelope_sweeps = nr42 & 0b111;
        self.sound_channel_4.envelope_enabled = true;
        if nr42 >> 3 == 0 { self.sound_channel_4.enabled = false; }
    }

    fn update_sound_channel_4(&mut self, nr42: &u8, nr43: &u8, nr44: &u8) {
        self.sound_channel_4.frame_sequence_timer -= 1;
        if self.sound_channel_4.frame_sequence_timer == 0 {
            self.sound_channel_4.frame_sequence_timer = 8192;
            self.sound_channel_4.frame_sequence = (self.sound_channel_4.frame_sequence + 1) & 8;

            if self.sound_channel_4.frame_sequence % 2 == 0 && bit_logic::check_bit(*nr44, 6) && self.sound_channel_4.length != 0 {
                self.sound_channel_4.length -= 1;
                if self.sound_channel_4.length == 0 { self.sound_channel_4.enabled = false; }
            }

            if self.sound_channel_4.frame_sequence == 7 && self.sound_channel_4.envelope_enabled && (nr42 & 0b111) != 0 {
                self.sound_channel_4.envelope_sweeps -= 1;
                if self.sound_channel_4.envelope_sweeps == 0 {
                    self.sound_channel_4.envelope_sweeps = nr42 & 0b111;
                    if self.sound_channel_4.envelope_sweeps != 0 {
                        let new_amplitude = self.sound_channel_4.amplitude + if bit_logic::check_bit(*nr42, 3) { 1 } else { -1 };
                        if new_amplitude > 0 && new_amplitude <= 15 {
                            self.sound_channel_4.amplitude = new_amplitude;
                        } else {
                            self.sound_channel_4.envelope_enabled = false;
                        }
                    }
                }
            }
        }

        if self.sound_channel_4.frequency_timer == 0 {
            self.sound_channel_4.frequency_timer = (SOUND_CHANNEL_4_DIVISOR[(*nr43 as usize) & 0b111] as u16) << ((*nr43 as u16) >> 4);
            let xor_rs = (self.sound_channel_4.lfsr & 1) ^ ((self.sound_channel_4.lfsr & 0b10) >> 1);
            self.sound_channel_4.lfsr = (self.sound_channel_4.lfsr >> 1) | (xor_rs << 14);
            if bit_logic::check_bit(*nr43, 3) {
                self.sound_channel_4.lfsr = (self.sound_channel_4.lfsr | (xor_rs << 6)) & 0x7f;
            }
        } else {
            self.sound_channel_4.frequency_timer -= 1;
        }
    }

    fn get_amplitude_sound_channel_4(&self) -> f32 {
        if self.sound_channel_4.enabled {
            if (self.sound_channel_4.lfsr & 1) != 0 {
                0.0
            } else {
                (self.sound_channel_4.amplitude as f32) / 100.0
            }
        } else {
            0.0
        }
    }
}

impl SoundChannel1 {
    pub(crate) fn new() -> SoundChannel1 {
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

impl SoundChannel2 {
    pub(crate) fn new() -> SoundChannel2 {
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

impl SoundChannel3 {
    pub(crate) fn new() -> SoundChannel3 {
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

impl SoundChannel4 {
    pub(crate) fn new() -> SoundChannel4 {
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