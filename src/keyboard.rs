//! Reading input from the keyboard.

use conquer_once::spin::Lazy;
use pc_keyboard::{layouts, DecodedKey, Error, HandleControl, Keyboard, ScancodeSet2};
use ps2::{flags::ControllerConfigFlags, Controller};

use crate::sync::Mutex;

static KEYBOARD: Lazy<Mutex<Keyboard<layouts::Us104Key, ScancodeSet2>>> = Lazy::new(|| {
    Mutex::new(Keyboard::new(
        layouts::Us104Key,
        ScancodeSet2,
        HandleControl::Ignore,
    ))
});

/// Convert the scancode into a [`DecodedKey`] variant, if possible.
pub fn decode_key(scancode: u8) -> Result<Option<DecodedKey>, Error> {
    let mut keyboard = KEYBOARD.lock();
    let key: Option<DecodedKey> = keyboard
        .add_byte(scancode)?
        .and_then(|event| keyboard.process_keyevent(event));
    Ok(key)
}

pub fn initialize_ps2_controller() -> Result<(), ps2::error::ControllerError> {
    // We're doing low-level PS/2 controller commands and don't want to be interrupted
    x86_64::instructions::interrupts::without_interrupts(|| {
        // TODO: Steps 1 - 2
        let mut controller = unsafe { Controller::new() };

        // Step 3: Disable devices
        controller.disable_keyboard()?;
        controller.disable_mouse()?;

        // Step 4: Flush data buffer
        let _ = controller.read_data();

        // Step 5: Set config
        let mut config = controller.read_config()?;
        // Disable interrupts and scancode translation
        config.set(
            ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
                | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT
                | ControllerConfigFlags::ENABLE_TRANSLATE,
            false,
        );
        controller.write_config(config)?;

        // Step 6: Controller self-test
        controller.test_controller()?;
        // Write config again in case of controller reset
        controller.write_config(config)?;

        // Step 7: Determine if there are 2 devices
        let has_mouse = if config.contains(ControllerConfigFlags::DISABLE_MOUSE) {
            controller.enable_mouse()?;
            config = controller.read_config()?;
            // If mouse is working, this should now be unset
            !config.contains(ControllerConfigFlags::DISABLE_MOUSE)
        } else {
            false
        };
        // Disable mouse. If there's no mouse, this is ignored
        controller.disable_mouse()?;

        // Step 8: Interface tests
        let keyboard_works = controller.test_keyboard().is_ok();
        let mouse_works = has_mouse && controller.test_mouse().is_ok();

        // Step 9 - 10: Enable and reset devices
        config = controller.read_config()?;
        if keyboard_works {
            controller.enable_keyboard()?;
            config.set(ControllerConfigFlags::DISABLE_KEYBOARD, false);
            config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
            controller.keyboard().reset_and_self_test().unwrap();
        }
        if mouse_works {
            controller.enable_mouse()?;
            config.set(ControllerConfigFlags::DISABLE_MOUSE, false);
            config.set(ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT, true);
            controller.mouse().reset_and_self_test().unwrap();
            // This will start streaming events from the mouse
            controller.mouse().enable_data_reporting().unwrap();
        }

        // Write last configuration to enable devices and interrupts
        controller.write_config(config)?;

        // TODO: This initialization process only seems to work on QEMU

        Ok(())
    })
}
