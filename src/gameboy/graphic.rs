use super::{bit_logic, WIDTH, Gameboy};

enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl Gameboy {

    fn get_color(&self, address: u16, color_num: u8) -> Color {
        let palette: u8 = self.read_from_address(address);

        let (hi, lo) = match color_num {
            0 => (1, 0),
            1 => (3, 2),
            2 => (5, 4),
            3 => (7, 6),
            _ => (0, 0),
        };

        let mut color: i32 = (if bit_logic::check_bit(palette, hi) { 1 } else { 0 } << 1);
        color |= if bit_logic::check_bit(palette, lo) { 1 } else { 0 };
        
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

        if bit_logic::check_bit(lcd_control, 5) && window_y <= self.read_from_address(0xff44) {
            using_window = true;
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
        } else if bit_logic::check_bit(lcd_control, 6) {
            0x9c00
        } else {
            0x9800
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
                    if !(0..=143).contains(&scanline) || (pixel > 159) {
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

    pub(crate) fn draw_scanline(&mut self) {
        let control: u8 =  self.read_from_address(0xff40);
        if bit_logic::check_bit(control, 0) {
            self.render_tiles();
        }
        if bit_logic::check_bit(control, 1) {
            self.render_sprites();
        }
    }
}