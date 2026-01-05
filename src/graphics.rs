use crate::result::Result;
use core::cmp::min;

pub trait Bitmap {
    fn bytes_per_pixel(&self) -> u32;
    fn pixels_per_line(&self) -> u32;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn buf_mut(&mut self) -> *mut u8;

    /// # Safety
    #[allow(unused)]
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

    #[allow(unused)]
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

    #[allow(unused)]
    fn is_in_x_range(&self, px: u32) -> bool {
        px < min(self.width(), self.pixels_per_line())
    }

    #[allow(unused)]
    fn is_in_y_range(&self, py: u32) -> bool {
        py < self.height()
    }
}

unsafe fn unchecked_draw_point<T: Bitmap>(
    buf: &mut T,
    color: u32,
    x: u32,
    y: u32,
) {
    *buf.unchecked_pixel_at_mut(x, y) = color;
}

/// Draws a point on the bitmap with a specified color at
/// (x, y).
#[allow(unused)]
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

/// Fills a rectangle on the bitmap with a specified color.
/// (x, y) specifies the top-left corner of the rectangle,
/// and (w, h) specify its width and height respectively.
#[allow(unused)]
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
        || !buf.is_in_x_range(w)
        || !buf.is_in_y_range(h)
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

/// Draws a line on the bitmap with a specified color.
/// (x0, y0) and (x1, y1) specify the endpoints of the line.
#[allow(unused)]
fn draw_line<T: Bitmap>(
    buf: &mut T,
    color: u32,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
) -> Result<()> {
    if !buf.is_in_x_range(x0)
        || !buf.is_in_y_range(y0)
        || !buf.is_in_x_range(x1)
        || !buf.is_in_y_range(y1)
    {
        return Err("Out of bounds");
    }

    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = (y1 as i32 - y0 as i32).abs();
    let sx = (x1 as i32 - x0 as i32).signum();
    let sy = (y1 as i32 - y0 as i32).signum();

    if dx >= dy {
        (0..dx)
            .map(|rx| {
                (rx, increase_in_short_side(dx, dy, rx))
            })
            .for_each(|(rx, ry)| unsafe {
                unchecked_draw_point(
                    buf,
                    color,
                    (x0 as i32 + rx * sx) as u32,
                    (y0 as i32 + ry * sy) as u32,
                )
            });
    } else {
        (0..dy)
            .map(|ry| {
                (increase_in_short_side(dy, dx, ry), ry)
            })
            .for_each(|(rx, ry)| unsafe {
                unchecked_draw_point(
                    buf,
                    color,
                    (x0 as i32 + rx * sx) as u32,
                    (y0 as i32 + ry * sy) as u32,
                )
            });
    }

    Ok(())
}

/// Calculates the increase in the short side corresponding
/// to the increase in the long side.
/// Returns the integer closest (short_side / long_side) *
/// i.
///
/// long_side: length of long side
/// short_side: length of short side
/// i: increase in long side
fn increase_in_short_side(
    long_leng: i32,
    short_leng: i32,
    i: i32,
) -> i32 {
    if long_leng == 0 {
        0
    } else {
        (2 * short_leng * i + long_leng) / (2 * long_leng)
    }
}

fn lookup_font(c: char) -> Option<[[char; 8]; 16]> {
    const FONT_SOURCE: &str = include_str!("./font.txt");
    if let Ok(c) = u8::try_from(c) {
        let mut fi = FONT_SOURCE.split('\n');
        while let Some(line) = fi.next() {
            if let Some(line) = line.strip_prefix("0x") {
                if let Ok(idx) =
                    u8::from_str_radix(line, 16)
                {
                    if idx != c {
                        continue;
                    }
                    let mut font = [['*'; 8]; 16];
                    for (y, line) in
                        fi.clone().take(16).enumerate()
                    {
                        for (x, c) in
                            line.chars().enumerate()
                        {
                            if let Some(e) =
                                font[y].get_mut(x)
                            {
                                *e = c;
                            }
                        }
                    }
                    return Some(font);
                }
            }
        }
    }
    None
}

pub fn draw_font_fg<T: Bitmap>(
    buf: &mut T,
    x: u32,
    y: u32,
    color: u32,
    c: char,
) {
    let Some(font) = lookup_font(c) else {
        return;
    };
    for (dy, row) in font.iter().enumerate() {
        for (dx, pixel) in row.iter().enumerate() {
            if *pixel != '*' {
                continue;
            }
            let _ = draw_point(
                buf,
                color,
                x + dx as u32,
                y + dy as u32,
            );
        }
    }
}

#[allow(unused)]
fn draw_str_fg<T: Bitmap>(
    buf: &mut T,
    x: u32,
    y: u32,
    color: u32,
    s: &str,
) {
    s.chars().enumerate().for_each(|(i, c)| {
        draw_font_fg(buf, x + i as u32 * 8, y, color, c)
    });
}

#[allow(unused)]
pub fn draw_test<T: Bitmap>(buf: &mut T) {
    let vw = buf.width() - 1;
    let vh = buf.height() - 1;
    fill_rect(buf, 0x000000, 0, 0, vw, vh)
        .expect("fill_rect failed");
    fill_rect(buf, 0xff0000, 32, 32, 32, 32)
        .expect("fill_rect failed");
    fill_rect(buf, 0x00ff00, 64, 64, 64, 64)
        .expect("fill_rect failed");
    fill_rect(buf, 0x0000ff, 128, 128, 128, 128)
        .expect("fill_rect failed");

    draw_line(buf, 0xff00ff, 600, 0, 0, 600)
        .expect("draw_line failed");

    draw_line(buf, 0xffff00, 0, 0, vw, 0)
        .expect("draw_line failed");
    draw_line(buf, 0xffffff, 0, 0, 0, vh)
        .expect("draw_line failed");
    draw_line(buf, 0xffffff, vw, 0, vw, vh)
        .expect("draw_line failed");
    draw_line(buf, 0xffffff, 0, vh, vw, vh)
        .expect("draw_line failed");

    let (x0, y0) = (400, 500);
    for i in 0..=10 {
        draw_line(
            buf,
            0xaaaaaa,
            x0 - 100 + i * 20,
            y0 - 100,
            x0 - 100 + i * 20,
            y0 + 100,
        )
        .expect("draw_line failed");

        draw_line(
            buf,
            0xaaaaaa,
            x0 - 100,
            y0 - 100 + i * 20,
            x0 + 100,
            y0 - 100 + i * 20,
        )
        .expect("draw_line failed");
    }

    for i in 0..20 {
        draw_line(
            buf,
            0xff0000,
            x0,
            y0,
            x0 - 100 + i * 10,
            y0 + 100,
        )
        .expect("draw_line failed");
        draw_line(
            buf,
            0x00ff00,
            x0,
            y0,
            x0 + 100 - i * 10,
            y0 - 100,
        )
        .expect("draw_line failed");
        draw_line(
            buf,
            0x00ffff,
            x0,
            y0,
            x0 - 100,
            y0 - 100 + i * 10,
        )
        .expect("draw_line failed");
        draw_line(
            buf,
            0xffff00,
            x0,
            y0,
            x0 + 100,
            y0 + 100 - i * 10,
        )
        .expect("draw_line failed");
    }

    for (i, c) in "ABCDEFG".chars().enumerate() {
        draw_font_fg(
            buf,
            i as u32 * 16 + 256,
            i as u32 * 16,
            0xffffff,
            c,
        );
    }
}
