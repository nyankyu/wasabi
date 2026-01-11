use crate::serial::SerialPort;
use core::fmt;
use core::mem::size_of;
use core::slice;

pub fn global_print(args: fmt::Arguments) {
    let mut writer = SerialPort::default();
    fmt::write(&mut writer, args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::global_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ($crate::print!("[INFO]  {}:{:<3}: {}\n",
        file!(), line!(), format_args!($($arg)*)));
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ($crate::print!("\x1b[33m[WARN]  {}:{:<3}: {}\x1b[0m\n",
        file!(), line!(), format_args!($($arg)*)));
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ($crate::print!("\x1b[31m[ERROR]  {}:{:<3}: {}\x1b[0m\n",
        file!(), line!(), format_args!($($arg)*)));
}

fn hexdump_bytes(bytes: &[u8]) {
    for (i, chunk) in bytes.chunks(16).enumerate() {
        // address
        print!("{:08X}: ", i * 16);

        // hex values
        chunk.iter().for_each(|byte| print!("{byte:02X} "));
        // padding
        (0..(16 - chunk.len())).for_each(|_| print!("   "));

        // ascii values
        print!("|");
        chunk.iter().for_each(|byte| {
            print!(
                "{}",
                match *byte {
                    0x20..=0x7e => *byte as char,
                    _ => '.',
                }
            );
        });
        println!("|");
    }
}

pub fn hexdump<T: Sized>(data: &T) {
    hexdump_bytes(unsafe {
        slice::from_raw_parts(
            data as *const T as *const u8,
            size_of::<T>(),
        )
    });
}
