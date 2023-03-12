use proc_bitfield::bitfield;
use core::arch::asm;

bitfield!(
pub const struct GDTSegment(pub u64): FromRaw {
    pub segment: u64 @ ..,

    pub base_hi: u16 @ 56..=63,
    pub flags: u8 @ 52..=55,
    pub limit_hi: u8 @ 48..=51,

    pub present: bool @ 47,
    pub privilege: u8 @ 45..=46,
    pub descriptor_type: bool @ 44,
    pub executable: bool @ 43,
    pub dc: bool @ 42,
    pub rw: bool @ 41,
    pub access: bool @ 40,
    pub access_byte: u8 @ 40..=47,

    pub base_lo: u32 @ 16..=39,
    pub limit_lo: u32 @ 0..=15,
}
);

impl GDTSegment {
    pub const fn null() -> Self {
        GDTSegment(0)
    }

    pub const fn flat(is_kernel: bool, is_data: bool) -> Self {
        let mut seg = Self::null();

        let mut access_byte = 0;
        if is_kernel {
            access_byte |= 0x90;
        } else {
            access_byte |= 0xf0;
        }

        if is_data {
            access_byte |= 0x2;
        } else {
            access_byte |= 0xa;
        }

        seg.set_access_byte(access_byte);
        seg.set_flags(0xc);
        seg.set_limit_hi(0xff);
        seg.set_limit_lo(0xffff);

        seg
    }
}

#[repr(C)]
#[repr(packed)]
pub struct GDTDesc {
    size: u16,
    offset: u32,
    pad: u16,
}

pub static mut GDTR: GDTDesc = GDTDesc {
    size: 5 * 64 - 1,
    offset: 0,
    pad: 0,
};

pub const GDT: &'static [GDTSegment; 5] = &[
    GDTSegment::null(),
    GDTSegment::flat(true, false),
    GDTSegment::flat(true, true),
    GDTSegment::flat(false, false),
    GDTSegment::flat(false, true),
];

extern "C" {
    fn reload_segments();
}

pub fn load_gdt() {
    unsafe {
        lgdt(&mut GDTR, GDT);
        reload_segments();
    }
}

pub unsafe fn lgdt(gdtr: &'static mut GDTDesc, gdt: &'static [GDTSegment; 5]) {
    gdtr.offset = gdt.as_ptr() as u32;
    asm!("lgdt [eax]", in("eax") gdtr);
}
