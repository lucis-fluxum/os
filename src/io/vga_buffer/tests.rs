use crate::{println, serial_print, serial_println};
use core::fmt::Write;

use super::color::ColoredChar;
use super::WRITER;

#[test_case]
fn test_vga_buffer_println_single() {
    serial_print!("test_vga_buffer_println_single... ");
    println!("test_vga_buffer_println_single output");
    serial_println!("[ok]");
}

#[test_case]
fn test_vga_buffer_println_many() {
    serial_print!("test_vga_buffer_println_many... ");
    for _ in 0..200 {
        println!("test_vga_buffer_println_many output");
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_vga_buffer_println_bytes_match() {
    serial_print!("test_vga_buffer_println_bytes_match... ");

    let s = "Some test string that fits on a single line";

    // Avoid deadlocks in case an interrupt occurs while WRITER is locked
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        // Use writeln since we've already locked WRITER. Also, print a newline before the test
        // string so any existing text on the current line is removed.
        writeln!(writer, "\n{}", s).unwrap();
        let row_pos = writer.row_position;
        for (i, c) in s.bytes().enumerate() {
            let screen_char: ColoredChar = writer.buffer.chars[row_pos - 1][i].read();
            assert_eq!(screen_char.ascii_character, c);
        }
    });

    serial_println!("[ok]");
}
