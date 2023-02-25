use crate::encoder;
use autopad::autopad;
use volatile_register::RW;

autopad!(
#[repr(C)]
pub struct PRMCIO {
    0x3c0 => pub vga_attr: RW<u8>,

    0x3d4 => pub vga_color_idx: RW<u8>,
    pub vga_color_data: RW<u8>,

    0x3da => pub vga_color_in_stat: RW<u8>,
}
);

impl PRMCIO {
    pub unsafe fn write_attr(&mut self, idx: u8, data: u8) {
        self.vga_attr.write(idx);
        self.vga_attr.write(data);
    }

    pub unsafe fn write_reg(&mut self, idx: u8, data: u8) {
        self.vga_color_idx.write(idx);
        self.vga_color_data.write(data);
    }

    pub unsafe fn read_reg(&self, idx: u8) -> u8 {
        self.vga_color_idx.write(idx);
        self.vga_color_data.read()
    }

    pub fn lock(&mut self, lock: bool) {
        let val = if lock { 0x99 } else { 0x57 };
        unsafe { self.write_reg(0x1f, val) };
    }

    pub fn disable_palette(&mut self) {
        unsafe {
            self.vga_color_in_stat.read();
            self.vga_attr.write(0x20);
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.write_reg(0x14, 0x0);
            self.write_reg(0x17, 0xe3);
            self.write_reg(0x19, 0x10);
            self.write_reg(0x1b, 0x05);
            self.write_reg(0x22, 0xff);
            self.write_reg(0x33, 0x11);
        }
    }

    pub fn init_attr(&mut self) {
        unsafe {
            self.write_attr(0, 0x01);
            self.write_attr(1, 0x02);
            self.write_attr(2, 0x03);
            self.write_attr(3, 0x04);
            self.write_attr(4, 0x05);
            self.write_attr(5, 0x06);
            self.write_attr(6, 0x07);
            self.write_attr(7, 0x08);
            self.write_attr(8, 0x09);
            self.write_attr(9, 0x0a);
            self.write_attr(10, 0x0b);
            self.write_attr(11, 0x0c);
            self.write_attr(12, 0x0d);
            self.write_attr(13, 0x0e);
            self.write_attr(14, 0x0f);
            self.write_attr(15, 0x01);
            self.write_attr(16, 0x4a);
            self.write_attr(17, 0x0f);
            self.write_attr(18, 0x00);
            self.write_attr(19, 0x00);
        }
    }

    // Safety: must be called in correct pramdac/vertical mode sequence
    pub unsafe fn set_horizontal_video_mode(&mut self, vm: &encoder::VideoModeInfo) {
        let hsync_start = vm.nvhtotal - 32;
        self.write_reg(4, (hsync_start / 8) as u8);

        // HSYNCEND
        let tmp = self.read_reg(5) & 0xe0;
        self.write_reg(5, tmp | (((hsync_start + 16) / 8 - 1) as u8 & 0x1f));

        // HTOTAL
        self.write_reg(0, (vm.nvhtotal / 8 - 5) as u8);

        // HBLANKSTART
        self.write_reg(2, (vm.crtc_hend / 8 - 1) as u8);

        // HBLANKEND
        let tmp = self.read_reg(3) & 0xe0;
        self.write_reg(3, tmp | ((vm.nvhtotal / 8 - 1) as u8 & 0x1f));

        let tmp = self.read_reg(5) & !0x80;
        self.write_reg(5, tmp | ((((vm.nvhtotal / 8 - 1) & 0x20) << 2) as u8));

        // HDISPEND
        let tmp = self.read_reg(0x17) & 0x7f;
        self.write_reg(0x17, tmp);
        self.write_reg(1, (vm.crtc_hend / 8 - 1) as u8);
        self.write_reg(2, (vm.crtc_hend / 8 - 1) as u8);
        let tmp = self.read_reg(0x17) & 0x7f;
        self.write_reg(0x17, tmp | 0x80);

        // LINESTRIDE
        let linestride = vm.width / 8 * vm.pixel_depth;
        let tmp = self.read_reg(0x19) & 0x1f;
        self.write_reg(0x19, tmp | ((linestride >> 3) & 0xe0) as u8);
    }

    // Safety: must be called in correct pramdac/horizontal mode sequence
    pub unsafe fn set_vertical_video_mode(&mut self, vm: &encoder::VideoModeInfo) {
        // VSYNC_START
        let tmp = self.read_reg(7) & 0x7b;
        let a = (vm.crtc_vstart >> 2) & 0x80;
        let b = (vm.crtc_vstart >> 6) & 0x4;
        self.write_reg(7, tmp | (a as u8) | (b as u8));
        self.write_reg(0x10, (vm.crtc_vstart & 0xff) as u8);

        // VTOTAL
        let tmp = self.read_reg(7) & 0xde;
        let a = (vm.crtc_vtotal >> 4) & 0x20;
        let b = (vm.crtc_vtotal >> 8) & 0x01;
        self.write_reg(7, tmp | (a as u8) | (b as u8));
        self.write_reg(6, (vm.crtc_vtotal & 0xff) as u8);

        // VBLANKEND
        let tmp = self.read_reg(0x16) & 0x80;
        self.write_reg(0x16, tmp | ((vm.crtc_vtotal & 0x7f) as u8));

        // VDISPEND
        let tmp = self.read_reg(7) & 0xbd;
        let a = ((vm.height - 1) >> 3) & 0x40;
        let b = ((vm.height - 1) >> 7) & 0x02;
        self.write_reg(7, tmp | (a as u8) | (b as u8));
        self.write_reg(0x12, ((vm.height - 1) & 0xff) as u8);

        // VBLANKSTART
        let tmp = self.read_reg(9) & 0xdf;
        self.write_reg(9, tmp | ((((vm.height - 1) >> 4) & 0x20) as u8));
        let tmp = self.read_reg(7) & 0xf7;
        self.write_reg(7, tmp | ((((vm.height - 1) >> 5) & 0x08) as u8));
        self.write_reg(0x15, ((vm.height - 1) & 0xff) as u8);

        // LINECOMP
        let linecomp = 0x3ff;
        let tmp = self.read_reg(7) & 0xef;
        self.write_reg(7, tmp | ((linecomp >> 4) & 0x10) as u8);
        let tmp = self.read_reg(9) & 0xbf;
        self.write_reg(9, tmp | ((linecomp >> 3) & 0x40) as u8);
        self.write_reg(0x18, (linecomp & 0xff) as u8);

        // REPAINT1
        let tmp = if vm.width < 1280 {
            0x04
        } else {
            0x00
        };
        self.write_reg(0x1a, tmp);

        let mut tmp = ((vm.nvhtotal / 8 - 5) & 0x40) >> 2;
        tmp |= ((vm.height - 1) & 0x400) >> 7;
        tmp |= (vm.crtc_vstart & 0x400) >> 8;
        tmp |= ((vm.height - 1) & 0x400) >> 9;
        tmp |= (vm.crtc_vstart & 0x400) >> 10;
        self.write_reg(0x25, tmp as u8);

        let tmp = core::cmp::min(vm.pixel_depth, 3) as u8;
        self.write_reg(0x28, tmp | 0x80);

        let mut tmp = self.read_reg(0x2d) & 0xe0;
        if vm.nvhtotal / 8 - 1 >= 260 {
            tmp |= 0x1;
        }
        self.write_reg(0x2d, tmp);
    }
}
