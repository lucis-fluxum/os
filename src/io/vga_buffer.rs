use core::fmt;

use spin::Mutex;
use volatile::Volatile;

use lazy_static::lazy_static;

use crate::io::vga_buffer::color::*;

pub mod color;
pub mod macros;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct VGABuffer {
    chars: [[Volatile<ColoredChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    row_position: usize,
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut VGABuffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ColoredChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                // Only supports ASCII and the additional bytes of code page 437
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Print '■' for unsupported characters
                _ => self.write_byte(0xfe)
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_position >= BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.row_position += 1;
        }

        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ColoredChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut VGABuffer) },
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
