//! Reading input from the keyboard.

use conquer_once::spin::Lazy;
use log::debug;
use pc_keyboard::{layouts, DecodedKey, Error, HandleControl, Keyboard, ScancodeSet2};
use x86_64::instructions::port::Port;

use self::ps2::PS2Controller;
use crate::sync::Mutex;

mod ps2;

static KEYBOARD: Lazy<Mutex<Keyboard<layouts::Us104Key, ScancodeSet2>>> = Lazy::new(|| {
    Mutex::new(Keyboard::new(
        layouts::Us104Key,
        ScancodeSet2,
        HandleControl::Ignore,
    ))
});

/// Get the scancode for the pressed key from IO port `0x60`.
pub fn read_scancode() -> u8 {
    let mut port = Port::new(0x60);
    unsafe { port.read() }
}

/// Convert the scancode into a [`DecodedKey`] variant, if possible.
pub fn decode_key(scancode: u8) -> Result<Option<DecodedKey>, Error> {
    let mut keyboard = KEYBOARD.lock();
    let key: Option<DecodedKey> = keyboard
        .add_byte(scancode)?
        .and_then(|event| keyboard.process_keyevent(event));
    Ok(key)
}

pub fn initialize_ps2_controller() {
    let mut controller = PS2Controller::new();
    // We're doing low-level PS/2 controller commands and don't want to be interrupted
    x86_64::instructions::interrupts::without_interrupts(|| {
        debug!("Keyboard controller status: {:#010b}", controller.status());
        debug!("Controller self-test");
        controller.controller_command(0xaa, None);
        debug!("> {:#x}", controller.read_data());
        debug!("Disabling scancode set 1 translation");
        controller.controller_command(0x60, Some(0x21));
    });
}
