use super::{cpu::io, encoder};

const NV2A_BASE: *mut u32 = 0xfd00_0000 as *mut u32;

#[derive(Copy, Clone)]
enum NV2ABlock {
    PMC = 0x0,
    PRMVIO = 0xc_0000,
    PFB = 0x10_0000,
    PCRTC = 0x60_0000,
    PRMCIO = 0x60_1000,
    PRAMDAC = 0x68_0000,
}

impl NV2ABlock {
    fn to_pointer_u32(&self) -> *mut u32 {
        // Safety: All blocks are valid offsets from the base address
        // Assumes that the base address is properly mapped in the address space
        let value = *self as u32;
        unsafe { NV2A_BASE.byte_add(value as usize) }
    }

    fn to_pointer_u8(&self) -> *mut u8 {
        // Safety: All blocks are valid offsets from the base address
        // Assumes that the base address is properly mapped in the address space
        let value = *self as u32;
        unsafe { NV2A_BASE.byte_add(value as usize) as *mut u8 }
    }
}

#[derive(Copy, Clone)]
enum PCRTCReg {
    Start = 0x800,
}

impl PCRTCReg {
    fn to_pointer_u32(&self) -> *mut u32 {
        let pcrtc = NV2ABlock::PCRTC;
        let ptr: *mut u32 = pcrtc.to_pointer_u32();

        // Safety: All registers are valid offsets into the block
        unsafe { ptr.byte_add(*self as usize) as *mut u32 }
    }
}

pub fn set_pcrtc_start_addr(addr: u32) {
    unsafe {
        let pcrtc_start = PCRTCReg::Start;
        let pcrtc_start: *mut u32 = pcrtc_start.to_pointer_u32();
        pcrtc_start.write_volatile(addr);
    }
}

unsafe fn prmvio_seq(reg: u8, val: u8) {
    let prmvio = NV2ABlock::PRMVIO;
    let prmvio: *mut u8 = prmvio.to_pointer_u8();
    prmvio.byte_add(0x3c4).write_volatile(reg);
    prmvio.byte_add(0x3c5).write_volatile(val);
}

unsafe fn prmvio_graph(reg: u8, val: u8) {
    let prmvio = NV2ABlock::PRMVIO;
    let prmvio: *mut u8 = prmvio.to_pointer_u8();
    prmvio.byte_add(0x3ce).write_volatile(reg);
    prmvio.byte_add(0x3cf).write_volatile(val);
}

fn prmvio_init() {
    unsafe {
        prmvio_seq(0, 0x03);
        prmvio_seq(1, 0x21);
        prmvio_seq(2, 0x0f);
        prmvio_seq(3, 0x00);
        prmvio_seq(4, 0x06);
        prmvio_graph(0, 0);
        prmvio_graph(1, 0);
        prmvio_graph(2, 0);
        prmvio_graph(3, 0);
        prmvio_graph(4, 0);
        prmvio_graph(5, 0x40);
        prmvio_graph(6, 0x05);
        prmvio_graph(7, 0x0f);
        prmvio_graph(8, 0xff);
    }
}

pub fn pfb_init() {
    let pfb = NV2ABlock::PFB;
    let pfb_reg = pfb.to_pointer_u32();
    unsafe {
        // FIXME: Document Cromwell magic numbers
        // FIXME: Support 128MB RAM
        pfb_reg.byte_add(0x200).write_volatile(0x0307_0003);
        pfb_reg.byte_add(0x204).write_volatile(0x1144_8000);
    }
}

fn prmcio_lock(lock: bool) {
    let prmcio = NV2ABlock::PRMCIO;
    let prmcio_base: *mut u8 = prmcio.to_pointer_u8();
    let val = if lock { 0x99 } else { 0x57 };

    unsafe {
        prmcio_base.byte_add(0x3d4).write_volatile(0x1f);
        prmcio_base.byte_add(0x3d5).write_volatile(val);
    }
}

fn prmcio_init() {
    let prmcio = NV2ABlock::PRMCIO;
    let prmcio: *mut u8 = prmcio.to_pointer_u8();
    unsafe {
        prmcio.byte_add(0x14).write_volatile(0x0);
        prmcio.byte_add(0x17).write_volatile(0xe3);
        prmcio.byte_add(0x19).write_volatile(0x10);
        prmcio.byte_add(0x1b).write_volatile(0x05);
        prmcio.byte_add(0x22).write_volatile(0xff);
        prmcio.byte_add(0x33).write_volatile(0x11);
    }
}

unsafe fn prmcio_get_reg(idx: u8) -> u8 {
    let prmcio = NV2ABlock::PRMCIO;
    let prmcio: *mut u8 = prmcio.to_pointer_u8();
    prmcio.byte_add(0x3d4).write_volatile(idx);
    prmcio.byte_add(0x3d5).read_volatile()
}

unsafe fn prmcio_set_reg(idx: u8, val: u8) {
    let prmcio = NV2ABlock::PRMCIO;
    let prmcio: *mut u8 = prmcio.to_pointer_u8();
    prmcio.byte_add(0x3d4).write_volatile(idx);
    prmcio.byte_add(0x3d5).write_volatile(val);
}

unsafe fn prmcio_attr(idx: u8, val: u8) {
    let prmcio = NV2ABlock::PRMCIO;
    let prmcio: *mut u8 = prmcio.to_pointer_u8();
    let prmcio = prmcio.byte_add(0x3c0);
    prmcio.write_volatile(idx);
    prmcio.write_volatile(val);
}

fn prmcio_init_attr() {
    unsafe {
        prmcio_attr(0, 0x01);
        prmcio_attr(1, 0x02);
        prmcio_attr(2, 0x03);
        prmcio_attr(3, 0x04);
        prmcio_attr(4, 0x05);
        prmcio_attr(5, 0x06);
       	prmcio_attr(6, 0x07);
        prmcio_attr(7, 0x08);
        prmcio_attr(8, 0x09);
       	prmcio_attr(9, 0x0a);
        prmcio_attr(10, 0x0b);
        prmcio_attr(11, 0x0c);
      	prmcio_attr(12, 0x0d);
     	prmcio_attr(13, 0x0e);
     	prmcio_attr(14, 0x0f);
        prmcio_attr(15, 0x01);
       	prmcio_attr(16, 0x4a);
        prmcio_attr(17, 0x0f);
      	prmcio_attr(18, 0x00);
      	prmcio_attr(19, 0x00);
    }
}

fn prmcio_disable_palette() {
    let prmcio = NV2ABlock::PRMCIO;
    let prmcio: *mut u8 = prmcio.to_pointer_u8();
    unsafe {
        prmcio.byte_add(0x3d0).byte_add(0xa).read_volatile();
        prmcio.byte_add(0x3c0).write_volatile(0x20);
    }
}

fn pramdac_init(enc: &encoder::Model) {
    let pramdac = NV2ABlock::PRAMDAC;
    let pramdac: *mut u32 = pramdac.to_pointer_u32();

    unsafe {
        pramdac.byte_add(0x884).write_volatile(0x0);
        pramdac.byte_add(0x888).write_volatile(0x0);
        pramdac.byte_add(0x88c).write_volatile(0x1000_1000);
        pramdac.byte_add(0x890).write_volatile(0x1000_0000);
        pramdac.byte_add(0x894).write_volatile(0x1000_0000);
        pramdac.byte_add(0x898).write_volatile(0x1000_0000);
        pramdac.byte_add(0x89c).write_volatile(0x1000_0000);

        if enc.is_xcalibur() {
            // Set YUV
            pramdac.byte_add(0x880).write_volatile(0x2110_1100);
            pramdac.byte_add(0x630).write_volatile(0x2);
            pramdac.byte_add(0x84c).write_volatile(0x0080_1080);
            pramdac.byte_add(0x8c4).write_volatile(0x4080_1080);
        } else {
            // Set RGB
            pramdac.byte_add(0x880).write_volatile(0x0);
            pramdac.byte_add(0x630).write_volatile(0x0);
            pramdac.byte_add(0x84c).write_volatile(0x0);
            pramdac.byte_add(0x8c4).write_volatile(0x0);
        }
    }
}

fn set_video_mode(vm: &encoder::VideoModeInfo) {
    let pramdac = NV2ABlock::PRAMDAC;
    let pramdac: *mut u32 = pramdac.to_pointer_u32();

    unsafe {
        pramdac.byte_add(0x820).write_volatile(vm.crtc_hend - 1);
        pramdac.byte_add(0x824).write_volatile(vm.nvhtotal);
        pramdac.byte_add(0x828).write_volatile(vm.width - 1);
        pramdac.byte_add(0x834).write_volatile(0x0);
        pramdac.byte_add(0x82c).write_volatile(vm.nvhstart);
        pramdac.byte_add(0x830).write_volatile(vm.nvhstart + 32);
        pramdac.byte_add(0x838).write_volatile(vm.width - 1);

        let hsync_start = vm.nvhtotal - 32;
        prmcio_set_reg(4, (hsync_start / 8) as u8);
        
        // HSYNCEND
        let tmp = prmcio_get_reg(5) & 0xe0;
        prmcio_set_reg(5, tmp | ((hsync_start + 16) / 8 - 1) as u8 & 0x1f);

        // HTOTAL
        prmcio_set_reg(0, (vm.nvhtotal / 8 - 5) as u8);

        // HBLANKSTART
        prmcio_set_reg(2, (vm.crtc_hend / 8 - 1) as u8);
        
        // HBLANKEND
        let tmp = prmcio_get_reg(3) & 0xe0;
        prmcio_set_reg(3, tmp | ((vm.nvhtotal / 8 - 1) as u8 & 0x1f));

        let tmp = prmcio_get_reg(5) & !0x80;
        prmcio_set_reg(5, tmp | ((((vm.nvhtotal / 8 - 1) & 0x20) << 2) as u8));

        // HDISPEND
        let tmp = prmcio_get_reg(0x17) & 0x7f;
        prmcio_set_reg(0x17, tmp);
        prmcio_set_reg(1, (vm.crtc_hend / 8 - 1) as u8);
        prmcio_set_reg(2, (vm.crtc_hend / 8 - 1) as u8);
        let tmp = prmcio_get_reg(0x17) & 0x7f;
        prmcio_set_reg(0x17, tmp | 0x80);

        // LINESTRIDE
        let linestride = vm.width / 8 * vm.pixel_depth;
        let tmp = prmcio_get_reg(0x19) & 0x1f;
        prmcio_set_reg(0x19, tmp | ((linestride >> 3) & 0xe0) as u8);

        pramdac.byte_add(0x800).write_volatile(vm.height - 1);
        pramdac.byte_add(0x804).write_volatile(vm.nvvtotal);
        pramdac.byte_add(0x808).write_volatile(vm.height - 1);
        pramdac.byte_add(0x814).write_volatile(0x0);
        pramdac.byte_add(0x80c).write_volatile(vm.nvvstart);
        pramdac.byte_add(0x810).write_volatile(vm.nvvstart + 3);
        pramdac.byte_add(0x818).write_volatile(vm.height - 1);

        // VSYNC_START
        let tmp = prmcio_get_reg(7) & 0x7b;
        let a = (vm.crtc_vstart >> 2) & 0x80;
        let b = (vm.crtc_vstart >> 6) & 0x4;
        prmcio_set_reg(7, tmp | (a as u8) | (b as u8));
        prmcio_set_reg(0x10, (vm.crtc_vstart & 0xff) as u8);

        // VTOTAL
        let tmp = prmcio_get_reg(7) & 0xde;
        let a = (vm.crtc_vtotal >> 4) & 0x20;
        let b = (vm.crtc_vtotal >> 8) & 0x01;
        prmcio_set_reg(7, tmp | (a as u8) | (b as u8));
        prmcio_set_reg(6, (vm.crtc_vtotal & 0xff) as u8);

        // VBLANKEND
        let tmp = prmcio_get_reg(0x16) & 0x80;
        prmcio_set_reg(0x16, tmp | ((vm.crtc_vtotal & 0x7f) as u8));

        // VDISPEND
        let tmp = prmcio_get_reg(7) & 0xbd;
        let a = ((vm.height - 1) >> 3) & 0x40;
        let b = ((vm.height - 1) >> 7) & 0x02;
        prmcio_set_reg(7, tmp | (a as u8) | (b as u8));
        prmcio_set_reg(0x12, ((vm.height - 1) & 0xff) as u8);

        // VBLANKSTART
        let tmp = prmcio_get_reg(9) & 0xdf;
        prmcio_set_reg(9, tmp | ((((vm.height - 1) >> 4) & 0x20) as u8));
        let tmp = prmcio_get_reg(7) & 0xf7;
        prmcio_set_reg(7, tmp | ((((vm.height - 1) >> 5) & 0x08) as u8));
        prmcio_set_reg(0x15, ((vm.height - 1) & 0x08) as u8);

        // LINECOMP
        let linecomp = 0x3ff;
        let tmp = prmcio_get_reg(7) & 0xef;
        prmcio_set_reg(7, tmp | ((linecomp >> 4) & 0x10) as u8);
        let tmp = prmcio_get_reg(9) & 0xbf;
        prmcio_set_reg(9, tmp | ((linecomp >> 3) & 0x40) as u8);
        prmcio_set_reg(0x18, (linecomp & 0xff) as u8);

        // REPAINT1
        let mut tmp = ((vm.nvhtotal / 8 - 5) & 0x40) >> 2;
        tmp |= ((vm.height - 1) * 0x400) >> 7;
        tmp |= (vm.crtc_vstart & 0x400) >> 8;
        tmp |= ((vm.height - 1) * 0x400) >> 9;
        tmp |= (vm.crtc_vstart & 0x400) >> 10;
        prmcio_set_reg(0x25, tmp as u8);

        let tmp = core::cmp::max(vm.pixel_depth, 3) as u8;
        prmcio_set_reg(0x28, tmp | 0x80);

        let mut tmp = prmcio_get_reg(0x2d) & 0xe0;
        if vm.nvhtotal / 8 - 1 >= 260 {
            tmp |= 0x1;
        }
        prmcio_set_reg(0x2d, tmp);
    }
}

pub fn init_gpu() {
    pfb_init();

    let encoder = encoder::Model::detect();
    let av_mode = encoder::AVMode::detect();

    prmcio_lock(false);

    // Kill video
    unsafe {
        io::write_u8(0x80d3, 5);
    }

    pramdac_init(&encoder);
    prmcio_init();

    // FIXME: Set video mode in encoder
    let video_mode = av_mode
        .get_video_mode(&encoder)
        .expect("AV Mode should have video mode");
    set_video_mode(&video_mode);

    prmcio_disable_palette();
    unsafe {
        prmcio_set_reg(0x44, 0x3);
        prmvio_init();
        prmcio_set_reg(0x44, 0x0);
    }

    prmcio_init_attr();

    unsafe {
        io::write_u8(0x80d8, 4);
        io::write_u8(0x80d6, 5);
    }

    let pmc = NV2ABlock::PMC;
    let pcrtc = NV2ABlock::PCRTC;
    let pmc: *mut u32 = pmc.to_pointer_u32();
    let pcrtc: *mut u32 = pcrtc.to_pointer_u32();
    unsafe {
        pcrtc.byte_add(0x140).write_volatile(0x1);
        pcrtc.byte_add(0x100).write_volatile(0x1);
        pcrtc.byte_add(0x140).write_volatile(0x1);
        pmc.byte_add(0x140).write_volatile(0x1);
        pmc.byte_add(0x100).write_volatile(0x1);
        pmc.byte_add(0x140).write_volatile(0x1);
        
        pcrtc.byte_add(0x8000).write_volatile(0xf0000000 | (64 * 0x10_0000 - 0x40_0000));
        pmc.byte_add(0x8000).write_volatile(0xf0000000 | (64 * 0x10_0000 - 0x40_0000));
        io::write_u8(0x80d3, 4);
        prmvio_seq(0x01, 0x01);
    }

    // FIXME: Enable encoder output
}
