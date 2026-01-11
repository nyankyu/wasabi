#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;
use core::writeln;

use wasabi::error;
use wasabi::graphics::draw_test;
use wasabi::info;
use wasabi::init::init_basic_runtime;
use wasabi::print::hexdump;
use wasabi::println;
use wasabi::qemu::QemuExitCode;
use wasabi::qemu::exit_qemu;
use wasabi::uefi::EfiHandle;
use wasabi::uefi::EfiMemoryType;
use wasabi::uefi::EfiSystemTable;
use wasabi::uefi::VramTextWriter;
use wasabi::uefi::init_vram;
use wasabi::warn;
use wasabi::x86::hlt;

#[unsafe(no_mangle)]
fn efi_main(
    image_handle: EfiHandle,
    efi_system_table: &EfiSystemTable,
) {
    let mut vram = init_vram(efi_system_table)
        .expect("Failed to init vram");

    draw_test(&mut vram);

    let mut w = VramTextWriter::new(&mut vram);

    for i in 0..4 {
        writeln!(w, "This is line {i}").unwrap();
    }

    let memory_map =
        init_basic_runtime(image_handle, efi_system_table);

    /*
        let mut total_memory_pages = 0;

        for e in memory_map.iter() {
            if e.memory_type()
                != EfiMemoryType::CONVENTIONAL_MEMORY
            {
                continue;
            }
            total_memory_pages += e.number_of_pages();
            writeln!(w, "{e:?}").unwrap();
        }

        let total_memory_size =
            total_memory_pages * 4096 / 1024 / 1024;

        writeln!(
            w,
            "Total: {} pages = {} MiB",
            total_memory_pages, total_memory_size
        )
        .unwrap();
    */

    println!("Booting WasabiOS");
    info!("info");
    warn!("warn");
    error!("error");

    let v = [1, 2, 3, 4, 5];
    hexdump(&v);

    let s = ['a', 'b', 'c', 'd', 'e', 'f'];
    hexdump(&s);

    let t = 100u8;
    hexdump(&t);

    loop {
        hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Fail);
}
