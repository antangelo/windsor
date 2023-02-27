use crate::{encoder, font};

#[derive(Copy, Clone, Debug)]
pub struct RGBA(u8, u8, u8, u8);

pub const COLOR_WHITE: RGBA = RGBA(0xff, 0xff, 0xff, 0xff);

pub unsafe fn print_char2(
    dest: *mut u32,
    bytes_per_line: u32,
    rgba: &RGBA,
    ch: u8,
    ) -> u32 {
    let mask = if font::VMIRROR > 0 {
        0x01
    } else {
        0x80
    };

    let font_loc = (ch as u32) * ((font::WIDTH + 7) / 8) * font::HEIGHT;
    let dest = dest as *mut u8;

    for y in 0..font::HEIGHT {
        let font_loc = font_loc + y;
        let font = font::FONT[font_loc as usize];

        let y_offset = y * bytes_per_line;
        let dest = dest.add(y_offset as usize);

        for x in 0..font::WIDTH {
            let dest = dest.add(4 * (x as usize));
            let mask = if font::VMIRROR > 0 {
                mask << x
            } else {
                mask >> x
            };

            if font & mask == 0 {
                continue;
            }

            dest.byte_add(0).write(rgba.0);
            dest.byte_add(1).write(rgba.1);
            dest.byte_add(2).write(rgba.2);
            dest.byte_add(3).write(rgba.3);
        }
    }

    font::WIDTH
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

