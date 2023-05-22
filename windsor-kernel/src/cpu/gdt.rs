use core::arch::asm;
use bitbybit::bitfield;
use arbitrary_int::{u2, u4, u24};

#[bitfield(u64, default: 0)]
pub struct GDTSegment {
    #[bits(56..=63, rw)]
    pub base_hi: u8,

    #[bits(52..=55, rw)]
    pub flags: u4,

    #[bits(48..=51, rw)]
    pub limit_hi: u4,

    #[bit(47, rw)]
    pub present: bool,

    #[bits(45..=46, rw)]
    pub privilege: u2,

    #[bit(44, rw)]
    pub descriptor_type: bool,

    #[bit(43, rw)]
    pub executable: bool,

    #[bit(42, rw)]
    pub dc: bool,

    #[bit(41, rw)]
    pub rw: bool,

    #[bit(40, rw)]
    pub access: bool,

    #[bits(40..=47, rw)]
    pub access_byte: u8,

    #[bits(16..=39, rw)]
    pub base_lo: u24,

    #[bits(0..=15, rw)]
    pub limit_lo: u16,
}

impl GDTSegment {
    pub const fn null() -> Self {
        Self::new_with_raw_value(0)
    }

    pub const fn flat(is_kernel: bool, is_data: bool) -> Self {
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

        Self::null()
            .with_access_byte(access_byte)
            .with_flags(u4::new(0xc))
            .with_limit_hi(u4::new(0xf))
            .with_limit_lo(0xffff)
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

pub unsafe fn lgdt(gdtr: &'static mut GDTDesc, gdt: &'static [GDTSegment; 5]) {
    gdtr.offset = gdt.as_ptr() as u32;
    asm!("lgdt [eax]", in("eax") gdtr);
}
