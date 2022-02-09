use crate::{ WIDTH, SCREEN_DATA_SIZE, FREQUENCY_4096, FREQUENCY_262144, FREQUENCY_65536, FREQUENCY_16384 };
use crate::Cpu;
use crate::Memory;
use crate::bit_logic;

const IS_DEBUG_MODE: bool = true;

pub const TIMA: u16 = 0xff05;
const TMA: u16 = 0xff06;
pub const TAC: u16 = 0xff07;

const VERTICAL_BLANK_SCAN_LINE: u8 = 144;
const VERTICAL_BLANK_SCAN_LINE_MAX: u8 = 153;
const SCANLINE_COUNTER_START: u16 = 456;

enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

pub enum MemoryWriteResult {
    None,
    ResetDividerCounter,
    SetTimerCounter,
}

pub struct Gameboy {
    target_pc: i32,
    scanline_counter: i32,
    pub timer_counter: i32,
    pub divider_counter: i32,
    pub screen_data: [u8; SCREEN_DATA_SIZE as usize],
    scanline_bg: [bool; WIDTH as usize],
    cpu: Cpu,
    memory: Memory,
}

impl Gameboy {
    pub fn new(cpu: Cpu, memory: Memory) -> Gameboy {
        Gameboy {
            target_pc: -1,
            scanline_counter: SCANLINE_COUNTER_START as i32,
            timer_counter: 0,
            divider_counter: 0,
            screen_data: [0; SCREEN_DATA_SIZE as usize],
            scanline_bg: [false; WIDTH as usize],
            cpu: cpu,
            memory: memory,
        }
    }

    pub fn key_pressed(&mut self, key: u8) {
        let previously_unset: bool = !bit_logic::check_bit(self.memory.gamepad_state, key);

        self.memory.gamepad_state = bit_logic::reset_bit(self.memory.gamepad_state, key);

        let button: bool = key > 3;

        let key_req: u8 = self.raw_read_from_rom(0xff00);
        let mut should_request_interrupt: bool = false;

        if button && !bit_logic::check_bit(key_req, 5) {
            should_request_interrupt = true;
        } else if !button && !bit_logic::check_bit(key_req, 4) {
            should_request_interrupt = true;
        }

        if should_request_interrupt && !previously_unset {
            self.request_interrupt(4);
        }
    }

    pub fn key_released(&mut self, key: u8) {
        self.memory.gamepad_state = bit_logic::set_bit(self.memory.gamepad_state, key);
    }

    fn read_from_address(&self, address: u16) -> u8 {
        self.memory.read_from_memory(address)
    }

    fn write_to_address(&mut self, address: u16, value: u8) {
        for memory_result in self.memory.write_to_memory(address, value) {
            match memory_result {
                MemoryWriteResult::ResetDividerCounter => {
                    self.divider_counter = 0
                },
                MemoryWriteResult::SetTimerCounter => {
                    self.set_clock_freq()
                },
                _ => { },
            }
        }
    }

    fn raw_read_from_rom(&self, address: u16) -> u8 {
        self.memory.rom[address as usize]
    }

    fn raw_write_to_rom(&mut self, address: u16, value: u8) {
        self.memory.rom[address as usize] = value;
    }

    pub fn update(&mut self) -> u8 {
        let mut cycles: u8 = 4;
        if !self.cpu.halted {
            if IS_DEBUG_MODE {
                if self.target_pc == -1 {
                    let mut line = String::new();
                    println!("Enter new PC to run to:");
                    std::io::stdin().read_line(&mut line).unwrap();
                    if !line.is_empty() {
                        self.target_pc = line.trim().parse().unwrap_or(-1);
                    }
                } else if self.target_pc == (self.cpu.pc as i32) {
                    self.target_pc = -1;
                }
            }
            cycles = self.cpu.update(&mut self.memory) * 4;
            if IS_DEBUG_MODE {
                println!("{}", self.cpu.debug());
            }
        }
        if IS_DEBUG_MODE {
            if self.raw_read_from_rom(0xff02) == 0x81 {
                let c: char = self.raw_read_from_rom(0xff01) as char;
                print!("{}", c);
                self.raw_write_to_rom(0xff02, 0x0);
            }
        }
        self.update_timer(cycles);
        self.update_graphics(cycles);
        cycles += self.do_interrupts();
        cycles
    }

    fn is_clock_enabled(&self) -> bool {
        bit_logic::check_bit(self.read_from_address(TAC), 2)
    }

    pub fn get_clock_freq(&self) -> u8 {
        self.read_from_address(TAC) & 0x3
    }

    pub fn set_clock_freq(&mut self) {
        match self.get_clock_freq() {
            0 => { self.timer_counter = FREQUENCY_4096 as i32 },
            1 => { self.timer_counter = FREQUENCY_262144 as i32 },
            2 => { self.timer_counter = FREQUENCY_65536 as i32 },
            3 => { self.timer_counter = FREQUENCY_16384 as i32 },
            _ => { },
        }
    }

    fn do_divider_register(&mut self, cycles: u8) {
        self.divider_counter += cycles as i32;
        if self.divider_counter >= 255 {
            self.divider_counter = 0;
            self.raw_write_to_rom(0xff04, self.raw_read_from_rom(0xff04).wrapping_add(1));
        }
    }

    fn update_timer(&mut self, cycles: u8) {
        self.do_divider_register(cycles);
        if self.is_clock_enabled() {
            self.timer_counter -= cycles as i32;
            if self.timer_counter <= 0 {
                self.set_clock_freq();
                let (tima, tma): (u8, u8) = (self.read_from_address(TIMA), self.read_from_address(TMA));
                if tima == 255 {
                    self.write_to_address(TIMA, tma);
                    self.request_interrupt(2);
                } else {
                    self.write_to_address(TIMA, tima.wrapping_add(1))
                }
            }
        }
    }

    fn is_lcd_enabled(&self) -> bool {
        bit_logic::check_bit(self.read_from_address(0xff40), 7)
    }

    fn set_lcd_status(&mut self) {
        let mut status: u8 = self.read_from_address(0xff41);
        if !self.is_lcd_enabled() {
            self.scanline_counter = SCANLINE_COUNTER_START as i32;
            self.raw_write_to_rom(0xff44, 0);
            status &= 252;
            status = bit_logic::set_bit(status, 0);
            self.write_to_address(0xff41, status);
            return;
        }
        let current_line: u8 = self.read_from_address(0xff44);
        let current_mode: u8 = status & 0x3;
        let mode: u8;
        let mut req_int: bool = false;

        if current_line >= 144 {
            mode = 1;
            status = bit_logic::set_bit(status, 0);
            status = bit_logic::reset_bit(status, 1);
            req_int = bit_logic::check_bit(status, 4);
        } else {
            let mode_2_bounds: i32 = 376; // 456 - 80
            let mode_3_bounds: i32 = 204; // mode_2_bounds - 172
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
        if current_line == self.read_from_address(0xff45) {
            status = bit_logic::set_bit(status, 2);
            if bit_logic::check_bit(status, 6) {
                self.request_interrupt(1);
            }
        } else {
            status = bit_logic::reset_bit(status, 2);
        }
        self.write_to_address(0xff41, status);
    }

    fn get_color(&self, address: u16, color_num: u8) -> Color {
        let palette: u8 = self.read_from_address(address);

        let (hi, lo) = match color_num {
            0 => (1, 0),
            1 => (3, 2),
            2 => (5, 4),
            3 => (7, 6),
            _ => (0, 0),
        };

        let mut color: i32 = (if bit_logic::check_bit(palette, hi) { 1 } else { 0 } << 1) as i32;
        color |= if bit_logic::check_bit(palette, lo) { 1 } else { 0 } as i32;
        
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
        let (lcd_control, scroll_y, scroll_x, window_y, window_x): (u8, u8, u8, u8, u8) = (self.read_from_address(0xff40), self.read_from_address(0xff42), self.read_from_address(0xff43), self.read_from_address(0xff4a), self.read_from_address(0xff4b).wrapping_sub(7));

        let mut using_window: bool = false;

        if bit_logic::check_bit(lcd_control, 5) {
            if window_y <= self.read_from_address(0xff44) {
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
            scroll_y.wrapping_add(self.read_from_address(0xff44))
        } else {
            self.read_from_address(0xff44).wrapping_sub(window_y)
        };

        let tile_row: u16 = (y_pos as u16).wrapping_div(8).wrapping_mul(32);
        for pixel in 0..WIDTH {
            let mut x_pos: u8 = pixel.wrapping_add(scroll_x as u16) as u8;
            if using_window && pixel >= (window_x as u16) {
                x_pos = pixel.wrapping_sub(window_x as u16) as u8;
            }
            let tile_column: u16 = x_pos.wrapping_div(8) as u16;
            let tile_address: u16 = background_memory.wrapping_add(tile_row).wrapping_add(tile_column);
            let tile_num: i16 = if unsig {
                self.read_from_address(tile_address) as i16
            } else {
                (self.read_from_address(tile_address) as i8) as i16
            };
            let mut tile_location: u16 = tile_data;
            if unsig {
                tile_location += tile_num.wrapping_mul(16) as u16;
            } else {
                tile_location += tile_num.wrapping_add(128).wrapping_mul(16) as u16;
            }
            let mut line: u8 = y_pos.wrapping_rem(8);
            line *= 2;
            let (data_1, data_2): (u8, u8) = {
                let temp_address: u16 = tile_location.wrapping_add(line as u16);
                (self.read_from_address(temp_address), self.read_from_address(temp_address.wrapping_add(1)))
            };

            let mut color_bit: i32 = x_pos.wrapping_rem(8) as i32;
            color_bit = color_bit.wrapping_sub(7);
            color_bit = color_bit.wrapping_mul(-1);
            let mut color_num: i32 = if bit_logic::check_bit(data_2, color_bit as u8) { 1 } else { 0 };
            color_num <<= 1;
            color_num |= if bit_logic::check_bit(data_1, color_bit as u8) { 1 } else { 0 };

            let color: Color = self.get_color(0xff47, color_num as u8);
            let (red, green, blue): (u8, u8, u8) = match color {
                Color::White => (255, 255, 255),
                Color::LightGray => (0xcc, 0xcc, 0xcc),
                Color::DarkGray => (0x77, 0x77, 0x77),
                _ => (0, 0, 0),
            };

            let finally: u8 = self.read_from_address(0xff44);
            if finally > 143 || pixel > 159 {
                continue;
            }

            self.scanline_bg[pixel as usize] = matches!(color, Color::White);

            let y: u32 = (finally as u32).wrapping_mul(WIDTH as u32).wrapping_mul(3);
            let x: u32 = (pixel as u32).wrapping_mul(3);
            let xy: u32 = x.wrapping_add(y);
            self.screen_data[xy as usize] = red;
            self.screen_data[xy.wrapping_add(1) as usize] = green;
            self.screen_data[xy.wrapping_add(2) as usize] = blue;
        }
    }

    fn render_sprites(&mut self) {
        let use_8x16: bool = bit_logic::check_bit(self.read_from_address(0xff40), 2);
        for sprite in 0..40 {
            let index: u8 = sprite * 4;
            let temp_address: u16 = 0xfe00 + (index as u16);
            let (y_pos, x_pos, tile_location, attributes): (u8, u8, u8, u8) =
                (self.read_from_address(temp_address).wrapping_sub(16),
                self.read_from_address(temp_address + 1).wrapping_sub(8),
                self.read_from_address(temp_address + 2),
                self.read_from_address(temp_address + 3));
            
            let y_flip: bool = bit_logic::check_bit(attributes, 6);
            let x_flip: bool = bit_logic::check_bit(attributes, 5);
            let priority: bool = !bit_logic::check_bit(attributes, 7);
            let scanline: i32 = self.read_from_address(0xff44) as i32;

            let y_size: i32 = if use_8x16 { 16 } else { 8 };

            if (scanline >= (y_pos as i32)) && (scanline < ((y_pos as i32).wrapping_add(y_size))) {
                
                let mut line: i32 = scanline.wrapping_sub(y_pos as i32);

                if y_flip  {
                    line -= y_size;
                    line *= -1;
                }

                line *= 2;
                let data_address: u16 = 0x8000 + (tile_location as u16).wrapping_mul(16) + line as u16;
                let (data_1, data_2): (u8, u8) = (self.read_from_address(data_address), self.read_from_address(data_address.wrapping_add(1)));
                for tile_pixel in (0u8..=7).rev() {
                    let mut color_bit: i32 = tile_pixel as i32;
                    if x_flip {
                        color_bit -= 7;
                        color_bit *= -1;
                    }
                    let mut color_num: u32 = if bit_logic::check_bit(data_2, color_bit as u8) { 1 } else { 0 };
                    color_num <<= 1;
                    color_num |= if bit_logic::check_bit(data_1, color_bit as u8) { 1 } else { 0 };
                    
                    let color_address: u16 = if bit_logic::check_bit(attributes, 4) { 0xff49 } else { 0xff48 };
                    let color: Color = self.get_color(color_address, color_num as u8);

                    if matches!(color, Color::White) {
                        continue;
                    }
                    
                    let (red, green, blue): (u8, u8, u8) = match color {
                        Color::White => (255, 255, 255),
                        Color::LightGray => (0xcc, 0xcc, 0xcc),
                        Color::DarkGray => (0x77, 0x77, 0x77),
                        _ => (0, 0, 0),
                    };

                    let x_pix: u32 = 7_u32.wrapping_sub(tile_pixel as u32);

                    let pixel: u32 = (x_pos as u32).wrapping_add(x_pix);
                    if (scanline < 0) || (scanline > 143) || (pixel > 159) {
                        continue;
                    }

                    if self.scanline_bg[pixel as usize] || priority {
                        let y: i64 = (scanline as i64).wrapping_mul(WIDTH as i64).wrapping_mul(3);
                        let x: i64 = (pixel as i64).wrapping_mul(3);
                        let xy: i64 = x.wrapping_add(y);
                        self.screen_data[xy as usize] = red;
                        self.screen_data[xy.wrapping_add(1) as usize] = green;
                        self.screen_data[xy.wrapping_add(2) as usize] = blue;
                    }
                }
            }
        }
    }

    fn draw_scanline(&mut self) {
        let control: u8 =  self.read_from_address(0xff40);
        if bit_logic::check_bit(control, 0) {
            self.render_tiles();
        }
        if bit_logic::check_bit(control, 1) {
            self.render_sprites();
        }
    }

    fn update_graphics(&mut self, cycles: u8) {
        self.set_lcd_status();
        if self.is_lcd_enabled() {
            self.scanline_counter -= cycles as i32;
        } else {
            return;
        }
        if self.scanline_counter <= 0 {
            let current_line: u8 = {
                self.raw_write_to_rom(0xff44, self.raw_read_from_rom(0xff44).wrapping_add(1));
                self.read_from_address(0xff44)
            };
            self.scanline_counter = SCANLINE_COUNTER_START as i32;
            if current_line == VERTICAL_BLANK_SCAN_LINE {
                self.request_interrupt(0);
            } else if current_line > VERTICAL_BLANK_SCAN_LINE_MAX {
                self.raw_write_to_rom(0xff44, 0);
            } else if current_line < VERTICAL_BLANK_SCAN_LINE {
                self.draw_scanline();
            }
        }
    }

    fn request_interrupt(&mut self, interrupt_id: u8) {
        self.write_to_address(0xff0f, bit_logic::set_bit(self.read_from_address(0xff0f), interrupt_id));
    }

    fn service_interrupt(&mut self, interrupt_id: u8) {
        self.cpu.interrupts_enabled = false;
        self.write_to_address(0xff0f, bit_logic::reset_bit(self.read_from_address(0xff0f), interrupt_id));
    
        let pc: u16 = self.cpu.pc;
        self.cpu.push(&mut self.memory, (pc >> 8) as u8);
        self.cpu.push(&mut self.memory, pc as u8);

        match interrupt_id {
            0 => { self.cpu.pc = 0x40 },
            1 => { self.cpu.pc = 0x48 },
            2 => { self.cpu.pc = 0x50 },
            3 => { self.cpu.pc = 0x58 },
            4 => { self.cpu.pc = 0x60 },
            _ => {},
        }
    }

    fn do_interrupts(&mut self) -> u8 {
        let (req, enabled): (u8, u8) = (self.read_from_address(0xff0f), self.read_from_address(0xffff));
        let potential_for_interrupts: u8 = req & enabled;
        if potential_for_interrupts == 0 {
            return 0;
        }
        if self.cpu.interrupts_enabled {
            self.cpu.halted = false;
            for i in 0..5 {
                if bit_logic::check_bit(req, i) && bit_logic::check_bit(enabled, i) {
                    self.service_interrupt(i);
                    return 20;
                }
            }
        }
        self.cpu.halted = false;
        0
    }
}