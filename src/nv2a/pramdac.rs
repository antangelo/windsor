use volatile_register::RW;

use crate::encoder;

#[repr(C)]
pub struct PRAMDAC {
    pad0: [u8; 0x500],

    // 0x500
    pub nvpll: RW<u32>,

    // 0x504
    pub mpll: RW<u32>,
    pub vpll: RW<u32>,

    pad1: [u8; 0x514 - 0x50c],

    pub pll_test_counter: RW<u32>,

    pad2: [u8; 0x600 - 0x518],

    pub gen_ctl: RW<u32>,

    pad3: [u8; 0x630 - 0x604],

    pub fmt0: RW<u32>,

    pad4: [u8; 0x800 - 0x634],

    // 0x800
    pub vdisplay_end: RW<u32>,

    // 0x804
    pub vtotal: RW<u32>,

    // 0x808
    pub vcrtc: RW<u32>,

    // 0x80c
    pub vsync_start: RW<u32>,

    // 0x810
    pub vsync_end: RW<u32>,

    // 0x814
    pub vvalid_start: RW<u32>,

    // 0x818
    pub vvalid_end: RW<u32>,

    // 0x81c
    pad8: u32,

    // 0x820
    pub hdisplay_end: RW<u32>,

    // 0x824
    pub htotal: RW<u32>,

    // 0x828
    pub hcrtc: RW<u32>,

    // 0x82c
    pub hstart: RW<u32>,

    // 0x830
    pub hsync_end: RW<u32>,

    // 0x834
    pub hvalid_start: RW<u32>,

    // 0x838
    pub hvalid_end: RW<u32>,

    pad11: [u8; 0x84c - 0x83c],

    // 0x84c
    pub fmt1: RW<u32>,

    pad12: [u8; 0x880 - 0x850],

    // 0x880
    pub fmt2: RW<u32>,

    // 0x884 - Magic configuration registers
    pub r884: RW<u32>,
    pub r888: RW<u32>,
    pub r88c: RW<u32>,
    pub r890: RW<u32>,
    pub r894: RW<u32>,
    pub r898: RW<u32>,
    pub r89c: RW<u32>,

    pad13: [u8; 0x8c4 - 0x8a0],

    // 0x8c4
    pub fmt3: RW<u32>,
}

impl PRAMDAC {
    pub fn init(&mut self, enc: &encoder::Model) {
        unsafe {
            self.r884.write(0x0);
            self.r888.write(0x0);
            self.r88c.write(0x1000_1000);
            self.r890.write(0x1000_0000);
            self.r894.write(0x1000_0000);
            self.r898.write(0x1000_0000);
            self.r89c.write(0x1000_0000);

            if enc.is_xcalibur() {
                // Set YUV
                self.fmt0.write(0x2);
                self.fmt1.write(0x0080_1080);
                self.fmt2.write(0x2110_1100);
                self.fmt3.write(0x4080_1080);
            } else {
                // Set RGB
                self.fmt0.write(0x0);
                self.fmt1.write(0x0);
                self.fmt2.write(0x0);
                self.fmt3.write(0x0);
            }
        }
    }

    pub unsafe fn set_horizontal_video_mode(&mut self, vm: &encoder::VideoModeInfo) {
        self.hdisplay_end.write(vm.crtc_hend - 1);
        self.htotal.write(vm.nvhtotal);
        self.hcrtc.write(vm.width - 1);
        self.hvalid_start.write(0x0);
        self.hstart.write(vm.nvhstart);
        self.hsync_end.write(vm.nvhstart + 32);
        self.hvalid_end.write(vm.width - 1);
    }

    pub unsafe fn set_vertical_video_mode(&mut self, vm: &encoder::VideoModeInfo) {
        self.vdisplay_end.write(vm.height - 1);
        self.vtotal.write(vm.nvvtotal);
        self.vcrtc.write(vm.height - 1);
        self.vvalid_start.write(0x0);
        self.vsync_start.write(vm.nvvstart);
        self.vsync_end.write(vm.nvvstart + 3);
        self.vvalid_end.write(vm.height - 1);
    }
}
