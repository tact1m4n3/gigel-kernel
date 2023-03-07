use crate::{arch::x86_64::io::Pio, sync::Mutex};

pub static COM1: Mutex<SerialPort> = Mutex::new(SerialPort::new(Pio::new(0x3f8)));
pub static COM2: Mutex<SerialPort> = Mutex::new(SerialPort::new(Pio::new(0x2f8)));

pub fn init() {
    COM1.lock().init();
    COM2.lock().init();
}

pub struct SerialPort {
    data: Pio<u8>,
    int_enable: Pio<u8>,
    fifo_ctrl: Pio<u8>,
    line_ctrl: Pio<u8>,
    modem_ctrl: Pio<u8>,
    line_status: Pio<u8>,
    _modem_status: Pio<u8>,
}

impl SerialPort {
    pub const fn new(port: Pio<u8>) -> Self {
        Self {
            data: port,
            int_enable: port + 1,
            fifo_ctrl: port + 2,
            line_ctrl: port + 3,
            modem_ctrl: port + 4,
            line_status: port + 5,
            _modem_status: port + 6,
        }
    }

    pub fn init(&self) {
        self.int_enable.write(0x00);
        self.line_ctrl.write(0x80);
        self.data.write(0x01);
        self.int_enable.write(0x00);
        self.line_ctrl.write(0x03);
        self.fifo_ctrl.write(0x07);
        self.modem_ctrl.write(0x0B);
        self.int_enable.write(0x01);
    }

    pub fn write_byte(&self, c: u8) {
        while (self.line_status.read() & 0x20) == 0 {}
        self.data.write(c);
    }
}
