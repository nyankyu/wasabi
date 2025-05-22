#![no_std]
#![no_main]

use core::arch::asm;
use core::cmp::min;
use core::mem::offset_of;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::null_mut;

type EfiHandle = u64;
type EfiVoid = u8;
type Result<T> = core::result::Result<T, &'static str>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[must_use]
#[repr(u64)]
enum EfiStatus {
    Success = 0,
}

#[no_mangle]
fn efi_main(
    _image_handle: EfiHandle,
    efi_system_table: &EfiSystemTable,
) {
    let mut vram = init_vram(efi_system_table)
        .expect("Failed to init vram");

    let vw = vram.width();
    let vh = vram.height();
    fill_rect(&mut vram, 0x000000, 0, 0, vw, vh)
        .expect("fill_rect failed");
    fill_rect(&mut vram, 0xff0000, 32, 32, 32, 32)
        .expect("fill_rect failed");
    fill_rect(&mut vram, 0x00ff00, 64, 64, 64, 64)
        .expect("fill_rect failed");
    fill_rect(&mut vram, 0x0000ff, 128, 128, 128, 128)
        .expect("fill_rect failed");

    for i in 0..256 {
        let _ = draw_point(&mut vram, 0x010101 * i, i, i);
    }

    loop {
        hlt();
    }
}

pub fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        hlt();
    }
}

#[repr(C)]
struct EfiBootServicesTable {
    _reserved0: [u64; 40],
    locate_protocol: extern "win64" fn(
        protocol: *const EfiGuid,
        registration: *const EfiVoid,
        interface: *mut *mut EfiVoid,
    ) -> EfiStatus,
}
const _: () = assert!(
    offset_of!(EfiBootServicesTable, locate_protocol)
        == 320
);

#[repr(C)]
struct EfiSystemTable {
    _reserved0: [u64; 12],
    pub boot_services: &'static EfiBootServicesTable,
}
const _: () = assert!(
    offset_of!(EfiSystemTable, boot_services) == 96
);

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct EfiGuid {
    pub data0: u32,
    pub data1: u16,
    pub data2: u16,
    pub data3: [u8; 8],
}

const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EfiGuid =
    EfiGuid {
        data0: 0x9042a9de,
        data1: 0x23dc,
        data2: 0x4a38,
        data3: [
            0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a,
        ],
    };

#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocol<'a> {
    reserved: [u64; 3],
    pub mode: &'a EfiGraphicsOutputProtocolMode<'a>,
}

#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocolMode<'a> {
    pub max_mode: u32,
    pub mode: u32,
    pub info: &'a EfiGraphicsOutputProtocolPixelInfo,
    pub size_of_info: u64,
    pub frame_buffer_base: usize,
    pub frame_buffer_size: usize,
}

#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocolPixelInfo {
    version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    _padding: [u32; 5],
    pub pixels_per_scan_line: u32,
}
const _: () = assert!(
    size_of::<EfiGraphicsOutputProtocolPixelInfo>() == 36
);

fn locate_graphic_protocol<'a>(
    efi_system_table: &EfiSystemTable,
) -> Result<&'a EfiGraphicsOutputProtocol<'a>> {
    let mut graphics_output_protocol =
        null_mut::<EfiGraphicsOutputProtocol>();
    let status =
        (efi_system_table.boot_services.locate_protocol)(
            &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
            null_mut::<EfiVoid>(),
            &mut graphics_output_protocol
                as *mut *mut EfiGraphicsOutputProtocol
                as *mut *mut EfiVoid,
        );
    if status != EfiStatus::Success {
        return Err(
            "Failed to locate Graphics Output Protocol",
        );
    }
    Ok(unsafe { &*graphics_output_protocol })
}

trait Bitmap {
    fn bytes_per_pixel(&self) -> u32;
    fn pixels_per_line(&self) -> u32;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn buf_mut(&mut self) -> *mut u8;

    unsafe fn unchecked_pixel_at_mut(
        &mut self,
        x: u32,
        y: u32,
    ) -> *mut u32 {
        self.buf_mut().add(
            ((y * self.pixels_per_line() + x)
                * self.bytes_per_pixel())
                as usize,
        ) as *mut u32
    }

    fn pixel_at_mut(
        &mut self,
        x: u32,
        y: u32,
    ) -> Option<&mut u32> {
        if self.is_in_x_range(x) && self.is_in_y_range(y) {
            unsafe {
                Some(
                    &mut *(self
                        .unchecked_pixel_at_mut(x, y)),
                )
            }
        } else {
            None
        }
    }

    fn is_in_x_range(&self, px: u32) -> bool {
        px < min(self.width(), self.pixels_per_line())
    }
    fn is_in_y_range(&self, py: u32) -> bool {
        py < self.height()
    }
}

#[derive(Clone, Copy)]
struct VramBufferInfo {
    buf: *mut u8,
    width: u32,
    height: u32,
    pixels_per_line: u32,
}

impl Bitmap for VramBufferInfo {
    fn bytes_per_pixel(&self) -> u32 {
        4
    }

    fn pixels_per_line(&self) -> u32 {
        self.pixels_per_line
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn buf_mut(&mut self) -> *mut u8 {
        self.buf
    }
}

fn init_vram(
    efi_system_table: &EfiSystemTable,
) -> Result<VramBufferInfo> {
    let gp = locate_graphic_protocol(efi_system_table)?;

    Ok(VramBufferInfo {
        buf: gp.mode.frame_buffer_base as *mut u8,
        width: gp.mode.info.horizontal_resolution,
        height: gp.mode.info.vertical_resolution,
        pixels_per_line: gp.mode.info.pixels_per_scan_line,
    })
}

unsafe fn unchecked_draw_point<T: Bitmap>(
    buf: &mut T,
    color: u32,
    x: u32,
    y: u32,
) {
    *buf.unchecked_pixel_at_mut(x, y) = color;
}

fn draw_point<T: Bitmap>(
    buf: &mut T,
    color: u32,
    x: u32,
    y: u32,
) -> Result<()> {
    *(buf.pixel_at_mut(x, y).ok_or("Out of bounds")?) =
        color;
    Ok(())
}

fn fill_rect<T: Bitmap>(
    buf: &mut T,
    color: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<()> {
    if !buf.is_in_x_range(x)
        || !buf.is_in_y_range(y)
        || !buf.is_in_x_range(x + w - 1)
        || !buf.is_in_y_range(y + h - 1)
    {
        return Err("Out of bounds");
    }

    for i in 0..h {
        for j in 0..w {
            unsafe {
                unchecked_draw_point(
                    buf,
                    color,
                    x + j,
                    y + i,
                );
            }
        }
    }

    Ok(())
}
