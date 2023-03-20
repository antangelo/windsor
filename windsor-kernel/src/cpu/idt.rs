use core::arch::asm;
use proc_bitfield::bitfield;

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
        Self { size, offset, pad: 0 }
    }

    pub const fn zero() -> Self {
        Self::new(0, 0)
    }
}

bitfield!(
    #[derive(Copy, Clone)]
pub const struct Entry(pub u64): FromRaw {
    pub entry: u64 @ ..,

    offset_upper: u16 @ 48..=63,

    present: bool @ 47,
    dpl: u8 @ 45..=46,
    zero: bool [read_only] @ 44,
    gate_type: u8 @ 40..=43,
    reserved: u8 [read_only] @ 32..=39,

    segment: u16 @ 16..=31,
    offset_lower: u16 @ 0..=15,
}
);

impl Entry {
    pub const fn new(offset: u32, gate_type: GateType, segment: u16) -> Self {
        let mut ent = Self(0);

        ent.set_offset_lower((offset & 0xffff) as u16);
        ent.set_offset_upper((offset >> 16) as u16);
        ent.set_gate_type(gate_type as u8);
        ent.set_segment(segment);
        ent.set_present(true);

        ent
    }
}

pub unsafe fn lidt(idtr: &'static Descriptor) {
    asm!("lidt [eax]", in("eax") idtr as *const _ as *const u8);
}
