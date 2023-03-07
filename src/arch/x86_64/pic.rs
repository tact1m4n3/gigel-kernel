use crate::sync::Mutex;

use super::io::Pio;

pub static PIC1: Mutex<Pic> = Mutex::new(Pic::new(Pio::new(0x20), true, 0x20));
pub static PIC2: Mutex<Pic> = Mutex::new(Pic::new(Pio::new(0xA0), false, 0x28));

pub fn init() {
    PIC1.lock().init();
    PIC2.lock().init();
}

pub struct Pic {
    cmd: Pio<u8>,
    data: Pio<u8>,
    is_master: bool,
    offset: u8,
}

impl Pic {
    pub const fn new(port: Pio<u8>, is_master: bool, offset: u8) -> Self {
        Self {
            cmd: port,
            data: port + 1,
            is_master,
            offset,
        }
    }

    pub fn init(&self) {
        self.cmd.write(0x11);
        self.data.write(self.offset);
        if self.is_master {
            self.data.write(0x04);
        } else {
            self.data.write(0x02);
        }
        self.data.write(0x1);
    }

    pub fn send_eoi(&self) {
        self.cmd.write(0x20);
    }
}
