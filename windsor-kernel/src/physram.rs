use crate::cpu::mmu::{ContiguousPhysicalMemory, PhysramAllocator};

fn free_bit(int: u32) -> Option<u8> {
    if int == 0xffff_ffff {
        return None;
    }

    for i in 0..32 {
        if (int >> i) & 1 == 0 {
            return Some(i);
        }
    }

    None
}

#[inline(always)]
fn get_bit(val: u32, bit: u8) -> bool {
    (val >> bit) & 1 == 1
}

#[inline(always)]
fn set_bit(val: u32, bit: u8) -> u32 {
    val | (1 << bit)
}

#[inline(always)]
fn clear_bit(val: u32, bit: u8) -> u32 {
    val & !(1 << bit)
}

const MB_BYTES: u32 = 1 << 20;
const FRAME_BYTES: u32 = 0x1000;

const FRAME_COUNT: usize = (64 * MB_BYTES / FRAME_BYTES) as usize;
const BITMAP_SIZE: usize = FRAME_COUNT / 32;

pub struct BitmapAlloc {
    map: [u32; BITMAP_SIZE],
}

impl BitmapAlloc {
    pub fn new() -> Self {
        Self {
            map: [0; BITMAP_SIZE],
        }
    }

    // paddr -> (map index, u32 index)
    fn addr_to_idx(paddr: u32) -> (usize, u8) {
        let frame_idx = paddr >> 12;

        let idx = frame_idx / 32;
        let bit = frame_idx % 32;
        (idx as usize, bit as u8)
    }
}

unsafe impl PhysramAllocator for BitmapAlloc {
    fn free(&mut self, paddr: u32) {
        let (idx, bit) = Self::addr_to_idx(paddr);

        let map = &mut self.map[idx];
        *map = clear_bit(*map, bit);
    }

    fn alloc(&mut self) -> Option<u32> {
        for (idx, map) in self.map.iter_mut().enumerate() {
            if let Some(bit) = free_bit(*map) {
                *map = set_bit(*map, bit);

                let idx = idx as u32;
                let bit = bit as u32;
                let frame_idx = idx * 32 + bit;
                return Some(frame_idx << 12);
            }
        }

        None
    }

    fn mark_allocated(&mut self, paddr: u32) {
        let (idx, bit) = Self::addr_to_idx(paddr);

        let map = &mut self.map[idx];
        *map = set_bit(*map, bit);
    }

    fn alloc_contiguous(&mut self, frames: u32) -> Option<ContiguousPhysicalMemory> {
        let mut start: Option<(usize, u8)> = None;

        for idx in 0..self.map.len() {
            let ent = self.map[idx];
            if ent == 0xffff_ffff {
                start = None;
                continue;
            }

            for bit in 0..32 {
                if get_bit(ent, bit) {
                    start = None;
                }

                let (start_idx, start_bit) = match start {
                    None => {
                        start = Some((idx, bit));
                        (idx, bit)
                    }
                    Some(start) => start,
                };

                let frames_allocated = (32 * (idx - start_idx)) as u32 + (bit as u32);
                if frames_allocated == frames {
                    let start_addr = 32 * start_idx as u32 + start_bit as u32;
                    let start_addr = start_addr << 12;
                    return Some(ContiguousPhysicalMemory::new(start_addr, frames));
                }
            }
        }

        None
    }

    fn free_contiguous(&mut self, mem: ContiguousPhysicalMemory) {
        let (start_idx, start_bit) = Self::addr_to_idx(mem.addr());

        for frame in 0..mem.frames() {
            let idx = start_idx + (frame / 32) as usize;
            let bit = start_bit + (frame % 32) as u8;

            let ent = &mut self.map[idx];
            *ent = clear_bit(*ent, bit);
        }
    }
}
