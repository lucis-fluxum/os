#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExitCode {
    Success = 0x10,
    Failed = 0x11,
}

const PORT_ADDR: u16 = 0xf4;

pub fn exit(exit_code: ExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(PORT_ADDR);
        port.write(exit_code as u32);
    }
}
