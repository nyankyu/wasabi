#![no_std]
#![no_main]

use core::panic::PanicInfo;

use wasabi::graphics::draw_test;
use wasabi::info;
use wasabi::init::init_basic_runtime;
use wasabi::println;
use wasabi::qemu::QemuExitCode;
use wasabi::qemu::exit_qemu;
use wasabi::uefi::EfiHandle;
use wasabi::uefi::EfiSystemTable;
use wasabi::uefi::init_vram;
use wasabi::x86::hlt;
use wasabi::x86::init_exceptions;
use wasabi::x86::trigger_debug_interrupt;

#[unsafe(no_mangle)]
fn efi_main(image_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    let mut vram = init_vram(efi_system_table).expect("Failed to init vram");

    draw_test(&mut vram);

    let _memory_map = init_basic_runtime(image_handle, efi_system_table);

    println!("Booting WasabiOS");

    let cr3 = wasabi::x86::read_cr3();
    println!("cr3 = {cr3:#p}");
    let t = Some(unsafe { &*cr3 });
    println!("{t:?}");
    let t = t.and_then(|t| t.next_level(0));
    println!("{t:?}");
    let t = t.and_then(|t| t.next_level(0));
    println!("{t:?}");
    let t = t.and_then(|t| t.next_level(0));
    println!("{t:?}");

    let (_gdt, _idt) = init_exceptions();
    info!("Exception initialized!");
    trigger_debug_interrupt();

    loop {
        hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Fail);
}
