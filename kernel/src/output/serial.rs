use core::fmt::Write;

use lazy_static::lazy_static;
use spinning_top::Spinlock;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL_PORT: Spinlock<SerialPort> = unsafe {
        let mut s = SerialPort::new(0x3F8);
        s.init();
        Spinlock::new(s)
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::output::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    SERIAL_PORT
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}
