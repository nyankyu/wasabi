extern crate alloc;

use crate::result::Result;
use crate::uefi::EfiMemoryDescriptor;
use crate::uefi::EfiMemoryType;
use crate::uefi::MemoryMapHolder;
use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;
use alloc::boxed::Box;
use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::cmp::max;
use core::fmt;
use core::mem::size_of;
use core::ops::DerefMut;
use core::ptr::null_mut;

pub fn round_up_to_nearest_pow2(v: usize) -> Result<usize> {
    1usize
        .checked_shl(
            usize::BITS - v.wrapping_sub(1).leading_zeros(),
        )
        .ok_or("Out of range")
}

#[test_case]
fn round_up_to_nearest_pow2_test() {
    assert_eq!(round_up_to_nearest_pow2(0), Err("Out of range"));
    assert_eq!(round_up_to_nearest_pow2(1), Ok(1));
    assert_eq!(round_up_to_nearest_pow2(2), Ok(2));
    assert_eq!(round_up_to_nearest_pow2(3), Ok(4));
    assert_eq!(round_up_to_nearest_pow2(4), Ok(4));
    assert_eq!(round_up_to_nearest_pow2(5), Ok(8));
    assert_eq!(round_up_to_nearest_pow2(6), Ok(8));
    assert_eq!(round_up_to_nearest_pow2(7), Ok(8));
    assert_eq!(round_up_to_nearest_pow2(8), Ok(8));
    assert_eq!(round_up_to_nearest_pow2(9), Ok(16));
    assert_eq!(round_up_to_nearest_pow2(10), Ok(16));
    assert_eq!(round_up_to_nearest_pow2(15), Ok(16));
    assert_eq!(round_up_to_nearest_pow2(16), Ok(16));
    assert_eq!(round_up_to_nearest_pow2(17), Ok(32));
    assert_eq!(round_up_to_nearest_pow2(18), Ok(32));
}

struct Header {
    next_header: Option<Box<Header>>,
    size: usize,
    is_allocated: bool,
    _reserved: usize,
}

const HEADER_SIZE: usize = size_of::<Header>();

#[allow(clippy::assertions_on_constants)]
const _: () = assert!(HEADER_SIZE == 32);
const _: () = assert!(HEADER_SIZE.count_ones() == 1);

pub const LAYOUT_PAGE_4K: Layout = unsafe {
    Layout::from_size_align_unchecked(4096, 4096)
};

impl Header {
    fn can_provide(
        &self,
        size: usize,
        align: usize,
    ) -> bool {
        self.size >= size + HEADER_SIZE * 2 + align
    }

    fn is_allocated(&self) -> bool {
        self.is_allocated
    }

    fn end_address(&self) -> usize {
        self as *const Header as usize + self.size
    }

    unsafe fn new_from_address(
        address: usize,
    ) -> Box<Header> {
        let header = address as *mut Header;
        unsafe {
            header.write(Header {
                next_header: None,
                size: 0,
                is_allocated: false,
                _reserved: 0,
            });
            Box::from_raw(header)
        }
    }

    unsafe fn from_allocated_region(
        address: *mut u8,
    ) -> Box<Header> {
        let header = unsafe { address.sub(HEADER_SIZE) }
            as *mut Header;
        unsafe { Box::from_raw(header) }
    }

    fn provide(
        &mut self,
        size: usize,
        align: usize,
    ) -> Option<*mut u8> {
        let size = max(
            round_up_to_nearest_pow2(size).ok()?,
            HEADER_SIZE,
        );
        let align = max(align, HEADER_SIZE);
        if self.is_allocated()
            || !self.can_provide(size, align)
        {
            return None;
        }

        let mut size_used = 0;
        let allocated_address =
            (self.end_address() - size) & !(align - 1);

        let mut header_for_allocated = unsafe {
            Self::new_from_address(
                allocated_address - HEADER_SIZE,
            )
        };
        header_for_allocated.is_allocated = true;
        header_for_allocated.size = size + HEADER_SIZE;
        size_used += header_for_allocated.size;
        header_for_allocated.next_header =
            self.next_header.take();

        if header_for_allocated.end_address()
            != self.end_address()
        {
            let mut header_for_padding = unsafe {
                Self::new_from_address(
                    header_for_allocated.end_address(),
                )
            };
            header_for_padding.is_allocated = false;
            header_for_padding.size = self.end_address()
                - header_for_allocated.end_address();
            size_used += header_for_padding.size;
            header_for_padding.next_header =
                header_for_allocated.next_header.take();
            header_for_allocated.next_header =
                Some(header_for_padding);
        }

        assert!(self.size >= size_used + HEADER_SIZE);
        self.size -= size_used;
        self.next_header = Some(header_for_allocated);
        Some(allocated_address as *mut u8)
    }
}

impl Drop for Header {
    fn drop(&mut self) {
        panic!("Header should not be dropped!");
    }
}

impl fmt::Debug for Header {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            f,
            "Header @ {:#018X} {{ size: {:#018X}, is_allocated: {} }}",
            self as *const Header as usize,
            self.size,
            self.is_allocated()
        )
    }
}

pub struct FirstFitAllocator {
    first_header: RefCell<Option<Box<Header>>>,
}

#[global_allocator]
pub static ALLOCATOR: FirstFitAllocator =
    FirstFitAllocator {
        first_header: RefCell::new(None),
    };

unsafe impl Sync for FirstFitAllocator {}

unsafe impl GlobalAlloc for FirstFitAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc_with_options(layout)
    }

    unsafe fn dealloc(
        &self,
        ptr: *mut u8,
        _layout: Layout,
    ) {
        let mut region =
            unsafe { Header::from_allocated_region(ptr) };
        region.is_allocated = false;
        Box::leak(region);
    }
}

impl FirstFitAllocator {
    pub fn alloc_with_options(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        let mut header = self.first_header.borrow_mut();
        let mut header = header.deref_mut();
        loop {
            match header {
                Some(e) => match e
                    .provide(layout.size(), layout.align())
                {
                    Some(p) => break p,
                    None => {
                        header = e.next_header.borrow_mut();
                        continue;
                    }
                },
                None => {
                    break null_mut::<u8>();
                }
            }
        }
    }

    pub fn init_with_mmap(
        &self,
        memory_map: &MemoryMapHolder,
    ) {
        for e in memory_map.iter() {
            if e.memory_type()
                != EfiMemoryType::CONVENTIONAL_MEMORY
            {
                continue;
            }

            self.add_free_from_descriptor(e);
        }
    }

    fn add_free_from_descriptor(
        &self,
        desc: &EfiMemoryDescriptor,
    ) {
        let mut start_address =
            desc.physical_start() as usize;
        let mut size =
            desc.number_of_pages() as usize * 4096;
        if start_address == 0 {
            start_address += 4096;
            size = size.saturating_sub(4096);
        }
        if size <= 4096 {
            return;
        }

        let mut header = unsafe {
            Header::new_from_address(start_address)
        };
        header.next_header = None;
        header.is_allocated = false;
        header.size = size;
        let mut first_header =
            self.first_header.borrow_mut();
        let prev_last = first_header.replace(header);
        drop(first_header);

        let mut header = self.first_header.borrow_mut();
        header.as_mut().unwrap().next_header = prev_last;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test_case]
    fn malloc_iterate_free_and_alloc() {
        use alloc::vec::Vec;
        for i in 0..100 {
            let mut vec = Vec::new();
            vec.resize(i, 10);
        }
    }

    #[test_case]
    fn malloc_align() {
        let mut pointers = [null_mut::<u8>(); 100];
        for align in [1,2,4,8,16,32,4096] {
            for e in pointers.iter_mut() {
                *e = ALLOCATOR.alloc_with_options(
                    Layout::from_size_align(1234, align)
                    .expect("Failed to create Layout"),
                );

                assert!(*e as usize != 0);
                assert!((*e as usize) % align == 0);
            }
        }
    }
}