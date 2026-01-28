#![no_std]
#![no_main]

use core::panic::PanicInfo;

use wasabi::graphics::draw_test;
use wasabi::info;
use wasabi::init::init_basic_runtime;
use wasabi::init::init_paging;
use wasabi::println;
use wasabi::qemu::QemuExitCode;
use wasabi::qemu::exit_qemu;
use wasabi::uefi::EfiHandle;
use wasabi::uefi::EfiSystemTable;
use wasabi::uefi::init_vram;
use wasabi::uefi::locate_loaded_image_protocol;
use wasabi::x86::hlt;
use wasabi::x86::init_exceptions;
use wasabi::x86::trigger_debug_interrupt;

#[unsafe(no_mangle)]
fn efi_main(image_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    println!("Booting WasabiOS");

    let loaded_image_protocol =
        locate_loaded_image_protocol(image_handle, efi_system_table)
            .expect("Failed to get LoadedImageProtocol");
    println!("image_base: {:#018X}", loaded_image_protocol.image_base);
    println!("image_size: {:#018X}", loaded_image_protocol.image_size);

    //let mut vram = init_vram(efi_system_table).expect("Failed to init vram");
    //draw_test(&mut vram);

    let memory_map = init_basic_runtime(image_handle, efi_system_table);

    let (_gdt, _idt) = init_exceptions();
    info!("Exception initialized!");
    trigger_debug_interrupt();

    init_paging(&memory_map);
    info!("Now we are using out own page tables!");

    loop {
        hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Fail);
}
