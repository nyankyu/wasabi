extern crate alloc;

use crate::allocator::ALLOCATOR;
use crate::uefi::EfiHandle;
use crate::uefi::EfiMemoryType::*;
use crate::uefi::EfiSystemTable;
use crate::uefi::MemoryMapHolder;
use crate::uefi::exit_from_efi_boot_services;
use crate::x86::PAGE_SIZE;
use crate::x86::PML4;
use crate::x86::PageAttr;
use crate::x86::write_cr3;
use alloc::boxed::Box;

pub fn init_basic_runtime(
    image_handle: EfiHandle,
    efi_system_table: &EfiSystemTable,
) -> MemoryMapHolder {
    let mut memory_map = MemoryMapHolder::new();
    exit_from_efi_boot_services(
        image_handle,
        efi_system_table,
        &mut memory_map,
    );
    ALLOCATOR.init_with_mmap(&memory_map);
    memory_map
}

pub fn init_paging(memory_map: &MemoryMapHolder) {
    let mut table = PML4::new();
    let end_of_mem = memory_map
        .iter()
        .filter(|e| {
            matches!(
                e.memory_type(),
                CONVENTIONAL_MEMORY | LOADER_CODE | LOADER_DATA
            )
        })
        .map(|e| e.physical_start() + e.number_of_pages() * (PAGE_SIZE as u64))
        .max()
        .unwrap_or(0)
        .max(0x1_0000_0000u64);

    table
        .create_mapping(0, end_of_mem, 0, PageAttr::ReadWriteKernel)
        .expect("Failed to create initial page mapping");

    unsafe {
        write_cr3(Box::into_raw(table));
    }
}
