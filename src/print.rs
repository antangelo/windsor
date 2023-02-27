use crate::{encoder, font, font2};

#[derive(Copy, Clone, Debug)]
pub struct RGBA(u8, u8, u8, u8);

pub const COLOR_WHITE: RGBA = RGBA(0xff, 0xff, 0xff, 0xff);

pub unsafe fn print_char2(
    dest: *mut u32,
    bytes_per_line: u32,
    rgba: &RGBA,
    ch: u8,
    ) -> u32 {
    let mask = if font2::FONT_VMIRROR > 0 {
        0x01
    } else {
        0x80
    };

    let font_loc = (ch as u32) * ((font2::FONT_WIDTH + 7) / 8) * font2::FONT_HEIGHT;
    let dest = dest as *mut u8;

    for y in 0..font2::FONT_HEIGHT {
        let font_loc = font_loc + y;
        let font = font2::FONT[font_loc as usize];

        let y_offset = y * bytes_per_line;
        let dest = dest.add(y_offset as usize);

        for x in 0..font2::FONT_WIDTH {
            let dest = dest.add(4 * (x as usize));

            if font & (mask >> x) == 0 {
                continue;
            }

            dest.byte_add(0).write(rgba.0);
            dest.byte_add(1).write(rgba.1);
            dest.byte_add(2).write(rgba.2);
            dest.byte_add(3).write(rgba.3);
        }
    }

    font2::FONT_WIDTH
}

unsafe fn print_char(
    dest: *mut u32,
    bytes_per_line: u32,
    rgba: &RGBA,
    ch: u8,
    ) -> u32 {
    if ch == b'\t' {
        let dw = (dest as u32) % bytes_per_line;
        let dw = (dw - 1) % (32 << 2);
        return ((32 << 2) - dw) >> 2;
    }

    let space_width = 5;
    if ch < b'!' || ch > b'~' {
        return space_width;
    }

    let start = font::FONT_WA_STARTS[(ch - b' ' - 1) as usize];
    let width = font::FONT_WA_STARTS[(ch - b' ') as usize];
    let width = width - start;

    let dest = dest as *mut u8;

    for y in 0..font::HEIGHT {
        let start = start + y * font::WIDTH;

        let y_offset = (y as usize) * (bytes_per_line as usize);
        let dest = dest.byte_add(y_offset);

        for x in 0..width {
            let start = start + x;
            let dest = dest.byte_add(4 * (x as usize));

            let b = font::FONT_BA_CHARSET[(start >> 1) as usize];
            let b = if start & 1 == 0 {
                b >> 4
            } else {
                b & 0xf
            };

            let b = b as u32;

            if true && b != 0 {
                dest.byte_add(0).write(((b * rgba.0 as u32) >> 4) as u8);
                dest.byte_add(1).write(((b * rgba.1 as u32) >> 4) as u8);
                dest.byte_add(2).write(((b * rgba.2 as u32) >> 4) as u8);
                dest.byte_add(3).write(rgba.3);
            }
        }
    }

    return width as u32;
}

unsafe fn print_string_bytes(
    dest: *mut u32,
    bytes_per_line: u32,
    rgba: &RGBA,
    string: &[u8],
    ) -> u32 {
    let mut width = 0;
    
    for ch in string.iter() {
        if *ch == 0 || *ch == b'\n' {
            return width;
        }

        width += print_char2(dest.add(width as usize), bytes_per_line, rgba, *ch);
    }

    width
}

pub struct VGAPrinter {
    fb_addr: *mut u32,
    fb_width: u32,
    bpp: u32,
    cursor_x: u32,
    cursor_y: u32,
    margin_x: u32,
}

impl VGAPrinter {
    pub fn new(fb_addr: *mut u32, vm: &encoder::VideoModeInfo) -> Self {
        Self {
            fb_addr,
            fb_width: vm.width,
            bpp: 4,
            cursor_x: 4 + vm.xmargin * 4,
            cursor_y: 1 + vm.ymargin,
            margin_x: vm.xmargin,
        }
    }

    pub fn print_string_bytes(&mut self, rgba: RGBA, string: &[u8]) {
        let width = unsafe {
            print_string_bytes(
                self.fb_addr.byte_add((self.cursor_y * self.fb_width * self.bpp + self.cursor_x) as usize),
                self.fb_width * self.bpp,
                &rgba,
                string,
                )
        };

        self.cursor_x += width * self.bpp;
        if self.cursor_x > ((self.fb_width - self.margin_x) * self.bpp) {
            self.cursor_y += font::HEIGHT as u32;
            self.cursor_x = self.margin_x * self.bpp;
        }
    }
}

