use super::{cpu::io, encoder};
use volatile_register::RW;

mod pramdac;
mod prmcio;
mod prmvio;

#[repr(C)]
pub struct PMC {
    // 0x0
    pub boot: RW<u32>,

    // 0x4
    pad0: [u8; 0x100 - 0x4],

    // 0x100
    pub intr: RW<u32>,

    // 0x104
    pad1: [u8; 0x140 - 0x104],

    // 0x140
    pub intr_en: RW<u32>,

    // 0x144
    pad2: [u8; 0x200 - 0x144],

    // 0x200
    pub blk_en: RW<u32>,
}

#[repr(C)]
pub struct PFB {
    pad0: [u8; 0x200],

    // 0x200
    pub cfg0: RW<u32>,

    // 0x204
    pub cfg1: RW<u32>,

    // 0x20c
    pub cstat: RW<u32>,

    pad1: [u8; 0x410 - 0x210],

    // 0x410
    pub wbc: RW<u32>,
}

#[repr(C)]
pub struct PCRTC {
    pad0: [u8; 0x100],

    // 0x100
    pub intr: RW<u32>,
    pad1: [u8; 0x140 - 0x104],

    // 0x140
    pub intr_en: RW<u32>,
    pad2: [u8; 0x800 - 0x144],

    // 0x800
    pub start: RW<u32>,
    pub config: RW<u32>,
    pub raster: RW<u32>,
}

pub struct NV2A {
    pmc: *mut PMC,
    prmvio: *mut prmvio::PRMVIO,
    pfb: *mut PFB,
    pcrtc: *mut PCRTC,
    prmcio: *mut prmcio::PRMCIO,
    pramdac: *mut pramdac::PRAMDAC,
}

impl NV2A {
    fn pmc(&self) -> &PMC {
        unsafe { &*self.pmc }
    }

    fn pmc_mut(&mut self) -> &mut PMC {
        unsafe { &mut *self.pmc }
    }

    fn prmvio(&self) -> &prmvio::PRMVIO {
        unsafe { &*self.prmvio }
    }

    fn prmvio_mut(&mut self) -> &mut prmvio::PRMVIO {
        unsafe { &mut *self.prmvio }
    }

    fn pfb(&self) -> &PFB {
        unsafe { &*self.pfb }
    }

    fn pfb_mut(&mut self) -> &mut PFB {
        unsafe { &mut *self.pfb }
    }

    fn pcrtc(&self) -> &PCRTC {
        unsafe { &*self.pcrtc }
    }

    fn pcrtc_mut(&mut self) -> &mut PCRTC {
        unsafe { &mut *self.pcrtc }
    }

    fn prmcio(&self) -> &prmcio::PRMCIO {
        unsafe { &*self.prmcio }
    }

    fn prmcio_mut(&mut self) -> &mut prmcio::PRMCIO {
        unsafe { &mut *self.prmcio }
    }

    fn pramdac(&self) -> &pramdac::PRAMDAC {
        unsafe { &*self.pramdac }
    }

    fn pramdac_mut(&mut self) -> &mut pramdac::PRAMDAC {
        unsafe { &mut *self.pramdac }
    }

    pub fn set_video_mode(&mut self, vm: &encoder::VideoModeInfo) {
        unsafe {
            self.pramdac_mut().set_horizontal_video_mode(vm);
            self.prmcio_mut().set_horizontal_video_mode(vm);
            self.pramdac_mut().set_vertical_video_mode(vm);
            self.prmcio_mut().set_vertical_video_mode(vm);
        }
    }
}

pub fn get_device() -> NV2A {
    let base: u32 = 0xfd00_0000;
    NV2A {
        pmc: base as *mut PMC,
        prmvio: (base + 0xc_0000) as *mut prmvio::PRMVIO,
        pfb: (base + 0x10_0000) as *mut PFB,
        pcrtc: (base + 0x60_0000) as *mut PCRTC,
        prmcio: (base + 0x60_1000) as *mut prmcio::PRMCIO,
        pramdac: (base + 0x68_0000) as *mut pramdac::PRAMDAC,
    }
}

pub fn set_pcrtc_start_addr(gpu: &mut NV2A, addr: u32) {
    let pcrtc = gpu.pcrtc_mut();

    unsafe {
        pcrtc.start.write(addr);
    }
}

pub fn pfb_init(gpu: &mut NV2A, ram_128: bool) {
    let pfb = gpu.pfb_mut();

    let mut cfg0 = 0x0307_0003;
    if ram_128 {
        cfg0 |= 0x100;
    }

    unsafe {
        // FIXME: Document Cromwell magic numbers
        pfb.cfg0.write(cfg0);
        pfb.cfg1.write(0x1144_8000);
    }
}

pub fn init_gpu(gpu: &mut NV2A) {
    // FIXME: Support 128MB
    pfb_init(gpu, false);

    let encoder = encoder::Model::detect();
    let av_mode = encoder::AVMode::detect();

    gpu.prmcio_mut().lock(false);

    // Kill video
    unsafe {
        io::write_u8(0x80d3, 5);
    }

    gpu.pramdac_mut().init(&encoder);
    gpu.prmcio_mut().init();

    // FIXME: Set video mode in encoder
    let video_mode = av_mode
        .get_video_mode(&encoder)
        .expect("AV Mode should have video mode");
    gpu.set_video_mode(&video_mode);

    gpu.prmcio_mut().disable_palette();
    unsafe {
        gpu.prmcio_mut().write_reg(0x44, 0x3);
        gpu.prmvio_mut().init();
        gpu.prmcio_mut().write_reg(0x44, 0x0);
    }

    gpu.prmcio_mut().init_attr();

    unsafe {
        io::write_u8(0x80d8, 4);
        io::write_u8(0x80d6, 5);

        {
            let pcrtc = gpu.pcrtc_mut();
            pcrtc.intr_en.write(0x1);
            pcrtc.intr.write(0x1);
            pcrtc.intr_en.write(0x1);
        }

        {
            let pmc = gpu.pmc_mut();
            pmc.intr_en.write(0x1);
            pmc.intr.write(0x1);
            pmc.intr_en.write(0x1);
        }

        // Cromwell writes the fb address to pcrtc/pmc offset 0x8000
        // Stock kernel doesn't seem to do that, so it's omitted

        io::write_u8(0x80d3, 4);
        gpu.prmvio_mut().seq(0x1, 0x1);
    }

    // FIXME: Enable encoder output
}
