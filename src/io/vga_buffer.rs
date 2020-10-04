use core::{
    fmt::{self, Write},
    mem,
};

use conquer_once::spin::Lazy;
use volatile::Volatile;
use x86_64::instructions::port::Port;

use self::color::*;
use crate::sync::Mutex;

pub(crate) mod color;

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

const CRT_ADDR_REGISTER: u16 = 0x3d4;
const CRT_DATA_REGISTER: u16 = 0x3d5;
const SET_CURSOR_POS_LOW: u8 = 0x0f;
const SET_CURSOR_POS_HIGH: u8 = 0x0e;

#[repr(transparent)]
struct RawVGABuffer {
    chars: [[Volatile<ColoredChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

struct VGABuffer {
    row_position: usize,
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut RawVGABuffer,
}

impl VGABuffer {
    fn write_byte(&mut self, byte: u8) {
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

    fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                // Only supports ASCII and the additional bytes of code page 437
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Print '■' for unsupported characters
                _ => self.write_byte(0xfe),
            }
        }
        if !string.is_empty() {
            self.update_cursor();
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

    fn update_cursor(&self) {
        let mut control_port = Port::new(CRT_ADDR_REGISTER);
        let mut data_port = Port::new(CRT_DATA_REGISTER);
        debug_assert!(self.row_position * BUFFER_WIDTH + self.column_position < u16::MAX as usize);
        let position_bytes =
            ((self.row_position * BUFFER_WIDTH + self.column_position) as u16).to_le_bytes();
        unsafe {
            control_port.write(SET_CURSOR_POS_LOW);
            data_port.write(position_bytes[0]);
            control_port.write(SET_CURSOR_POS_HIGH);
            data_port.write(position_bytes[1]);
        }
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

    fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
    }
}

impl fmt::Write for VGABuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

static VGA_BUFFER: Lazy<Mutex<VGABuffer>> = Lazy::new(|| {
    let buffer = Mutex::new(VGABuffer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut RawVGABuffer) },
    });
    // This fills the buffer with white-on-black spaces so the cursor shows up in blank areas
    buffer.lock().clear_screen();
    buffer
});

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Disable interrupts to prevent deadlock if an interrupt handler tries to print something
    // while VGA_BUFFER is locked
    x86_64::instructions::interrupts::without_interrupts(|| {
        VGA_BUFFER.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _print_colored(args: fmt::Arguments, color_code: ColorCode) {
    // Disable interrupts to prevent deadlock if an interrupt handler tries to print something
    // while VGA_BUFFER is locked
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut writer = VGA_BUFFER.lock();
        let old_color_code = mem::replace(&mut writer.color_code, color_code);
        writer.write_fmt(args).unwrap();
        let _ = mem::replace(&mut writer.color_code, old_color_code);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::println;

    #[test_case]
    fn test_vga_buffer_println_single() {
        println!("test_vga_buffer_println_single output");
    }

    #[test_case]
    fn test_vga_buffer_println_many() {
        for _ in 0..200 {
            println!("test_vga_buffer_println_many output");
        }
    }

    #[test_case]
    fn test_vga_buffer_println_bytes_match() {
        let s = "Some test string that fits on a single line";

        // Avoid deadlocks in case an interrupt occurs while VGA_BUFFER is locked
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut writer = VGA_BUFFER.lock();
            // Use writeln since we've already locked VGA_BUFFER. Also, print a newline before the
            // test string so any existing text on the current line is removed.
            writeln!(writer, "\n{}", s).unwrap();
            let row_pos = writer.row_position;
            for (i, c) in s.bytes().enumerate() {
                let screen_char: ColoredChar = writer.buffer.chars[row_pos - 1][i].read();
                assert_eq!(screen_char.ascii_character, c);
            }
        });
    }
}
