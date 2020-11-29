//! Reading input from the keyboard.

use conquer_once::spin::Lazy;
use pc_keyboard::{layouts, DecodedKey, Error, HandleControl, Keyboard, ScancodeSet2};
use ps2::{flags::ControllerConfigFlags, Controller};
use x86_64::instructions::port::Port;

use crate::sync::Mutex;

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
    // We're doing low-level PS/2 controller commands and don't want to be interrupted
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut controller = unsafe { Controller::new() };
        controller
            .write_config(ControllerConfigFlags::SET_SYSTEM_FLAG)
            .unwrap();

        // TODO: This initialization process only seems to work on QEMU

        controller.enable_keyboard().unwrap();
        let mut keyboard = controller.keyboard();
        keyboard.set_defaults().unwrap();
        keyboard.enable_scanning().unwrap();

        controller.enable_mouse().unwrap();
        let mut mouse = controller.mouse();
        mouse.set_defaults().unwrap();
        mouse.enable_data_reporting().unwrap();

        controller
            .write_config(
                ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
                    | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT
                    | ControllerConfigFlags::SET_SYSTEM_FLAG,
            )
            .unwrap();
    });
}
