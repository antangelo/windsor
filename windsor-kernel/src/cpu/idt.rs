use core::arch::asm;
use bitbybit::bitfield;
use arbitrary_int::{u2, u4};

pub enum GateType {
    Interrupt = 0xe,
    Trap = 0xf,
}

#[repr(C)]
#[repr(packed)]
pub struct Descriptor {
    pub size: u16,
    pub offset: u32,
    pub pad: u16,
}

impl Descriptor {
    pub const fn new(size: u16, offset: u32) -> Self {
        Self {
            size,
            offset,
            pad: 0,
        }
    }

    pub const fn zero() -> Self {
        Self::new(0, 0)
    }
}

#[bitfield(u64)]
pub struct Entry {
    #[bits(48..=63, rw)]
    pub offset_upper: u16,

    #[bit(47, rw)]
    pub present: bool,

    #[bits(45..=46, rw)]
    pub dpl: u2,

    #[bit(44, r)]
    zero: bool,

    #[bits(40..=43, rw)]
    pub gate_type: u4,

    #[bits(32..=39, r)]
    reserved: u8,

    #[bits(16..=31, rw)]
    pub segment: u16,

    #[bits(0..=15, rw)]
    pub offset_lower: u16,
}

impl Entry {
    pub const fn new(offset: u32, gate_type: GateType, segment: u16) -> Self {
        Self::new_with_raw_value(0)
            .with_offset_lower((offset & 0xffff) as u16)
            .with_offset_upper((offset >> 16) as u16)
            .with_gate_type(u4::new(gate_type as u8))
            .with_segment(segment)
            .with_present(true)
    }
}

pub unsafe fn lidt(idtr: &'static Descriptor) {
    asm!("lidt [eax]", in("eax") idtr as *const _ as *const u8);
}
