// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.

use std::{cell::RefCell, cmp::min, rc::Rc};

use crate::{PioDevice, SpiDevice};

pub const PIXEL_WIDTH: u32 = 800;
pub const PIXEL_HEIGHT: u32 = 480;
pub const CHAR_WIDTH: u32 = 8;
pub const CHAR_HEIGHT: u32 = 16;
pub const TEXT_WIDTH: u32 = PIXEL_WIDTH / CHAR_WIDTH;
pub const TEXT_HEIGHT: u32 = PIXEL_HEIGHT / CHAR_HEIGHT;

pub const DATA_WRITE: u32 = 0x0000;
pub const DATA_READ: u32 = 0x4000;
pub const COMMAND_WRITE: u32 = 0x8000;
#[cfg(test)]
pub const STATUS_READ: u32 = 0xC000;

pub const POWER_AND_DISPLAY_CONTROL: u8 = 0x01;
pub const MEMORY_READ_WRITE_COMMAND: u8 = 0x02;
pub const PIXEL_CLOCK_SETTING: u8 = 0x04;
pub const SYSTEM_CONFIGURATION: u8 = 0x10;
pub const LCD_HORIZONTAL_DISPLAY_WIDTH: u8 = 0x14;
pub const LCD_HORIZONTAL_NON_DISPLAY_PERIOD_FINE_TUNING: u8 = 0x15;
pub const LCD_HORIZONTAL_NON_DISPLAY_PERIOD: u8 = 0x16;
pub const HSYNC_START_POSITION: u8 = 0x17;
pub const LCD_VERTICAL_DISPLAY_HEIGHT0: u8 = 0x19;
pub const LCD_VERTICAL_DISPLAY_HEIGHT1: u8 = 0x1A;
pub const LCD_VERTICAL_NON_DISPLAY_PERIOD0: u8 = 0x1B;
pub const LCD_VERTICAL_NON_DISPLAY_PERIOD1: u8 = 0x1C;
pub const VSYNC_START_POSITION0: u8 = 0x1D;
pub const VSYNC_START_POSITION1: u8 = 0x1E;
pub const DISPLAY_CONFIGURATION_REGISTER: u8 = 0x20;
pub const FONT_WRITE_CURSOR_HORIZONTAL0: u8 = 0x2A;
pub const FONT_WRITE_CURSOR_HORIZONTAL1: u8 = 0x2B;
pub const FONT_WRITE_CURSOR_VERTICAL0: u8 = 0x2C;
pub const FONT_WRITE_CURSOR_VERTICAL1: u8 = 0x2D;
pub const HORIZONTAL_START_OF_ACTIVE_WINDOW0: u8 = 0x30;
pub const HORIZONTAL_START_OF_ACTIVE_WINDOW1: u8 = 0x31;
pub const VERTICAL_START_OF_ACTIVE_WINDOW0: u8 = 0x32;
pub const VERTICAL_START_OF_ACTIVE_WINDOW1: u8 = 0x33;
pub const HORIZONTAL_END_OF_ACTIVE_WINDOW0: u8 = 0x34;
pub const HORIZONTAL_END_OF_ACTIVE_WINDOW1: u8 = 0x35;
pub const VERTICAL_END_OF_ACTIVE_WINDOW0: u8 = 0x36;
pub const VERTICAL_END_OF_ACTIVE_WINDOW1: u8 = 0x37;
pub const MEMORY_WRITE_CONTROL0: u8 = 0x40;
pub const MEMORY_WRITE_CONTROL1: u8 = 0x41;
pub const BLINK_TIME_CONTROL: u8 = 0x44;
pub const LAYER_TRANSPARENCY_REGISTER0: u8 = 0x52;
pub const BACKGROUND_COLOR0: u8 = 0x60;
pub const BACKGROUND_COLOR1: u8 = 0x61;
pub const BACKGROUND_COLOR2: u8 = 0x62;
pub const FOREGROUND_COLOR0: u8 = 0x63;
pub const FOREGROUND_COLOR1: u8 = 0x64;
pub const FOREGROUND_COLOR2: u8 = 0x65;
pub const PLL_CONTROL1: u8 = 0x88;
pub const PLL_CONTROL2: u8 = 0x89;
pub const PWM1_CONTROL: u8 = 0x8A;
pub const PWM1_DUTY_CYCLE: u8 = 0x8B;
pub const MEMORY_CLEAR_CONTROL: u8 = 0x8E;
pub const EXTRA_GENERAL_PURPOSE_IO: u8 = 0xC7;

#[derive(Clone)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait Display {
    fn set_on(&mut self, on: bool);
    fn set_read_layer(&mut self, layer: u32);
    fn set_write_layer(&mut self, layer: u32);
    fn draw_char(&mut self, x: u32, y: u32, c: u8, foreground: Color, background: Color);
    fn set_cursor(&mut self, x: u32, y: u32, enabled: bool, blink_time: Option<u8>);
    fn clear(&mut self, top_left: &Point, bottom_right: &Point, full_screen: bool);
    fn reset(&mut self);
}

#[derive(Clone)]
pub struct TextDisplay {
    on: bool,
    read_layer: u32,
    write_layer: u32,
    textbuffer0: Vec<u8>,
    textbuffer1: Vec<u8>,
    empty_textbuffer: Vec<u8>,
}

impl Default for TextDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl TextDisplay {
    pub fn new() -> Self {
        // See https://en.wikipedia.org/wiki/Linear_congruential_generator.
        let mut rand: u32 = 1;
        let mut textbuffer0 = vec![0; (TEXT_WIDTH * TEXT_HEIGHT) as usize];
        let mut textbuffer1 = vec![0; (TEXT_WIDTH * TEXT_HEIGHT) as usize];
        for i in 0..textbuffer0.len() {
            textbuffer0[i] = rand as u8;
            textbuffer1[i] = (rand >> 8) as u8;
            rand = rand.wrapping_mul(1664525).wrapping_add(1013904223);
        }
        Self {
            on: false,
            read_layer: 0,
            write_layer: 0,
            textbuffer0,
            textbuffer1,
            empty_textbuffer: vec![0; (TEXT_WIDTH * TEXT_HEIGHT) as usize],
        }
    }

    pub fn get_textbuffer(&self) -> &[u8] {
        if self.on {
            if self.read_layer == 1 {
                self.textbuffer1.as_slice()
            } else {
                self.textbuffer0.as_slice()
            }
        } else {
            self.empty_textbuffer.as_slice()
        }
    }

    pub fn get_text(&self) -> String {
        let mut result = String::new();
        if self.on {
            let text = if self.read_layer == 1 {
                &self.textbuffer1
            } else {
                &self.textbuffer0
            };
            let mut line = String::new();
            for y in 0..TEXT_HEIGHT {
                for x in 0..TEXT_WIDTH {
                    let c = text[(x + y * TEXT_WIDTH) as usize];
                    if c.is_ascii_graphic() {
                        line.push(c as char);
                    } else {
                        line.push(' ');
                    }
                }
                result.push_str(line.trim_end());
                result.push('\n');
                line.clear();
            }
            result.push_str(line.trim_end());
            result.truncate(result.trim_end().len());
        }
        result
    }
}

impl Display for TextDisplay {
    fn set_on(&mut self, on: bool) {
        self.on = on;
    }

    fn set_read_layer(&mut self, layer: u32) {
        self.read_layer = layer;
    }

    fn set_write_layer(&mut self, layer: u32) {
        self.write_layer = layer;
    }

    fn set_cursor(&mut self, _x: u32, _y: u32, _enabled: bool, _blink_time: Option<u8>) {}

    fn draw_char(&mut self, x: u32, y: u32, c: u8, _foreground: Color, _background: Color) {
        if x % CHAR_WIDTH == 0 && y % CHAR_HEIGHT == 0 {
            let x = x / CHAR_WIDTH;
            let y = y / CHAR_HEIGHT;
            let text = if self.write_layer == 1 {
                &mut self.textbuffer1
            } else {
                &mut self.textbuffer0
            };
            text[(x + y * TEXT_WIDTH) as usize] = c;
        }
    }

    fn clear(&mut self, top_left: &Point, bottom_right: &Point, full_screen: bool) {
        if full_screen {
            self.textbuffer0.fill(0);
            self.textbuffer1.fill(0);
            return;
        }
        let textbuffer = if self.write_layer == 1 {
            &mut self.textbuffer1
        } else {
            &mut self.textbuffer0
        };
        let x_start = top_left.x / CHAR_WIDTH;
        let x_end = (bottom_right.x + CHAR_WIDTH - 1) / CHAR_WIDTH;
        let y_start = top_left.y / CHAR_HEIGHT;
        let y_end = (bottom_right.y + CHAR_HEIGHT - 1) / CHAR_HEIGHT;
        for y in y_start..y_end {
            for x in x_start..x_end {
                textbuffer[(x + y * TEXT_WIDTH) as usize] = 0;
            }
        }
    }

    fn reset(&mut self) {
        *self = TextDisplay::default();
    }
}

#[derive(Clone)]
pub struct GraphicsCard {
    display: Rc<RefCell<dyn Display>>,
    ready: bool,
    current_register: u8,
    lcd_on: bool,
    pixel_clock_setting: u8,
    system_configuration: u8,
    lcd_horizontal_display_width: u8,
    lcd_horizontal_non_display_period_fine_tuning: u8,
    lcd_horizontal_non_display_period: u8,
    hsync_start_position: u8,
    lcd_vertical_display_height: u32,
    lcd_vertical_non_display_period: u32,
    vsync_start_position: u32,
    display_configuration: u8,
    text_cursor: Point,
    active_window_start: Point,
    active_window_end: Point,
    memory_write_control0: u8,
    memory_write_control1: u8,
    blink_time: u8,
    layer_transparency: u8,
    background_color: Color,
    foreground_color: Color,
    bits_per_pixel: Color,
    pll_control1: u8,
    pll_control2: u8,
    pwm1_control: u8,
    pwm1_duty_cyle: u8,
    gpio_on: bool,
}

impl Default for GraphicsCard {
    fn default() -> Self {
        Self::new(Rc::new(RefCell::new(TextDisplay::default())))
    }
}

impl GraphicsCard {
    const HEADER_MASK: u32 = 0xC000;

    const POWER_AND_DISPLAY_CONTROL_UNSUPPORTED_BITS: u8 = 0x03;
    const PIXEL_CLOCK_SETTING_BITS: u8 = 0x83;
    const SYSTEM_CONFIGURATION_BITS: u8 = 0x0F;
    const LCD_HORIZONTAL_NON_DISPLAY_PERIOD_BITS: u8 = 0x1F;
    const LCD_HORIZONTAL_NON_DISPLAY_PERIOD_FINE_TUNING_BITS: u8 = 0x8F;
    const HSYNC_START_POSITION_BITS: u8 = 0x1F;
    const DISPLAY_CONFIGURATION_BITS: u8 = 0x80;
    const MEMORY_WRITE_CONTROL0_BITS: u8 = 0xEF;
    const MEMORY_WRITE_CONTROL1_BITS: u8 = 0x01;
    const LAYER_TRANSPARENCY_BITS: u8 = 0x01;
    const PLL_CONTROL1_BITS: u8 = 0x9F;
    const PLL_CONTROL2_BITS: u8 = 0x07;
    const PWM1_CONTROL_BITS: u8 = 0xDF;

    const EIGHT_BITS_PER_PIXEL: Color = Color { r: 3, g: 3, b: 2 };
    const SIXTEEN_BITS_PER_PIXEL: Color = Color { r: 5, g: 6, b: 5 };

    const U9_MAX: u32 = (1 << 9) - 1;
    const U10_MAX: u32 = (1 << 10) - 1;

    pub fn new(display: Rc<RefCell<dyn Display>>) -> Self {
        Self {
            display,
            ready: false,
            current_register: 0,
            lcd_on: false,
            pixel_clock_setting: 0,
            system_configuration: 0,
            lcd_horizontal_display_width: 0,
            lcd_horizontal_non_display_period_fine_tuning: 0,
            lcd_horizontal_non_display_period: 0,
            hsync_start_position: 0,
            lcd_vertical_display_height: 0,
            lcd_vertical_non_display_period: 0,
            vsync_start_position: 0,
            display_configuration: 0,
            text_cursor: Point { x: 0, y: 0 },
            active_window_start: Point { x: 0, y: 0 },
            active_window_end: Point { x: 0, y: 0 },
            memory_write_control0: 0,
            memory_write_control1: 0,
            blink_time: 0,
            layer_transparency: 0,
            background_color: Color { r: 0, g: 0, b: 0 },
            foreground_color: Color {
                r: 0xF8,
                g: 0xFC,
                b: 0xF8,
            },
            bits_per_pixel: Self::EIGHT_BITS_PER_PIXEL,
            pll_control1: 0x07,
            pll_control2: 0x03,
            pwm1_control: 0,
            pwm1_duty_cyle: 0,
            gpio_on: false,
        }
    }

    pub fn set_display(&mut self, display: Rc<RefCell<dyn Display>>) {
        self.display = display;
    }

    pub fn display_on(&self) -> bool {
        // RA8875 GPIO pin is connected to TFT ON/OFF input (see R8875 schematic diagram).
        if !self.lcd_on || !self.gpio_on {
            return false;
        }
        // The backlight must be on too.
        if (self.pwm1_control & 0x80) != 0 {
            // If PWM1 enabled, PWM1 fixed frequency must not be selected.
            self.pwm1_control & 0x10 == 0 && self.pwm1_duty_cyle != 0
        } else {
            // If PWM1 disabled, PWM1_OUT must be HIGH.
            self.pwm1_control & 0x40 != 0
        }
    }

    fn read_data(&self) -> u8 {
        match self.current_register {
            POWER_AND_DISPLAY_CONTROL => (self.lcd_on as u8) << 7,
            MEMORY_READ_WRITE_COMMAND => panic!("Read from RA8875 MRWC register unsupported"),
            PIXEL_CLOCK_SETTING => self.pixel_clock_setting,
            SYSTEM_CONFIGURATION => self.system_configuration,
            LCD_HORIZONTAL_DISPLAY_WIDTH => self.lcd_horizontal_display_width,
            LCD_HORIZONTAL_NON_DISPLAY_PERIOD_FINE_TUNING => {
                self.lcd_horizontal_non_display_period_fine_tuning
            }
            LCD_HORIZONTAL_NON_DISPLAY_PERIOD => self.lcd_horizontal_non_display_period,
            HSYNC_START_POSITION => self.hsync_start_position,
            LCD_VERTICAL_DISPLAY_HEIGHT0 => Self::get_low(self.lcd_vertical_display_height),
            LCD_VERTICAL_DISPLAY_HEIGHT1 => Self::get_high(self.lcd_vertical_display_height),
            LCD_VERTICAL_NON_DISPLAY_PERIOD0 => Self::get_low(self.lcd_vertical_non_display_period),
            LCD_VERTICAL_NON_DISPLAY_PERIOD1 => {
                Self::get_high(self.lcd_vertical_non_display_period)
            }
            VSYNC_START_POSITION0 => Self::get_low(self.vsync_start_position),
            VSYNC_START_POSITION1 => Self::get_high(self.vsync_start_position),
            DISPLAY_CONFIGURATION_REGISTER => self.display_configuration,
            FONT_WRITE_CURSOR_HORIZONTAL0 => Self::get_low(self.text_cursor.x),
            FONT_WRITE_CURSOR_HORIZONTAL1 => Self::get_high(self.text_cursor.x),
            FONT_WRITE_CURSOR_VERTICAL0 => Self::get_low(self.text_cursor.y),
            FONT_WRITE_CURSOR_VERTICAL1 => Self::get_high(self.text_cursor.y),
            HORIZONTAL_START_OF_ACTIVE_WINDOW0 => Self::get_low(self.active_window_start.x),
            HORIZONTAL_START_OF_ACTIVE_WINDOW1 => Self::get_high(self.active_window_start.x),
            VERTICAL_START_OF_ACTIVE_WINDOW0 => Self::get_low(self.active_window_start.y),
            VERTICAL_START_OF_ACTIVE_WINDOW1 => Self::get_high(self.active_window_start.y),
            HORIZONTAL_END_OF_ACTIVE_WINDOW0 => Self::get_low(self.active_window_end.x),
            HORIZONTAL_END_OF_ACTIVE_WINDOW1 => Self::get_high(self.active_window_end.x),
            VERTICAL_END_OF_ACTIVE_WINDOW0 => Self::get_low(self.active_window_end.y),
            VERTICAL_END_OF_ACTIVE_WINDOW1 => Self::get_high(self.active_window_end.y),
            MEMORY_WRITE_CONTROL0 => self.memory_write_control0,
            MEMORY_WRITE_CONTROL1 => self.memory_write_control1,
            BLINK_TIME_CONTROL => self.blink_time,
            LAYER_TRANSPARENCY_REGISTER0 => self.layer_transparency,
            BACKGROUND_COLOR0 => Self::get_high_n(self.background_color.r, self.bits_per_pixel.r),
            BACKGROUND_COLOR1 => Self::get_high_n(self.background_color.g, self.bits_per_pixel.g),
            BACKGROUND_COLOR2 => Self::get_high_n(self.background_color.b, self.bits_per_pixel.b),
            FOREGROUND_COLOR0 => Self::get_high_n(self.foreground_color.r, self.bits_per_pixel.r),
            FOREGROUND_COLOR1 => Self::get_high_n(self.foreground_color.g, self.bits_per_pixel.g),
            FOREGROUND_COLOR2 => Self::get_high_n(self.foreground_color.b, self.bits_per_pixel.b),
            PLL_CONTROL1 => self.pll_control1,
            PLL_CONTROL2 => self.pll_control2,
            PWM1_CONTROL => self.pwm1_control,
            PWM1_DUTY_CYCLE => self.pwm1_duty_cyle,
            MEMORY_CLEAR_CONTROL => 0,
            EXTRA_GENERAL_PURPOSE_IO => self.gpio_on as u8,
            _ => panic!("Unsupported RA8875 register {:#04X}", self.current_register),
        }
    }

    fn write_data(&mut self, value: u8) {
        match self.current_register {
            POWER_AND_DISPLAY_CONTROL => {
                if value & Self::POWER_AND_DISPLAY_CONTROL_UNSUPPORTED_BITS != 0 {
                    panic!("Unsupported RA8875 PWRR register value {value:#04X}");
                }
                self.lcd_on = (value & 0x80) != 0;
                self.display.borrow_mut().set_on(self.display_on());
            }
            MEMORY_READ_WRITE_COMMAND => self.write_char(value),
            PIXEL_CLOCK_SETTING => {
                self.pixel_clock_setting = value & Self::PIXEL_CLOCK_SETTING_BITS;
                self.update_status();
            }
            SYSTEM_CONFIGURATION => {
                self.system_configuration = value & Self::SYSTEM_CONFIGURATION_BITS;
                self.update_status();
            }
            LCD_HORIZONTAL_DISPLAY_WIDTH => {
                self.lcd_horizontal_display_width = value;
                self.update_status();
            }
            LCD_HORIZONTAL_NON_DISPLAY_PERIOD_FINE_TUNING => {
                self.lcd_horizontal_non_display_period_fine_tuning =
                    value & Self::LCD_HORIZONTAL_NON_DISPLAY_PERIOD_FINE_TUNING_BITS;
                self.update_status();
            }
            LCD_HORIZONTAL_NON_DISPLAY_PERIOD => {
                self.lcd_horizontal_non_display_period =
                    value & Self::LCD_HORIZONTAL_NON_DISPLAY_PERIOD_BITS;
                self.update_status();
            }
            HSYNC_START_POSITION => {
                self.hsync_start_position = value & Self::HSYNC_START_POSITION_BITS;
                self.update_status();
            }
            LCD_VERTICAL_DISPLAY_HEIGHT0 => {
                Self::set_low(&mut self.lcd_vertical_display_height, value, u32::MAX);
                self.update_status();
            }
            LCD_VERTICAL_DISPLAY_HEIGHT1 => {
                Self::set_high(&mut self.lcd_vertical_display_height, value, u32::MAX);
                self.update_status();
            }
            LCD_VERTICAL_NON_DISPLAY_PERIOD0 => {
                Self::set_low(
                    &mut self.lcd_vertical_non_display_period,
                    value,
                    Self::U9_MAX,
                );
                self.update_status();
            }
            LCD_VERTICAL_NON_DISPLAY_PERIOD1 => {
                Self::set_high(
                    &mut self.lcd_vertical_non_display_period,
                    value,
                    Self::U9_MAX,
                );
                self.update_status();
            }
            VSYNC_START_POSITION0 => {
                Self::set_low(&mut self.vsync_start_position, value, Self::U9_MAX);
                self.update_status();
            }
            VSYNC_START_POSITION1 => {
                Self::set_high(&mut self.vsync_start_position, value, Self::U9_MAX);
                self.update_status();
            }
            DISPLAY_CONFIGURATION_REGISTER => {
                if value & !Self::DISPLAY_CONFIGURATION_BITS != 0 {
                    panic!("Unsupported RA8875 Display Configuration Register value {value}");
                }
                self.display_configuration = value;
                self.update_layers();
            }
            FONT_WRITE_CURSOR_HORIZONTAL0 => {
                Self::set_low(&mut self.text_cursor.x, value, Self::U10_MAX);
                self.update_cursor();
            }
            FONT_WRITE_CURSOR_HORIZONTAL1 => {
                Self::set_high(&mut self.text_cursor.x, value, Self::U10_MAX);
                self.update_cursor();
            }
            FONT_WRITE_CURSOR_VERTICAL0 => {
                Self::set_low(&mut self.text_cursor.y, value, Self::U9_MAX);
                self.update_cursor();
            }
            FONT_WRITE_CURSOR_VERTICAL1 => {
                Self::set_high(&mut self.text_cursor.y, value, Self::U9_MAX);
                self.update_cursor();
            }
            HORIZONTAL_START_OF_ACTIVE_WINDOW0 => Self::set_low(
                &mut self.active_window_start.x,
                value,
                PIXEL_WIDTH - CHAR_WIDTH,
            ),
            HORIZONTAL_START_OF_ACTIVE_WINDOW1 => Self::set_high(
                &mut self.active_window_start.x,
                value,
                PIXEL_WIDTH - CHAR_WIDTH,
            ),
            VERTICAL_START_OF_ACTIVE_WINDOW0 => Self::set_low(
                &mut self.active_window_start.y,
                value,
                PIXEL_HEIGHT - CHAR_HEIGHT,
            ),
            VERTICAL_START_OF_ACTIVE_WINDOW1 => Self::set_high(
                &mut self.active_window_start.y,
                value,
                PIXEL_HEIGHT - CHAR_HEIGHT,
            ),
            HORIZONTAL_END_OF_ACTIVE_WINDOW0 => {
                Self::set_low(&mut self.active_window_end.x, value, PIXEL_WIDTH)
            }
            HORIZONTAL_END_OF_ACTIVE_WINDOW1 => {
                Self::set_high(&mut self.active_window_end.x, value, PIXEL_WIDTH)
            }
            VERTICAL_END_OF_ACTIVE_WINDOW0 => {
                Self::set_low(&mut self.active_window_end.y, value, PIXEL_HEIGHT)
            }
            VERTICAL_END_OF_ACTIVE_WINDOW1 => {
                Self::set_high(&mut self.active_window_end.y, value, PIXEL_HEIGHT)
            }
            MEMORY_WRITE_CONTROL0 => {
                self.memory_write_control0 = value & Self::MEMORY_WRITE_CONTROL0_BITS;
                self.update_cursor();
                self.update_status();
            }
            MEMORY_WRITE_CONTROL1 => {
                if value & !Self::MEMORY_WRITE_CONTROL1_BITS != 0 {
                    panic!("Unsupported RA8875 Memory Write Control Register 1 value {value}");
                }
                self.memory_write_control1 = value;
                self.update_layers();
            }
            BLINK_TIME_CONTROL => {
                self.blink_time = value;
                self.update_cursor();
            }
            LAYER_TRANSPARENCY_REGISTER0 => {
                if value & !Self::LAYER_TRANSPARENCY_BITS != 0 {
                    panic!("Unsupported RA8875 Layer Transparency Register 0 value {value}");
                }
                self.layer_transparency = value;
                self.update_layers();
            }
            BACKGROUND_COLOR0 => {
                Self::set_high_n(&mut self.background_color.r, value, self.bits_per_pixel.r)
            }
            BACKGROUND_COLOR1 => {
                Self::set_high_n(&mut self.background_color.g, value, self.bits_per_pixel.g)
            }
            BACKGROUND_COLOR2 => {
                Self::set_high_n(&mut self.background_color.b, value, self.bits_per_pixel.b)
            }
            FOREGROUND_COLOR0 => {
                Self::set_high_n(&mut self.foreground_color.r, value, self.bits_per_pixel.r)
            }
            FOREGROUND_COLOR1 => {
                Self::set_high_n(&mut self.foreground_color.g, value, self.bits_per_pixel.g)
            }
            FOREGROUND_COLOR2 => {
                Self::set_high_n(&mut self.foreground_color.b, value, self.bits_per_pixel.b)
            }
            PLL_CONTROL1 => {
                self.pll_control1 = value & Self::PLL_CONTROL1_BITS;
                self.update_status();
            }
            PLL_CONTROL2 => {
                self.pll_control2 = value & Self::PLL_CONTROL2_BITS;
                self.update_status();
            }
            PWM1_CONTROL => {
                self.pwm1_control = value & Self::PWM1_CONTROL_BITS;
                self.display.borrow_mut().set_on(self.display_on());
            }
            PWM1_DUTY_CYCLE => {
                self.pwm1_duty_cyle = value;
                self.display.borrow_mut().set_on(self.display_on());
            }
            MEMORY_CLEAR_CONTROL => {
                if value & 0x80 != 0 {
                    self.clear(value & 0x40 == 0);
                }
            }
            EXTRA_GENERAL_PURPOSE_IO => {
                self.gpio_on = (value & 1) != 0;
                self.display.borrow_mut().set_on(self.display_on());
            }
            _ => panic!("Unsupported RA8875 register {:#04X}", self.current_register),
        }
    }

    fn get_low(register: u32) -> u8 {
        register as u8
    }

    fn get_high(register: u32) -> u8 {
        (register >> 8) as u8
    }

    fn get_high_n(register: u8, bits: u8) -> u8 {
        register >> (8 - bits)
    }

    fn set_low(register: &mut u32, value: u8, max_value: u32) {
        *register = min((*register & 0xFF00) | (value as u32), max_value);
    }

    fn set_high(register: &mut u32, value: u8, max_value: u32) {
        *register = min((*register & 0x00FF) | ((value as u32) << 8), max_value);
    }

    fn set_high_n(register: &mut u8, value: u8, bits: u8) {
        *register = (value & ((1 << bits) - 1)) << (8 - bits);
    }

    fn update_cursor(&mut self) {
        self.display.borrow_mut().set_cursor(
            self.text_cursor.x,
            self.text_cursor.y,
            self.memory_write_control0 & 0x40 != 0,
            if self.memory_write_control0 & 0x20 != 0 {
                Some(self.blink_time)
            } else {
                None
            },
        );
    }

    fn update_layers(&mut self) {
        let multilayer = self.display_configuration & 0x80 != 0;
        self.display
            .borrow_mut()
            .set_read_layer((multilayer && self.layer_transparency & 0x01 != 0) as u32);
        self.display
            .borrow_mut()
            .set_write_layer((multilayer && self.memory_write_control1 & 0x01 != 0) as u32);
    }

    fn update_status(&mut self) {
        const CRYSTAL_FREQUENCY: u32 = 20_000_000;
        const TEXT_MODE: u8 = 0x80;

        if self.lcd_horizontal_display_width as u32 >= PIXEL_WIDTH / 8 {
            panic!(
                "Unsupported RA8875 HDWR register value {}",
                self.lcd_horizontal_display_width
            );
        }
        if self.lcd_vertical_display_height >= PIXEL_HEIGHT {
            panic!(
                "Unsupported RA8875 VDHR register value {}",
                self.lcd_vertical_display_height
            );
        }

        let width_pixels = (self.lcd_horizontal_display_width as u32 + 1) * 8;
        let horizontal_non_display_pixels = (self.lcd_horizontal_non_display_period as u32 + 1) * 8
            + (self.lcd_horizontal_non_display_period_fine_tuning as u32 & 0xF)
            + 2;
        let hsync_start_pixels = (self.hsync_start_position as u32 + 1) * 8;
        let height_pixels = self.lcd_vertical_display_height + 1;
        let vertical_non_display_pixels = self.lcd_vertical_non_display_period + 1;
        let vsync_start_pixels = self.vsync_start_position + 1;

        // Back porch includes the pulse width, equal to 8 by default.
        let horizontal_back_porch = horizontal_non_display_pixels + 8;
        let horizontal_front_porch = hsync_start_pixels;
        let horizontal_period = horizontal_back_porch + width_pixels + horizontal_front_porch;
        // Back porch includes the pulse width, equal to 1 by default.
        let vertical_back_porch = vertical_non_display_pixels + 1;
        let vertical_front_porch = vsync_start_pixels;
        let vertical_period = vertical_back_porch + height_pixels + vertical_front_porch;

        self.bits_per_pixel = if self.system_configuration & 0x08 != 0 {
            Self::SIXTEEN_BITS_PER_PIXEL
        } else {
            Self::EIGHT_BITS_PER_PIXEL
        };
        let pll_divm = (self.pll_control1 >> 7) as u32;
        let pll_divn = (self.pll_control1 & 0x1F) as u32;
        let pll_divk = self.pll_control2 as u32;
        let system_clock = CRYSTAL_FREQUENCY * (pll_divn + 1) / ((pll_divm + 1) * (1 << pll_divk));
        let pixel_clock_inverted = (self.pixel_clock_setting & 0x80) != 0;
        let pixel_clock_period = self.pixel_clock_setting & 0x3;
        let pixel_clock = system_clock / (1 << pixel_clock_period);

        if pll_divn == 0 {
            panic!("Illegal RA8875 PLLC1 register divn value 0");
        }

        self.ready = width_pixels == PIXEL_WIDTH
            && height_pixels == PIXEL_HEIGHT
            // See Section 7.2 Display Timing characteristics in TFT Specification document.
            // Ranges for "mandatory" values are made up. In practice almost any value for the
            // back porch, front porch and period works. It seems that the DE "Data Enable" signal
            // is used to detect actual start and end of lines, instead of counting clock cycles
            // after HSYNC and starting after the mandatory back porch (46 cycles).
            && (862..=1200).contains(&horizontal_period) // typical=1056, actual=1054
            && (16..=354).contains(&horizontal_back_porch) // mandatory=46, actual=46
            && (16..=354).contains(&horizontal_front_porch) // typical=210, actual=208
            && (510..=650).contains(&vertical_period) //typical=525, actual=525
            && (7..=147).contains(&vertical_back_porch) // mandatory=23, actual=23
            && (7..=147).contains(&vertical_front_porch) //typical=22, actual=22
            && (self.memory_write_control0 & TEXT_MODE) != 0
            // See Note 3 in Section 5-9 "PLL Setting Registers" (crystal frequency is 20MHz).
            && pll_divn >= 5
            && pixel_clock_inverted
            && (30_000_000..50_000_000).contains(&pixel_clock);
    }

    fn write_char(&mut self, value: u8) {
        if !self.ready {
            return;
        }
        if self.text_cursor.x + CHAR_WIDTH > self.active_window_end.x + 1 {
            self.text_cursor.x = self.active_window_start.x;
            self.text_cursor.y += CHAR_HEIGHT;
        }
        if self.text_cursor.y + CHAR_HEIGHT > self.active_window_end.y + 1 {
            self.text_cursor.y = self.active_window_start.y;
        }
        self.display.borrow_mut().draw_char(
            self.text_cursor.x,
            self.text_cursor.y,
            value,
            self.foreground_color,
            self.background_color,
        );
        self.text_cursor.x += CHAR_WIDTH;
        if self.text_cursor.x < self.active_window_start.x {
            self.text_cursor.x = self.active_window_start.x;
            self.text_cursor.y += CHAR_HEIGHT;
        }
        if self.text_cursor.y < self.active_window_start.y {
            self.text_cursor.y = self.active_window_start.y;
        }
        self.update_cursor();
    }

    fn clear(&mut self, full_screen: bool) {
        self.display.borrow_mut().clear(
            &self.active_window_start,
            &self.active_window_end,
            full_screen,
        );
    }
}

// Requires Clock Polarity CPOL=0, Clock Phase NCPHA=1, Chip Select Not Active After Transfer
// CSNAAT=0, Chip Select Active After Transfer CSAAT=0 and Bits Per Transfer BITS=8 (16_BIT).
// See section 32.8.9, p703 of the Atmel SAM3X Datasheet.
const CHIP_SELECT_REQUIRED_MASK: u32 = 0xFF;
const CHIP_SELECT_REQUIRED_VALUE: u32 = 0x82;

impl SpiDevice for GraphicsCard {
    fn data_received(&mut self, data: u32, chip_select: u32) -> Option<u32> {
        if chip_select & CHIP_SELECT_REQUIRED_MASK != CHIP_SELECT_REQUIRED_VALUE {
            return Option::None;
        }
        if data & Self::HEADER_MASK == DATA_WRITE {
            self.write_data(data as u8);
            Option::Some(0)
        } else if data & Self::HEADER_MASK == DATA_READ {
            Option::Some(self.read_data() as u32)
        } else if data & Self::HEADER_MASK == COMMAND_WRITE {
            self.current_register = data as u8;
            Option::Some(0)
        } else {
            Option::Some(0) // STATUS_READ
        }
    }
}

impl PioDevice for GraphicsCard {
    fn pio_state_changed(&mut self, pins: &[u32; 4]) {
        if pins[1] & (1 << 12) == 0 {
            self.display.borrow_mut().reset();
            *self = Self::new(self.display.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHIP_SELECT: u32 = 0x1582;

    fn write_register(gpu: &mut GraphicsCard, register: u8, value: u8) {
        gpu.data_received(0x8000 | register as u32, CHIP_SELECT);
        gpu.data_received(value as u32, CHIP_SELECT);
    }

    fn initialize(gpu: &mut GraphicsCard) {
        write_register(gpu, PLL_CONTROL1, 11);
        write_register(gpu, PLL_CONTROL2, 2);
        write_register(gpu, PIXEL_CLOCK_SETTING, 0x81); // PCSR_PDATL | PCSR_2CLK
        write_register(gpu, SYSTEM_CONFIGURATION, 0x08); // SYSR_16BPP
        write_register(gpu, LCD_HORIZONTAL_DISPLAY_WIDTH, 99);
        write_register(gpu, LCD_HORIZONTAL_NON_DISPLAY_PERIOD, 4);
        write_register(gpu, HSYNC_START_POSITION, 25);
        write_register(gpu, LCD_VERTICAL_DISPLAY_HEIGHT0, 223);
        write_register(gpu, LCD_VERTICAL_DISPLAY_HEIGHT1, 1);
        write_register(gpu, LCD_VERTICAL_NON_DISPLAY_PERIOD0, 21);
        write_register(gpu, VSYNC_START_POSITION0, 21);
        write_register(gpu, POWER_AND_DISPLAY_CONTROL, 0x80); // PWRR_DISPON
        write_register(gpu, EXTRA_GENERAL_PURPOSE_IO, 1);
        write_register(gpu, PWM1_CONTROL, 0x8A); // P1CR_ENABLE | PWM_CLK_DIV1024
        write_register(gpu, PWM1_DUTY_CYCLE, 255);
        write_register(gpu, MEMORY_WRITE_CONTROL0, 0xE0); // TXTMODE | CURSOR | BLINK
    }

    #[test]
    fn data_received_read_data() {
        let mut gpu = GraphicsCard::default();

        let result0 = gpu.data_received(COMMAND_WRITE | FOREGROUND_COLOR0 as u32, CHIP_SELECT);
        let result2 = gpu.data_received(DATA_READ, CHIP_SELECT);

        assert_eq!(result0, Option::Some(0));
        assert_eq!(result2, Option::Some(0x07));
    }

    #[test]
    #[should_panic(expected = "Unsupported RA8875 register 0x03")]
    fn data_received_read_data_unsupported_register() {
        let mut gpu = GraphicsCard::default();

        gpu.data_received(0x8003, CHIP_SELECT);
        gpu.data_received(0x4000, CHIP_SELECT);
    }

    #[test]
    fn data_received_read_data_bad_chip_select() {
        let mut gpu = GraphicsCard::default();

        let result0 = gpu.data_received(COMMAND_WRITE | FOREGROUND_COLOR0 as u32, 0);
        let result2 = gpu.data_received(DATA_READ, 0);

        assert_eq!(result0, Option::None);
        assert_eq!(result2, Option::None);
    }

    #[test]
    fn data_received_write_data() {
        let mut gpu = GraphicsCard::default();

        let result0 = gpu.data_received(COMMAND_WRITE | BACKGROUND_COLOR0 as u32, CHIP_SELECT);
        let result1 = gpu.data_received(DATA_WRITE | 0x07, CHIP_SELECT);
        let result2 = gpu.data_received(DATA_READ, CHIP_SELECT);

        assert_eq!(result0, Option::Some(0));
        assert_eq!(result1, Option::Some(0));
        assert_eq!(result2, Option::Some(0x07));
    }

    #[test]
    #[should_panic(expected = "Unsupported RA8875 PWRR register value 0x82")]
    fn data_received_write_unsupported_power_option() {
        let mut gpu = GraphicsCard::default();
        write_register(&mut gpu, POWER_AND_DISPLAY_CONTROL, 0x82);
    }

    #[test]
    #[should_panic(expected = "Unsupported RA8875 HDWR register value 100")]
    fn data_received_write_unsupported_display_width() {
        let mut gpu = GraphicsCard::default();
        write_register(&mut gpu, LCD_HORIZONTAL_DISPLAY_WIDTH, 100);
    }

    #[test]
    #[should_panic(expected = "Unsupported RA8875 VDHR register value 480")]
    fn data_received_write_unsupported_display_height() {
        let mut gpu = GraphicsCard::default();
        write_register(&mut gpu, LCD_VERTICAL_DISPLAY_HEIGHT0, 224);
        write_register(&mut gpu, LCD_VERTICAL_DISPLAY_HEIGHT1, 1);
    }

    #[test]
    #[should_panic(expected = "Illegal RA8875 PLLC1 register divn value 0")]
    fn data_received_write_illegal_pll_divn_value() {
        let mut gpu = GraphicsCard::default();
        write_register(&mut gpu, PLL_CONTROL1, 0x0);
    }

    #[test]
    #[should_panic(expected = "Unsupported RA8875 register 0x03")]
    fn data_received_write_data_unsupported_register() {
        let mut gpu = GraphicsCard::default();

        gpu.data_received(0x8003, CHIP_SELECT);
        gpu.data_received(0x0000, CHIP_SELECT);
    }

    #[test]
    fn data_received_read_status() {
        let mut gpu = GraphicsCard::default();

        let result = gpu.data_received(STATUS_READ, CHIP_SELECT);

        assert_eq!(result, Option::Some(0));
    }

    #[test]
    fn ready_bad_display_width() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, LCD_HORIZONTAL_DISPLAY_WIDTH, 98);

        assert!(!gpu.ready);
    }

    #[test]
    fn ready_bad_display_height() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, LCD_VERTICAL_DISPLAY_HEIGHT0, 222);

        assert!(!gpu.ready);
    }

    #[test]
    fn ready_bad_horizontal_period() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, LCD_HORIZONTAL_NON_DISPLAY_PERIOD, 31);
        write_register(&mut gpu, HSYNC_START_POSITION, 31);

        assert!(!gpu.ready);
    }

    #[test]
    fn ready_bad_horizontal_front_porch() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, HSYNC_START_POSITION, 0);

        assert!(!gpu.ready);
    }

    #[test]
    fn ready_bad_vertical_period() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, LCD_VERTICAL_NON_DISPLAY_PERIOD0, 7);
        write_register(&mut gpu, VSYNC_START_POSITION0, 7);

        assert!(!gpu.ready);
    }

    #[test]
    fn ready_bad_vertical_back_porch() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, LCD_VERTICAL_NON_DISPLAY_PERIOD0, 0);
        write_register(&mut gpu, VSYNC_START_POSITION0, 50);

        assert!(!gpu.ready);
    }

    #[test]
    fn ready_bad_vertical_front_porch() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, LCD_VERTICAL_NON_DISPLAY_PERIOD0, 50);
        write_register(&mut gpu, VSYNC_START_POSITION0, 0);

        assert!(!gpu.ready);
    }

    #[test]
    fn ready_not_text_mode() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, MEMORY_WRITE_CONTROL0, 0);

        assert!(!gpu.ready);
    }

    #[test]
    fn ready_bad_pll_divn() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, PLL_CONTROL1, 4);

        assert!(!gpu.ready);
    }

    #[test]
    fn ready_bad_pixel_clock_setting() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, PIXEL_CLOCK_SETTING, 1);

        assert!(!gpu.ready);
    }

    #[test]
    fn ready_bad_pixel_clock_frequency() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, PLL_CONTROL2, 4);

        assert!(!gpu.ready);
    }

    #[test]
    fn display_on_lcd_off() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, POWER_AND_DISPLAY_CONTROL, 0);

        assert!(!gpu.display_on());
    }

    #[test]
    fn display_on_gpio_off() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, EXTRA_GENERAL_PURPOSE_IO, 0);

        assert!(!gpu.display_on());
    }

    #[test]
    fn display_on_pwm1_enabled_with_fixed_frequency() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, PWM1_CONTROL, 0x90);

        assert!(!gpu.display_on());
    }

    #[test]
    fn display_on_pwm1_disabled() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);

        write_register(&mut gpu, PWM1_CONTROL, 0x00);

        assert!(!gpu.display_on());
    }

    #[test]
    fn read_write_data_power_and_display_control() {
        let mut gpu = GraphicsCard {
            current_register: POWER_AND_DISPLAY_CONTROL,
            ..Default::default()
        };

        gpu.write_data(0x80);

        assert_eq!(gpu.read_data(), 0x80);
    }

    #[test]
    #[should_panic(expected = "Read from RA8875 MRWC register unsupported")]
    fn read_data_memory_read_write_command() {
        let gpu = GraphicsCard {
            current_register: MEMORY_READ_WRITE_COMMAND,
            ..Default::default()
        };

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn write_data_memory_read_write_command() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        gpu.current_register = MEMORY_READ_WRITE_COMMAND;

        gpu.write_data(b'h');

        assert_eq!(display.borrow().textbuffer0[0], b'h');
        assert_ne!(display.borrow().textbuffer1[0], b'h');
    }

    #[test]
    fn read_write_data_pixel_clock_setting() {
        let mut gpu = GraphicsCard {
            current_register: PIXEL_CLOCK_SETTING,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), GraphicsCard::PIXEL_CLOCK_SETTING_BITS);
    }

    #[test]
    fn read_write_data_system_configuration() {
        let mut gpu = GraphicsCard {
            current_register: SYSTEM_CONFIGURATION,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), GraphicsCard::SYSTEM_CONFIGURATION_BITS);
    }

    #[test]
    fn read_write_data_lcd_horizontal_display_width() {
        let mut gpu = GraphicsCard {
            current_register: LCD_HORIZONTAL_DISPLAY_WIDTH,
            ..Default::default()
        };

        gpu.write_data(0x63);

        assert_eq!(gpu.read_data(), 0x63);
    }

    #[test]
    fn read_write_data_lcd_horizontal_non_display_period() {
        let mut gpu = GraphicsCard {
            current_register: LCD_HORIZONTAL_NON_DISPLAY_PERIOD,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(
            gpu.read_data(),
            GraphicsCard::LCD_HORIZONTAL_NON_DISPLAY_PERIOD_BITS
        );
    }

    #[test]
    fn read_write_data_hsync_start_position() {
        let mut gpu = GraphicsCard {
            current_register: HSYNC_START_POSITION,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), GraphicsCard::HSYNC_START_POSITION_BITS);
    }

    #[test]
    fn read_write_data_lcd_vertical_display_height0() {
        let mut gpu = GraphicsCard {
            current_register: LCD_VERTICAL_DISPLAY_HEIGHT0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_lcd_vertical_display_height1() {
        let mut gpu = GraphicsCard {
            current_register: LCD_VERTICAL_DISPLAY_HEIGHT1,
            ..Default::default()
        };

        gpu.write_data(0x01);

        assert_eq!(gpu.read_data(), 0x01);
    }

    #[test]
    fn read_write_data_lcd_vertical_non_display_period0() {
        let mut gpu = GraphicsCard {
            current_register: LCD_VERTICAL_NON_DISPLAY_PERIOD0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_lcd_vertical_non_display_period1() {
        let mut gpu = GraphicsCard {
            current_register: LCD_VERTICAL_NON_DISPLAY_PERIOD1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x01);
    }

    #[test]
    fn read_write_data_vsync_start_position0() {
        let mut gpu = GraphicsCard {
            current_register: VSYNC_START_POSITION0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_vsync_start_position1() {
        let mut gpu = GraphicsCard {
            current_register: VSYNC_START_POSITION1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x01);
    }

    #[test]
    fn read_write_data_font_write_cursor_horizontal0() {
        let mut gpu = GraphicsCard {
            current_register: FONT_WRITE_CURSOR_HORIZONTAL0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_font_write_cursor_horizontal1() {
        let mut gpu = GraphicsCard {
            current_register: FONT_WRITE_CURSOR_HORIZONTAL1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x03);
    }

    #[test]
    fn read_write_data_font_write_cursor_vertical0() {
        let mut gpu = GraphicsCard {
            current_register: FONT_WRITE_CURSOR_VERTICAL0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_font_write_cursor_vertical1() {
        let mut gpu = GraphicsCard {
            current_register: FONT_WRITE_CURSOR_VERTICAL1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x01);
    }

    #[test]
    fn read_write_data_horizontal_start_of_active_window0() {
        let mut gpu = GraphicsCard {
            current_register: HORIZONTAL_START_OF_ACTIVE_WINDOW0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_horizontal_start_of_active_window1() {
        let mut gpu = GraphicsCard {
            current_register: HORIZONTAL_START_OF_ACTIVE_WINDOW1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x03);
    }

    #[test]
    fn read_write_data_vertical_start_of_active_window0() {
        let mut gpu = GraphicsCard {
            current_register: VERTICAL_START_OF_ACTIVE_WINDOW0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_vertical_start_of_active_window1() {
        let mut gpu = GraphicsCard {
            current_register: VERTICAL_START_OF_ACTIVE_WINDOW1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x01);
    }

    #[test]
    fn read_write_data_horizontal_end_of_active_window0() {
        let mut gpu = GraphicsCard {
            current_register: HORIZONTAL_END_OF_ACTIVE_WINDOW0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_horizontal_end_of_active_window1() {
        let mut gpu = GraphicsCard {
            current_register: HORIZONTAL_END_OF_ACTIVE_WINDOW1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x03);
    }

    #[test]
    fn read_write_data_vertical_end_of_active_window0() {
        let mut gpu = GraphicsCard {
            current_register: VERTICAL_END_OF_ACTIVE_WINDOW0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_vertical_end_of_active_window1() {
        let mut gpu = GraphicsCard {
            current_register: VERTICAL_END_OF_ACTIVE_WINDOW1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x01);
    }

    #[test]
    fn read_write_data_memory_write_control0() {
        let mut gpu = GraphicsCard {
            current_register: MEMORY_WRITE_CONTROL0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), GraphicsCard::MEMORY_WRITE_CONTROL0_BITS);
    }

    #[test]
    fn read_write_data_blink_time_control() {
        let mut gpu = GraphicsCard {
            current_register: BLINK_TIME_CONTROL,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_background_color0() {
        let mut gpu = GraphicsCard {
            bits_per_pixel: Color { r: 3, g: 4, b: 5 },
            current_register: BACKGROUND_COLOR0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x07);
    }

    #[test]
    fn read_write_data_background_color1() {
        let mut gpu = GraphicsCard {
            bits_per_pixel: Color { r: 3, g: 4, b: 5 },
            current_register: BACKGROUND_COLOR1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x0F);
    }

    #[test]
    fn read_write_data_background_color2() {
        let mut gpu = GraphicsCard {
            bits_per_pixel: Color { r: 3, g: 4, b: 5 },
            current_register: BACKGROUND_COLOR2,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x1F);
    }

    #[test]
    fn read_write_data_foreground_color0() {
        let mut gpu = GraphicsCard {
            bits_per_pixel: Color { r: 3, g: 4, b: 5 },
            current_register: FOREGROUND_COLOR0,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x07);
    }

    #[test]
    fn read_write_data_foreground_color1() {
        let mut gpu = GraphicsCard {
            bits_per_pixel: Color { r: 3, g: 4, b: 5 },
            current_register: FOREGROUND_COLOR1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x0F);
    }

    #[test]
    fn read_write_data_foreground_color2() {
        let mut gpu = GraphicsCard {
            bits_per_pixel: Color { r: 3, g: 4, b: 5 },
            current_register: FOREGROUND_COLOR2,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x1F);
    }

    #[test]
    fn read_write_data_pll_control1() {
        let mut gpu = GraphicsCard {
            current_register: PLL_CONTROL1,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), GraphicsCard::PLL_CONTROL1_BITS);
    }

    #[test]
    fn read_write_data_pll_control2() {
        let mut gpu = GraphicsCard {
            current_register: PLL_CONTROL2,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), GraphicsCard::PLL_CONTROL2_BITS);
    }

    #[test]
    fn read_write_data_pwm1_control() {
        let mut gpu = GraphicsCard {
            current_register: PWM1_CONTROL,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), GraphicsCard::PWM1_CONTROL_BITS);
    }

    #[test]
    fn read_write_data_pwm1_duty_cycle() {
        let mut gpu = GraphicsCard {
            current_register: PWM1_DUTY_CYCLE,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0xFF);
    }

    #[test]
    fn read_write_data_memory_clear_control() {
        let mut gpu = GraphicsCard {
            current_register: MEMORY_CLEAR_CONTROL,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x0);
    }

    #[test]
    fn read_write_data_extra_general_purpose_io() {
        let mut gpu = GraphicsCard {
            current_register: EXTRA_GENERAL_PURPOSE_IO,
            ..Default::default()
        };

        gpu.write_data(0xFF);

        assert_eq!(gpu.read_data(), 0x01);
    }

    #[test]
    fn write_char_ready() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);

        gpu.write_char(b'h');

        assert_eq!(display.borrow().textbuffer0[0], b'h');
        assert_ne!(display.borrow().textbuffer1[0], b'h');
    }

    #[test]
    fn write_char_not_ready() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());

        gpu.write_char(b'h');

        assert_ne!(display.borrow().textbuffer0[0], b'h');
        assert_ne!(display.borrow().textbuffer1[0], b'h');
    }

    #[test]
    fn get_textbuffer() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0x80);
        gpu.active_window_end = Point { x: 799, y: 479 };
        gpu.text_cursor = Point { x: 16, y: 48 };

        gpu.write_char(b'h');

        assert_eq!(display.borrow().get_textbuffer()[302], b'h');
    }

    #[test]
    fn get_textbuffer_horizontal_unaligned_char() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0x80);
        gpu.active_window_end = Point { x: 799, y: 479 };
        gpu.text_cursor = Point { x: 15, y: 48 };

        gpu.write_char(b'h');

        assert_eq!(display.borrow().get_textbuffer(), vec![0; 3000]);
    }

    #[test]
    fn get_textbuffer_vertical_unaligned_char() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0x80);
        gpu.active_window_end = Point { x: 799, y: 479 };
        gpu.text_cursor = Point { x: 16, y: 49 };

        gpu.write_char(b'h');

        assert_eq!(display.borrow().get_textbuffer(), vec![0; 3000]);
    }

    #[test]
    fn get_textbuffer_display_off() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, POWER_AND_DISPLAY_CONTROL, 0);

        gpu.write_char(b'h');

        assert_eq!(display.borrow().get_textbuffer()[0], 0);
    }

    #[test]
    fn get_text() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0x80);
        gpu.active_window_end = Point { x: 799, y: 479 };

        gpu.write_char(b'h');
        gpu.write_char(b'e');
        gpu.write_char(b' ');
        gpu.write_char(b'l');
        gpu.write_char(b'l');
        gpu.write_char(b'o');
        gpu.write_char(b' ');
        gpu.text_cursor.x = 0;
        gpu.text_cursor.y += 2 * CHAR_HEIGHT;
        gpu.write_char(b'w');
        gpu.write_char(b'o');
        gpu.write_char(b'r');
        gpu.write_char(b'l');
        gpu.write_char(b'd');
        gpu.write_char(b'!');
        gpu.write_char(b' ');
        gpu.text_cursor.x = 0;
        gpu.text_cursor.y += 2 * CHAR_HEIGHT;
        gpu.write_char(b' ');

        assert_eq!(display.borrow().get_text(), "he llo\n\nworld!");
    }

    #[test]
    fn get_text_100_chars() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0x80);
        gpu.active_window_end = Point { x: 799, y: 479 };

        for _i in 0..100 {
            gpu.write_char(b'.');
        }

        assert_eq!(
            display.borrow().get_text(),
            "..................................................\
             .................................................."
        );
    }

    #[test]
    fn get_text_30_lines() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0x80);
        gpu.active_window_end = Point { x: 799, y: 479 };

        gpu.write_char(b'h');
        gpu.write_char(b'e');
        gpu.write_char(b'l');
        gpu.write_char(b'l');
        gpu.write_char(b'o');
        gpu.text_cursor.x = 0;
        gpu.text_cursor.y += 29 * CHAR_HEIGHT;
        gpu.write_char(b'w');
        gpu.write_char(b'o');
        gpu.write_char(b'r');
        gpu.write_char(b'l');
        gpu.write_char(b'd');
        gpu.write_char(b'!');

        assert_eq!(
            display.borrow().get_text(),
            "hello\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\nworld!"
        );
    }

    #[test]
    fn get_text_display_off() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, POWER_AND_DISPLAY_CONTROL, 0);

        gpu.write_char(b'h');

        assert_eq!(display.borrow().get_text(), "");
    }

    #[test]
    fn horizontal_active_window() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0x80);
        gpu.active_window_start = Point { x: 8, y: 0 };
        gpu.active_window_end = Point { x: 24, y: 479 };

        gpu.write_char(b'h');
        gpu.write_char(b'e');
        gpu.write_char(b'l');
        gpu.write_char(b'l');
        gpu.write_char(b'o');

        assert_eq!(display.borrow().get_text(), "hel\n lo");
    }

    #[test]
    fn inverted_horizontal_active_window() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0x80);
        gpu.active_window_start = Point { x: 80, y: 0 };
        gpu.active_window_end = Point { x: 8, y: 479 };

        gpu.write_char(b'h');
        gpu.write_char(b'e');
        gpu.write_char(b'l');
        gpu.write_char(b'l');
        gpu.write_char(b'o');

        assert_eq!(
            display.borrow().get_text(),
            "h\n\n          e\n          l\n          l\n          o"
        );
    }

    #[test]
    fn vertical_active_window() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0x80);
        gpu.active_window_start = Point { x: 0, y: 16 };
        gpu.active_window_end = Point { x: 16, y: 64 };
        gpu.text_cursor = Point { x: 8, y: 48 };

        gpu.write_char(b'h');
        gpu.write_char(b'e');
        gpu.write_char(b'l');
        gpu.write_char(b'l');
        gpu.write_char(b'o');

        assert_eq!(display.borrow().get_text(), "\nel\nlo\n h");
    }

    #[test]
    fn inverted_vertical_active_window() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0x80);
        gpu.active_window_start = Point { x: 0, y: 48 };
        gpu.active_window_end = Point { x: 8, y: 16 };

        gpu.write_char(b'h');
        gpu.write_char(b'e');
        gpu.write_char(b'l');

        assert_eq!(display.borrow().get_text(), "h\n\n\nl");
    }

    #[test]
    fn clear_active_window() {
        let display = Rc::new(RefCell::new(TextDisplay::default()));
        let mut gpu = GraphicsCard::new(display.clone());
        initialize(&mut gpu);
        gpu.active_window_start = Point { x: 0, y: 16 };
        gpu.active_window_end = Point { x: 8, y: 48 };
        write_register(&mut gpu, MEMORY_CLEAR_CONTROL, 0xC0);

        assert_eq!(display.borrow().get_textbuffer()[100], 0);
        assert_eq!(display.borrow().get_textbuffer()[200], 0);
        assert!(display.borrow().get_textbuffer()[101] != 0);
        assert!(display.borrow().get_textbuffer()[201] != 0);
        assert!(display.borrow().get_textbuffer()[300] != 0);
    }

    #[test]
    fn pio_state_changed_pb12_low() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);
        gpu.active_window_end = Point { x: 799, y: 479 };

        gpu.pio_state_changed(&[0, 0, 0, 0]);

        assert_eq!(gpu.active_window_end.x, 0);
        assert_eq!(gpu.active_window_end.y, 0);
    }

    #[test]
    fn pio_state_changed_pb12_high() {
        let mut gpu = GraphicsCard::default();
        initialize(&mut gpu);
        gpu.active_window_end = Point { x: 799, y: 479 };

        gpu.pio_state_changed(&[0, 1 << 12, 0, 0]);

        assert_eq!(gpu.active_window_end.x, 799);
        assert_eq!(gpu.active_window_end.y, 479);
    }
}
