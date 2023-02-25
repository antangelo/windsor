use super::{cpu::io, encoder};
use autopad::autopad;
use volatile_register::RW;

mod pramdac;
mod prmcio;
mod prmvio;

autopad!(
#[repr(C)]
pub struct PMC {
    pub boot: RW<u32>,
    0x100 => pub intr: RW<u32>,
    0x140 => pub intr_en: RW<u32>,
    0x200 => pub blk_en: RW<u32>,
}
);

autopad!(
#[repr(C)]
pub struct PFB {
    0x200 => pub cfg0: RW<u32>,
    pub cfg1: RW<u32>,
    pub cstat: RW<u32>,

    0x410 => pub wbc: RW<u32>,
}
);

impl PFB {
    fn init(&mut self, ram_128: bool) {
        let mut cfg0 = 0x0307_0003;
        if ram_128 {
            cfg0 |= 0x100;
        }

        unsafe {
            // FIXME: Document Cromwell magic numbers
            self.cfg0.write(cfg0);
            self.cfg1.write(0x1144_8000);
        }
    }
}

autopad!(
#[repr(C)]
pub struct PCRTC {
    0x100 => pub intr: RW<u32>,
    0x140 => pub intr_en: RW<u32>,

    0x800 => pub start: RW<u32>,
    pub config: RW<u32>,
    pub raster: RW<u32>,
}
);

autopad!(
#[repr(C)]
pub struct NV2A {
    pub pmc: PMC,
    0xc_0000 => prmvio: prmvio::PRMVIO,
    0x10_0000 => pfb: PFB,
    0x60_0000 => pub pcrtc: PCRTC,
    0x60_1000 => prmcio: prmcio::PRMCIO,
    0x68_0000 => pramdac: pramdac::PRAMDAC,
}
);

pub fn get_device<'a>() -> &'a mut NV2A {
    unsafe { &mut *(0xfd00_0000 as *mut NV2A) }
}

impl NV2A {
    pub fn set_video_mode(&mut self, vm: &encoder::VideoModeInfo) {
        unsafe {
            self.pramdac.set_horizontal_video_mode(vm);
            self.prmcio.set_horizontal_video_mode(vm);
            self.pramdac.set_vertical_video_mode(vm);
            self.prmcio.set_vertical_video_mode(vm);
        }
    }

    pub unsafe fn set_fb(&mut self, fbaddr: u32) {
        //let fbaddr = fbaddr & 0x0fff_ffff;
        self.pcrtc.start.write(fbaddr);
    }

    pub fn init(&mut self, fbaddr: u32) -> encoder::VideoModeInfo {
        // FIXME: Support 128MB
        unsafe { self.set_fb(fbaddr & 0x0fff_ffff); }
        self.pfb.init(false);

        let encoder = encoder::Model::detect();
        let av_mode = encoder::AVMode::detect();

        self.prmcio.lock(false);

        // Kill video
        unsafe {
            self.set_fb(fbaddr);
            io::write_u8(0x80d3, 5);
        }

        self.pramdac.init(&encoder);
        self.prmcio.init();

        // FIXME: Set video mode in encoder
        let video_mode = av_mode
            .get_video_mode(&encoder)
            .expect("AV Mode should have video mode");
        self.set_video_mode(&video_mode);

        self.prmcio.disable_palette();
        unsafe {
            self.prmcio.write_reg(0x44, 0x3);
            self.prmvio.init();
            self.prmcio.write_reg(0x44, 0x0);
        }

        self.prmcio.init_attr();

        unsafe {
            io::write_u8(0x80d8, 4);
            io::write_u8(0x80d6, 5);

            if true {
            self.pcrtc.intr_en.write(0x1);
            self.pcrtc.intr.write(0x1);
            self.pcrtc.intr_en.write(0x1);

            self.pmc.intr_en.write(0x1);
            self.pmc.intr.write(0x1);
            self.pmc.intr_en.write(0x1);
            }

            self.set_fb(fbaddr);

            // Cromwell writes the fb address to pcrtc/pmc offset 0x8000
            // Stock kernel doesn't seem to do that, so it's omitted

            io::write_u8(0x80d3, 4);
            self.prmvio.seq(0x1, 0x1);
        }

        // FIXME: Enable encoder output
        return video_mode;
    }
}
