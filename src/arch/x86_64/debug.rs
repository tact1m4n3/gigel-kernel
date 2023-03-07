use core::fmt;

use crate::sync::MutexGuard;

use super::serial::{SerialPort, COM1};

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    Writer::new().write_fmt(args).unwrap();
}

pub struct Writer<'a> {
    serial: MutexGuard<'a, SerialPort>,
}

impl Writer<'_> {
    fn new() -> Self {
        Self {
            serial: COM1.lock(),
        }
    }

    pub fn write_byte(&mut self, c: u8) {
        self.serial.write_byte(c);
    }
}

impl fmt::Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            self.write_byte(c);
        }
        Ok(())
    }
}
