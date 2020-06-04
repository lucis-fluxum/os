use conquer_once::spin::Lazy;
use pc_keyboard::{layouts, DecodedKey, Error, HandleControl, Keyboard, ScancodeSet1};
use spinning_top::Spinlock;
use x86_64::instructions::port::Port;

pub(crate) static KEYBOARD: Lazy<Spinlock<Keyboard<layouts::Us104Key, ScancodeSet1>>> =
    Lazy::new(|| {
        Spinlock::new(Keyboard::new(
            layouts::Us104Key,
            ScancodeSet1,
            HandleControl::Ignore,
        ))
    });

pub(crate) fn decode_key() -> Result<Option<DecodedKey>, Error> {
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    let key: Option<DecodedKey> = keyboard
        .add_byte(scancode)?
        .and_then(|event| keyboard.process_keyevent(event));
    Ok(key)
}
