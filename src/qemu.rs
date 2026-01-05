use crate::x86::hlt;
use crate::x86::write_io_port_u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x01, // QEMU will exit with status 3
    Fail = 0x02,    // QEMU will exit with status 5
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    const IO_PORT: u16 = 0xf4;
    write_io_port_u8(IO_PORT, exit_code as u8);
    loop {
        hlt();
    }
}
