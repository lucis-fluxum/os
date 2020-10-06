use x86_64::instructions::port::Port;

const COMMAND_PORT: u16 = 0x64;
const DATA_PORT: u16 = 0x60;

// TODO: Use bitflags to check status register
// TODO: Timeout for reads/writes

pub struct PS2Controller {
    command_port: Port<u8>,
    data_port: Port<u8>,
}

impl PS2Controller {
    pub fn new() -> Self {
        Self {
            command_port: Port::new(COMMAND_PORT),
            data_port: Port::new(DATA_PORT),
        }
    }

    pub fn status(&mut self) -> u8 {
        unsafe { self.command_port.read() }
    }

    fn is_output_buffer_full(&mut self) -> bool {
        self.status() & 0b00000001 != 0
    }

    fn is_input_buffer_full(&mut self) -> bool {
        self.status() & 0b00000010 != 0
    }

    fn is_command_pending(&mut self) -> bool {
        self.is_input_buffer_full() && self.status() & 0b00001000 != 0
    }

    fn write_command(&mut self, command: u8) {
        while self.is_command_pending() {}
        unsafe { self.command_port.write(command) }
    }

    pub fn read_data(&mut self) -> u8 {
        while !self.is_output_buffer_full() {}
        unsafe { self.data_port.read() }
    }

    pub fn write_data(&mut self, data: u8) {
        while self.is_input_buffer_full() {}
        unsafe { self.data_port.write(data) }
    }

    pub fn controller_command(&mut self, command: u8, data: Option<u8>) {
        self.write_command(command);
        data.map(|data| self.write_data(data));
    }

    pub fn device_command(&mut self, command: u8, data: Option<u8>) {
        self.write_data(command);
        data.map(|data| self.write_data(data));
    }
}
