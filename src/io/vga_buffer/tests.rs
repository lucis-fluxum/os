use crate::{println, serial_print, serial_println};

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
    println!("{}", s);
    let row_pos = WRITER.lock().row_position;
    for (i, c) in s.bytes().enumerate() {
        let screen_char: ColoredChar = WRITER.lock().buffer.chars[row_pos - 1][i].read();
        assert_eq!(screen_char.ascii_character, c);
    }

    serial_println!("[ok]");
}
