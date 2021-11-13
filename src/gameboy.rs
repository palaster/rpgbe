use crate::{WIDTH, SCREEN_DATA_SIZE, CYCLES_PER_SECOND};
use crate::cpu::CPU;
use crate::memory::Memory;
use crate::bit_logic;

use std::sync::{Arc, Mutex};

pub const TIMA: u16 = 0xff05;
const TMA: u16 = 0xff06;
pub const TAC: u16 = 0xff07;

const VERTICAL_BLANK_SCAN_LINE: u16 = 144;
const VERTICAL_BLANK_SCAN_LINE_MAX: u16 = 153;
const SCANLINE_COUNTER_START: u16 = 456;

#[derive(Copy, Clone)]
enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

#[derive(Clone)]
pub struct Gameboy {
    scanline_counter: i32,
    timer_counter: i32,
    pub divider_counter: i32,
    halt_bug: bool,
    ei_halt_bug: bool,
    gamepad_state: u8,
    pub screen_data: [u8; SCREEN_DATA_SIZE as usize],
    scanline_bg: [bool; WIDTH as usize],
    cpu: CPU,
    memory: Arc<Mutex<Memory>>,
}

impl Gameboy {
    pub fn new(memory: Arc<Mutex<Memory>>) -> Gameboy {
        Gameboy {
            scanline_counter: SCANLINE_COUNTER_START as i32,
            timer_counter: 0,
            divider_counter: 0,
            halt_bug: false,
            ei_halt_bug: false,
            gamepad_state: 0xff,
            screen_data: [0; SCREEN_DATA_SIZE as usize],
            scanline_bg: [false; WIDTH as usize],
            cpu: CPU::new(memory.clone()),
            memory: memory,
        }
    }

    pub fn update(&mut self) -> f64 {
        let mut cycles: f64 = 4.0;
        if !self.cpu.is_halted() { cycles = self.cpu.update(); }
        /*
        {
            let mut memory = self.memory.lock().unwrap();
            if memory.rom[0xff02] == 0x81 {
                let c: char = memory.rom[0xff01] as char;
                print!("{}", c);
                memory.rom[0xff02] = 0x0;
            }
        }
        */
        /*
        self.update_timer(cycles);
        self.update_graphics(cycles);
        cycles += self.do_interrupts();
        */
        cycles
    }

    fn is_clock_enabled(&self) -> bool {
        let memory = self.memory.lock().unwrap();
        bit_logic::check_bit(memory.read_from_memory(None, TAC), 2)
    }

    pub fn get_clock_freq(&self) -> u8 {
        let memory = self.memory.lock().unwrap();
        memory.read_from_memory(None, TAC) & 0x3
    }

    pub fn set_clock_freq(&mut self) {
        let freq: u8 = self.get_clock_freq();
        match freq {
            0 => { self.timer_counter = CYCLES_PER_SECOND as i32 / 4096 },
            1 => { self.timer_counter = CYCLES_PER_SECOND as i32 / 262144 },
            2 => { self.timer_counter = CYCLES_PER_SECOND as i32 / 65536 },
            3 => { self.timer_counter = CYCLES_PER_SECOND as i32 / 16382 },
            _ => { },
        };
    }

    fn do_divider_register(&mut self, cycles: f64) {
        self.divider_counter += cycles as i32;
        if self.divider_counter >= 255 {
            self.divider_counter = 0;
            let mut memory = self.memory.lock().unwrap();
            memory.rom[0xff04] = memory.rom[0xff04].wrapping_add(1);
        }
    }

    fn update_timer(&mut self, cycles: f64) {
        self.do_divider_register(cycles);
        if self.is_clock_enabled() {
            self.timer_counter -= cycles as i32;
            if self.timer_counter <= 0 {
                self.set_clock_freq();
                let (tima, tma): (u8, u8) = {
                    let memory = self.memory.lock().unwrap();
                    (memory.read_from_memory(None, TIMA), memory.read_from_memory(None, TMA))
                };
                if tima == 255 {
                    {
                        let mut memory = self.memory.lock().unwrap();
                        memory.write_to_memory(None, TIMA, tma);
                    }
                    self.request_interrupt(2);
                } else {
                    let mut memory = self.memory.lock().unwrap();
                    memory.write_to_memory(None, TIMA, tima + 1);
                }
            }
        }
    }

    fn is_lcd_enabled(&self) -> bool {
        let memory = self.memory.lock().unwrap();
        bit_logic::check_bit(memory.read_from_memory(None, 0xff40), 7)
    }

    fn set_lcd_status(&mut self) {
        let mut status: u8 = {
            let memory = self.memory.lock().unwrap();
            memory.read_from_memory(None, 0xff41)
        };
        if !self.is_lcd_enabled() {
            let mut memory = self.memory.lock().unwrap();
            self.scanline_counter = SCANLINE_COUNTER_START as i32;
            memory.rom[0xff44] = 0;
            status &= 252;
            status = bit_logic::set_bit(status, 0);
            memory.write_to_memory(None, 0xff41, status);
            return;
        }
        let current_line: u8 = {
            let memory = self.memory.lock().unwrap();
            memory.read_from_memory(None, 0xff44)
        };
        let current_mode: u8 = status & 0x3;
        let mode: u8;
        let mut req_int: bool = false;

        if current_line >= 144 {
            mode = 1;
            status = bit_logic::set_bit(status, 0);
            status = bit_logic::reset_bit(status, 1);
            req_int = bit_logic::check_bit(status, 4);
        } else {
            let mode_2_bounds: i32 = 456 - 80;
            let mode_3_bounds: i32 = mode_2_bounds - 172;
            if self.scanline_counter >= mode_2_bounds {
                mode = 2;
                status = bit_logic::set_bit(status, 1);
                status = bit_logic::reset_bit(status, 0);
                req_int = bit_logic::check_bit(status, 5);
            } else if self.scanline_counter >= mode_3_bounds {
                mode = 3;
                status = bit_logic::set_bit(status, 1);
                status = bit_logic::set_bit(status, 0);
            } else {
                mode = 0;
                status = bit_logic::reset_bit(status, 1);
                status = bit_logic::reset_bit(status, 0);
                req_int = bit_logic::check_bit(status, 3);
            }
        }
        if req_int && current_mode != mode {
            self.request_interrupt(1);
        }
        if { let memory = self.memory.lock().unwrap(); current_line == memory.read_from_memory(None, 0xff45) } {
            status = bit_logic::set_bit(status, 2);
            if bit_logic::check_bit(status, 6) {
                self.request_interrupt(1);
            }
        } else {
            status = bit_logic::reset_bit(status, 2);
        }
        let mut memory = self.memory.lock().unwrap();
        memory.write_to_memory(None, 0xff41, status);
    }

    fn get_color(&self, address: u16, color_num: u8) -> Color {
        let memory = self.memory.lock().unwrap();
        let palette: u8 = memory.read_from_memory(None, address);

        let (hi, lo) = match color_num {
            0 => (1, 0),
            1 => (3, 2),
            2 => (5, 4),
            3 => (7, 6),
            _ => (0, 0),
        };

        let mut color: i32 = (bit_logic::bit_value(palette, hi) << 1) as i32;
        color |= bit_logic::bit_value(palette, lo) as i32;
        
        match color {
            0 => Color::White,
            1 => Color::LightGray,
            2 => Color::DarkGray,
            3 => Color::Black,
            _ => Color::White,
        }
    }

    fn render_tiles(&mut self) {
        let mut unsig: bool = true;
        let (lcd_control, scroll_x, scroll_y, window_y, window_x): (u8, u8, u8, u8, u8) = {
            let memory = self.memory.lock().unwrap();
            (memory.read_from_memory(None, 0xff40), memory.read_from_memory(None, 0xff42), memory.read_from_memory(None, 0xff43), memory.read_from_memory(None, 0xff4a), memory.read_from_memory(None, 0xff4b).wrapping_sub(7))
        };

        let mut using_window: bool = false;

        if bit_logic::check_bit(lcd_control, 5) {
            if window_y <= { let memory = self.memory.lock().unwrap(); memory.read_from_memory(None, 0xff44) } {
                using_window = true;
            }
        }

        let tile_data: u16 = if bit_logic::check_bit(lcd_control, 4) {
            0x8000
        } else {
            unsig = false;
            0x8800
        };

        let background_memory: u16 = if !using_window {
            if bit_logic::check_bit(lcd_control, 3) {
                0x9c00
            } else {
                0x9800
            }
        } else {
            if bit_logic::check_bit(lcd_control, 6) {
                0x9c00
            } else {
                0x9800
            }
        };

        let y_pos: u8 = if !using_window {
            let memory = self.memory.lock().unwrap();
            scroll_y.wrapping_add(memory.read_from_memory(None, 0xff44))
        } else {
            let memory = self.memory.lock().unwrap();
            memory.read_from_memory(None, 0xff44) - window_y
        };
        let tile_row: u16 = (y_pos / 8).wrapping_mul(32) as u16;
        for pixel in 0..WIDTH {
            let mut x_pos: u8 = (pixel + scroll_x as u16) as u8;
            if using_window {
                if pixel >= (window_x as u16) {
                    x_pos = (pixel - window_x as u16) as u8;
                }
            }
            let tile_column: u16 = x_pos as u16 / 8;
            //let mut tile_num: i16 = 0;
            let tile_address: u16 = background_memory + tile_row + tile_column;
            let tile_num: i16 = {
                let memory = self.memory.lock().unwrap();
                memory.read_from_memory(None, tile_address) as i16
            };
            /*
            if unsig {
                tile_num = memory.read_from_memory(None, tile_address) as i16;
            } else {
                tile_num = memory.read_from_memory(None, tile_address) as i16;
            }
            */
            let mut tile_location: u16 = tile_data;
            if unsig {
                tile_location += (tile_num * 16) as u16;
            } else {
                tile_location += ((tile_num + 128) * 16) as u16;
            }
            let mut line: u8 = y_pos % 8;
            line *= 2;
            let (data_1, data_2): (u8, u8) = {
                let memory = self.memory.lock().unwrap();
                let temp_address: u16 = tile_location + line as u16;
                (memory.read_from_memory(None, temp_address), memory.read_from_memory(None, temp_address + 1))
            };

            let mut color_bit: i32 = x_pos as i32 % 8;
            color_bit -= 7;
            color_bit *= -1;
            let mut color_num: u8 = bit_logic::bit_value(data_2, color_bit as u8);
            color_num <<= 1; 
            color_num |= bit_logic::bit_value(data_1, color_bit as u8);

            let color: Color = self.get_color(0xff47, color_num);
            let (red, green, blue): (u8, u8, u8) = match color {
                Color::White => (255, 255, 255),
                Color::DarkGray => (0xcc, 0xcc, 0xcc),
                Color::LightGray => (0x77, 0x77, 0x77),
                _ => (0, 0, 0),
            };

            let memory = self.memory.lock().unwrap();
            let finally: u8 = memory.read_from_memory(None, 0xff44);
            if finally > 143 || pixel > 159 {
                continue;
            }

            self.scanline_bg[pixel as usize] = matches!(color, Color::White);

            let x: u16 = (finally as u16).wrapping_mul(WIDTH).wrapping_mul(3);
            let y: u16 = pixel.wrapping_mul(3);
            let xy: u16 = x.wrapping_add(y);
            self.screen_data[xy as usize] = red;
            self.screen_data[xy.wrapping_add(1) as usize] = green;
            self.screen_data[xy.wrapping_add(2) as usize] = blue;
        }
    }

    fn render_sprites(&mut self) {
        let memory = self.memory.lock().unwrap();
        let use_8x16: bool = bit_logic::check_bit(memory.read_from_memory(None, 0xff40), 2);
        for sprite in 0..40 {
            let index: u8 = sprite * 4;
            let temp_address: u16 = 0xfe00 + index as u16;
            let y_pos: u8 = memory.read_from_memory(None, temp_address) - 16;
            let x_pos: u8 = memory.read_from_memory(None, temp_address + 1) - 8;
            let tile_location: u8 = memory.read_from_memory(None, temp_address + 2);
            let attributes: u8 = memory.read_from_memory(None, temp_address + 3);
            
            let y_flip: bool = bit_logic::check_bit(attributes, 6);
            let x_flip: bool = bit_logic::check_bit(attributes, 5);
            let priority: bool = !bit_logic::check_bit(attributes, 7);
            let scanline: i32 = memory.read_from_memory(None, 0xff44) as i32;

            let y_size: i32 = if use_8x16 { 16 } else { 8 };

            if (scanline >= y_pos as i32) && (scanline < (y_pos as i32 + y_size)) {
                
                let mut line: i32 = scanline - y_pos as i32;

                if y_flip  {
                    line -= y_size;
                    line *= -1;
                }

                line *= 2;
                let data_address: u16 = (0x8000 + (tile_location as u16 * 16)) + (line as u16);
                let data_1: u8 = memory.read_from_memory(None, data_address);
                let data_2: u8 = memory.read_from_memory(None, data_address + 1);
                for tile_pixel in (0..=7).rev() {
                    let mut color_bit: i16 = tile_pixel;
                    if x_flip {
                        color_bit = color_bit.wrapping_sub(7);
                        color_bit = color_bit.wrapping_mul(-1);
                    }
                    let mut color_num: u8 = bit_logic::bit_value(data_2, color_bit as u8);
                    color_num <<= 1;
                    color_num |= bit_logic::bit_value(data_1, color_bit as u8);
                    
                    let color_address: u16 = if bit_logic::check_bit(attributes, 4) { 0xff49 } else { 0xff48 };
                    let color: Color = self.get_color(color_address, color_num);

                    if matches!(color, Color::White) {
                        continue;
                    }
                    
                    let (red, green, blue): (u8, u8, u8) = match color {
                        Color::White => (255, 255, 255),
                        Color::LightGray => (0xcc, 0xcc, 0xcc),
                        Color::DarkGray => (0x77, 0x77, 0x77),
                        _ => (0, 0, 0),
                    };

                    let mut x_pix: i16 = 0 - tile_pixel;
                    x_pix += 7;

                    let pixel: i32 = (x_pos + x_pix as u8) as i32;
                    if (scanline < 0) || (scanline > 143) || (pixel < 0) || (pixel > 159) {
                        continue;
                    }

                    if self.scanline_bg[pixel as usize] || priority {
                        let x: u16 = (scanline as u16).wrapping_mul(WIDTH).wrapping_mul(3);
                        let y: u16 = pixel.wrapping_mul(3) as u16;
                        let xy: u16 = x.wrapping_add(y);
                        self.screen_data[xy as usize] = red;
                        self.screen_data[xy.wrapping_add(1) as usize] = green;
                        self.screen_data[xy.wrapping_add(2) as usize] = blue;
                    }
                }
            }
        }
    }

    fn draw_scanline(&mut self) {
        let control: u8 =  {
            let memory = self.memory.lock().unwrap();
            memory.read_from_memory(None, 0xff40)
        };
        if bit_logic::check_bit(control, 0) {
            self.render_tiles();
        }
        if bit_logic::check_bit(control, 1) {
            self.render_sprites();
        }
    }

    fn update_graphics(&mut self, cycles: f64) {
        self.set_lcd_status();
        if self.is_lcd_enabled() {
            self.scanline_counter -= cycles as i32;
        } else {
            return;
        }
        if self.scanline_counter <= 0 {
            let current_line: u16 = {
                let mut memory = self.memory.lock().unwrap();
                memory.rom[0xff44] += 1;
                memory.read_from_memory(None, 0xff44) as u16
            };
            self.scanline_counter = SCANLINE_COUNTER_START as i32;
            if current_line == VERTICAL_BLANK_SCAN_LINE {
                self.request_interrupt(0);
            } else if current_line > VERTICAL_BLANK_SCAN_LINE_MAX {
                let mut memory = self.memory.lock().unwrap();
                memory.rom[0xff44] = 0;
            } else if current_line < VERTICAL_BLANK_SCAN_LINE {
                self.draw_scanline();
            }
        }
    }

    fn request_interrupt(&self, interrupt_id: u8) {
        let mut memory = self.memory.lock().unwrap();
        let mut req: u8 = memory.read_from_memory(None, 0xff0f);
        req = bit_logic::set_bit(req, interrupt_id);
        memory.write_to_memory(None, 0xff0f, req);
    }

    fn service_interrupt(&mut self, interrupt_id: u8) {
        self.cpu.interrupts_enabled = false;
        let mut memory = self.memory.lock().unwrap();
        let mut req: u8 = memory.read_from_memory(None, 0xff0f);
        req = bit_logic::reset_bit(req, interrupt_id);
        memory.write_to_memory(None, 0xff0f, req);
    
        self.cpu.push((self.cpu.pc >> 8) as u8);
        self.cpu.push(self.cpu.pc as u8);

        match interrupt_id {
            0 => { self.cpu.pc = 0x40 },
            1 => { self.cpu.pc = 0x48 },
            2 => { self.cpu.pc = 0x50 },
            3 => { self.cpu.pc = 0x58 },
            4 => { self.cpu.pc = 0x60 },
            _ => {},
        }
    }

    fn do_interrupts(&mut self) -> f64 {
        let (req, enabled): (u8, u8) = {
            let memory = self.memory.lock().unwrap();
            (memory.read_from_memory(None, 0xff0f), memory.read_from_memory(None, 0xffff))
        };
        let potential_for_interrupts: u8 = req & enabled;
        if potential_for_interrupts == 0 {
            if self.ei_halt_bug { self.ei_halt_bug = false; }
            return 0.0;
        }
        if self.cpu.interrupts_enabled || self.ei_halt_bug {
            self.cpu.halted = false;
            for i in 0..5 {
                if bit_logic::check_bit(req, i) && bit_logic::check_bit(enabled, i) {
                    self.service_interrupt(i);
                    return 20.0;
                }
            }
            self.ei_halt_bug = false;
        } else if self.cpu.halted {
            self.cpu.halted = false;
            self.halt_bug = true;
        } else {
            self.cpu.halted = false;
        }
        0.0
    }
}