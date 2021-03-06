#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_colored {
    ($color:expr, $($arg:tt)*) =>
        ($crate::io::vga::_print_colored(format_args!($($arg)*), $color));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_colored {
    ($color:expr, $($arg:tt)*) =>
        ($crate::print_colored!($color, "{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::io::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
