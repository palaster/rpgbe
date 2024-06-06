use super::bit_logic;
use super::WIDTH;
use super::gameboy::{SCANLINE_COUNTER_START, Gameboy};

const VERTICAL_BLANK_SCAN_LINE: u8 = 144;
const VERTICAL_BLANK_SCAN_LINE_MAX: u8 = 153;

impl Gameboy {
    fn get_color(&self, address: u16, color_num: u8) -> u8 {
        let palette: u8 = self.read_from_memory(address);

        let (hi, lo) = match color_num {
            0 => (1, 0),
            1 => (3, 2),
            2 => (5, 4),
            3 => (7, 6),
            _ => (0, 0),
        };
        
        match (if bit_logic::check_bit(palette, hi) { 1 } else { 0 } << 1) | if bit_logic::check_bit(palette, lo) { 1 } else { 0 } {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            _ => 0,
        }
    }

    fn render_tiles(&mut self) {
        let mut unsig: bool = true;
        let (lcd_control, scroll_y, scroll_x, window_y, window_x): (u8, u8, u8, u8, u8) = (self.read_from_memory(0xff40), self.read_from_memory(0xff42), self.read_from_memory(0xff43), self.read_from_memory(0xff4a), self.read_from_memory(0xff4b) - 7);
        let ff44 = self.read_from_memory(0xff44);
        let using_window: bool = bit_logic::check_bit(lcd_control, 5) && window_y <= ff44;

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
        } else if bit_logic::check_bit(lcd_control, 6) {
            0x9c00
        } else {
            0x9800
        };

        let y_pos: u8 = if !using_window {
            scroll_y + ff44
        } else {
            ff44 - window_y
        };

        let tile_row: u16 = (y_pos as u16) / 8 * 32;
        let mut x_pos: u8;
        let mut tile_address: u16;
        let mut tile_num: i16;
        let mut tile_location: u16;
        let mut color_bit: i32;
        let mut color_num: i32;
        let line: u8 = (y_pos % 8) * 2;
        let mut xy: u32;
        for pixel in 0..WIDTH {
            x_pos = (pixel + (scroll_x as u16)) as u8;
            if using_window && pixel >= (window_x as u16) {
                x_pos = (pixel - (window_x as u16)) as u8;
            }
            tile_address = background_memory + tile_row + (x_pos / 8) as u16;
            tile_num = if unsig {
                self.read_from_memory(tile_address) as i16
            } else {
                (self.read_from_memory(tile_address) as i8) as i16
            };
            tile_location = tile_data;
            if unsig {
                tile_location += (tile_num * 16) as u16;
            } else {
                tile_location += ((tile_num + 128) * 16) as u16;
            }
            let (data_1, data_2): (u8, u8) = {
                let temp_address: u16 = tile_location + (line as u16);
                (self.read_from_memory(temp_address), self.read_from_memory(temp_address + 1))
            };

            color_bit = -(((x_pos % 8) as i32) - 7);
            color_num = (if bit_logic::check_bit(data_2, color_bit as u8) { 1 } else { 0 } << 1) | if bit_logic::check_bit(data_1, color_bit as u8) { 1 } else { 0 };

            let (red, green, blue): (u8, u8, u8) = match self.get_color(0xff47, color_num as u8) {
                0 => (255, 255, 255),
                1 => (0xcc, 0xcc, 0xcc),
                2 => (0x77, 0x77, 0x77),
                _ => (0, 0, 0),
            };

            if ff44 > 143 || pixel > 159 {
                continue;
            }

            self.scanline_bg[pixel as usize] = red == 255;

            xy = (pixel as u32) * 3 + (ff44 as u32) * (WIDTH as u32) * 3;
            self.screen_data[xy as usize] = red;
            self.screen_data[(xy + 1) as usize] = green;
            self.screen_data[(xy + 2) as usize] = blue;
        }
    }

    fn render_sprites(&mut self) {
        let y_size: i32 = if bit_logic::check_bit(self.read_from_memory(0xff40), 2) { 16 } else { 8 };
        let mut temp_address: u16;
        let mut scanline: i32;
        let mut line: i32;
        let mut data_address: u16;
        let mut color_bit: i32;
        let mut color_num: u32;
        let mut pixel: u32;
        let mut xy: i64;
        for sprite in 0u16..40 {
            temp_address = 0xfe00 + (sprite * 4);
            let (y_pos, x_pos, tile_location, attributes): (u8, u8, u8, u8) =
                (self.read_from_memory(temp_address) - 16,
                self.read_from_memory(temp_address + 1) - 8,
                self.read_from_memory(temp_address + 2),
                self.read_from_memory(temp_address + 3));
            
            scanline = self.read_from_memory(0xff44) as i32;
            if (scanline >= (y_pos as i32)) && (scanline < ((y_pos as i32) + y_size)) {
                line = scanline - (y_pos as i32);

                if bit_logic::check_bit(attributes, 6) {
                    line = -(line - y_size);
                }

                data_address = 0x8000 + (tile_location as u16) * 16 + (line * 2) as u16;
                let (data_1, data_2): (u8, u8) = (self.read_from_memory(data_address), self.read_from_memory(data_address + 1));
                for tile_pixel in (0u8..=7).rev() {
                    color_bit = tile_pixel as i32;
                    if bit_logic::check_bit(attributes, 5) {
                        color_bit = -(color_bit - 7);
                    }
                    color_num = (if bit_logic::check_bit(data_2, color_bit as u8) { 1 } else { 0 } << 1) | if bit_logic::check_bit(data_1, color_bit as u8) { 1 } else { 0 };
                    
                    let (red, green, blue): (u8, u8, u8) = match self.get_color(if bit_logic::check_bit(attributes, 4) { 0xff49 } else { 0xff48 }, color_num as u8) {
                        0 => (255, 255, 255),
                        1 => (0xcc, 0xcc, 0xcc),
                        2 => (0x77, 0x77, 0x77),
                        _ => (0, 0, 0),
                    };

                    if red == 255 {
                        continue;
                    }

                    pixel = (x_pos as u32) + (7_u32 - (tile_pixel as u32));
                    if pixel > 159 || scanline <= 0 || scanline > 143  {
                        continue;
                    }

                    if self.scanline_bg[pixel as usize] || !bit_logic::check_bit(attributes, 7) {
                        xy = (pixel as i64) * 3 + (scanline as i64) * (WIDTH as i64) * 3;
                        self.screen_data[xy as usize] = red;
                        self.screen_data[(xy + 1) as usize] = green;
                        self.screen_data[(xy + 2) as usize] = blue;
                    }
                }
            }
        }
    }

    fn draw_scanline(&mut self) {
        let control: u8 =  self.read_from_memory(0xff40);
        if bit_logic::check_bit(control, 0) {
            self.render_tiles();
        }
        if bit_logic::check_bit(control, 1) {
            self.render_sprites();
        }
    }

    fn is_lcd_enabled(&self) -> bool {
        bit_logic::check_bit(self.read_from_memory(0xff40), 7)
    }

    fn set_lcd_status(&mut self) {
        let mut status: u8 = self.read_from_memory(0xff41);
        if !self.is_lcd_enabled() {
            self.scanline_counter = SCANLINE_COUNTER_START as i32;
            self.rom[0xff44_usize] = 0;
            self.write_to_memory(0xff41, bit_logic::set_bit(status & 252, 0));
            return;
        }
        let current_line: u8 = self.read_from_memory(0xff44);
        let current_mode: u8 = status & 0x3;
        let mode: u8;
        let mut req_int: bool = false;

        if current_line >= 144 {
            mode = 1;
            status = bit_logic::reset_bit(bit_logic::set_bit(status, 0), 1);
            req_int = bit_logic::check_bit(status, 4);
        } else {
            const MODE_2_BOUNDS: i32 = 376; // 456 - 80
            const MODE_3_BOUNDS: i32 = 204; // mode_2_bounds - 172
            if self.scanline_counter >= MODE_2_BOUNDS {
                mode = 2;
                status = bit_logic::reset_bit(bit_logic::set_bit(status, 1), 0);
                req_int = bit_logic::check_bit(status, 5);
            } else if self.scanline_counter >= MODE_3_BOUNDS {
                mode = 3;
                status = bit_logic::set_bit(bit_logic::set_bit(status, 1), 0);
            } else {
                mode = 0;
                status = bit_logic::reset_bit(bit_logic::reset_bit(status, 1), 0);
                req_int = bit_logic::check_bit(status, 3);
            }
        }
        if req_int && current_mode != mode {
            self.request_interrupt(1);
        }
        if current_line == self.read_from_memory(0xff45) {
            status = bit_logic::set_bit(status, 2);
            if bit_logic::check_bit(status, 6) {
                self.request_interrupt(1);
            }
        } else {
            status = bit_logic::reset_bit(status, 2);
        }
        self.write_to_memory(0xff41, status);
    }

    pub(crate) fn update_graphics(&mut self, cycles: u8) {
        self.set_lcd_status();
        if self.is_lcd_enabled() {
            self.scanline_counter -= cycles as i32;
        } else {
            return;
        }
        if self.scanline_counter <= 0 {
            let current_line = {
                self.rom[0xff44_usize] += 1;
                self.read_from_memory(0xff44)
            };
            self.scanline_counter = SCANLINE_COUNTER_START as i32;
            if current_line == VERTICAL_BLANK_SCAN_LINE {
                self.request_interrupt(0);
            } else if current_line > VERTICAL_BLANK_SCAN_LINE_MAX {
                self.rom[0xff44_usize] = 0;
            } else if current_line < VERTICAL_BLANK_SCAN_LINE {
                self.draw_scanline();
            }
        }
    }
}