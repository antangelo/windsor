use core::arch::asm;
use proc_bitfield::bitfield;

pub struct ContiguousPhysicalMemory {
    addr: u32,
    frames: u32,
}

impl ContiguousPhysicalMemory {
    pub fn new(addr: u32, frames: u32) -> Self {
        Self { addr, frames }
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }

    pub fn frames(&self) -> u32 {
        self.frames
    }
}

pub unsafe trait PhysramAllocator {
    fn alloc(&mut self) -> Option<u32>;
    fn free(&mut self, paddr: u32);
    fn mark_allocated(&mut self, paddr: u32);

    /// Allocates frames that are contiguous in physical
    /// memory
    fn alloc_contiguous(&mut self, frames: u32) -> Option<ContiguousPhysicalMemory>;
    fn free_contiguous(&mut self, mem: ContiguousPhysicalMemory);
}

bitfield!(
#[derive(Copy, Clone)]
pub const struct PDEDirect(pub u32): FromRaw {
    entry: u32 @ ..,

    pub present: bool @ 0,
    pub allow_writes: bool @ 1,
    pub allow_usermode: bool @ 2,
    pub writethrough: bool @ 3,
    pub disable_cache: bool @ 4,
    pub accessed: bool [read_only] @ 5,
    pub dirty: bool [read_only] @ 6,

    pub is_large_page: bool @ 7,

    ignored: u8 @ 8..=11,

    pub pat: bool @ 12,

    addr_39_32: u8 @ 13..=16,

    reserved: u8 @ 16..=21,

    addr_31_22: u32 @ 22..=31,
}
);

impl PDEDirect {
    pub unsafe fn set_paddr(&mut self, paddr: u32) {
        self.set_addr_39_32(0);
        self.set_addr_31_22(paddr >> 22);
    }

    pub fn get_paddr(&self) -> u32 {
        self.addr_31_22() << 22
    }
}

bitfield!(
#[derive(Copy, Clone)]
pub const struct PageTableEntry(pub u32): FromRaw {
    entry: u32 @ ..,

    pub present: bool @ 0,
    pub allow_writes: bool @ 1,
    pub allow_usermode: bool @ 2,
    pub writethrough: bool @ 3,
    pub disable_cache: bool @ 4,
    pub accessed: bool [read_only] @ 5,
    pub dirty: bool [read_only] @ 6,

    pub pat: bool @ 7,

    ignored: u8 @ 8..=11,

    frame_addr_raw: u32 @ 12..=31,
}
);

impl PageTableEntry {
    pub unsafe fn set_frame_addr(&mut self, paddr: u32) {
        self.set_frame_addr_raw(paddr >> 12);
    }
}

bitfield!(
#[derive(Copy, Clone)]
pub const struct PDETable(pub u32): FromRaw {
    entry: u32 @ ..,

    pub present: bool @ 0,
    pub allow_writes: bool @ 1,
    pub allow_usermode: bool @ 2,
    pub writethrough: bool @ 3,
    pub disable_cache: bool @ 4,
    pub accessed: bool [read_only] @ 5,
    pub dirty: bool [read_only] @ 6,

    pub is_large_page: bool @ 7,
    pub global: bool @ 8,

    ignored: u8 @ 9..=11,

    pte_paddr_intl: u32 @ 12..=31,
}
);

impl PDETable {
    unsafe fn set_pte_address(&mut self, pte_paddr: u32) {
        self.set_pte_paddr_intl(pte_paddr >> 12);
    }
}

pub enum PDEType {
    None,
    Table(PDETable),
    Direct(PDEDirect),
}

bitfield!(
#[derive(Copy, Clone)]
pub const struct PageDirectoryEntry(pub u32): FromRaw {
    entry: u32 @ ..,

    present: bool @ 0,
    is_large_page: bool @ 7,
}
);

impl From<PDETable> for PageDirectoryEntry {
    fn from(value: PDETable) -> Self {
        Self(value.with_ignored(0).with_is_large_page(false).entry())
    }
}

impl From<PDEDirect> for PageDirectoryEntry {
    fn from(value: PDEDirect) -> Self {
        Self(value.with_ignored(0).with_is_large_page(true).entry())
    }
}

impl PageDirectoryEntry {
    pub const fn to_pde(&self) -> PDEType {
        if !self.present() {
            return PDEType::None;
        }

        if self.is_large_page() {
            return PDEType::Direct(PDEDirect(self.entry()));
        }

        PDEType::Table(PDETable(self.entry()))
    }

    pub fn replace(&mut self, pde: PDEType) {
        match pde {
            PDEType::None => self.set_entry(0),
            PDEType::Table(pde) => {
                self.set_entry(pde.entry());
            }
            PDEType::Direct(pde) => {
                self.set_entry(pde.entry());
            }
        }
    }
}

const fn pde_offset_from_addr(addr: u32) -> usize {
    (addr as usize) >> 22
}

const fn pte_offset_from_addr(addr: u32) -> usize {
    ((addr as usize) >> 12) & 0x3ff
}

bitfield!(
pub const struct CR3(pub u32): FromRaw {
    register: u32 @ ..,

    pub writethrough: bool @ 3,
    pub disable_cache: bool @ 4,

    pde_paddr_raw: u32 @ 12..=31,
}
);

impl CR3 {
    pub unsafe fn current() -> Self {
        let cr3: u32;
        asm!("mov eax, cr3", out("eax") cr3);
        Self(cr3)
    }

    pub unsafe fn from_pde_paddr(paddr: u32) -> CR3 {
        Self::with_pde_paddr_raw(Self(0), paddr >> 12)
    }

    pub unsafe fn set_pde_paddr(&mut self, paddr: u32) {
        self.set_pde_paddr_raw(paddr >> 12);
    }

    pub fn get_pde_paddr(&self) -> u32 {
        self.pde_paddr_raw() << 12
    }
}

pub struct Mapping {
    cr3: CR3,
    pde: &'static mut [PageDirectoryEntry; 1024],
    mapping_pte: ActivePageTable<'static>,
}

pub struct ActivePageTable<'tab> {
    pte: &'tab mut [PageTableEntry; 1024],
}

/// Adjusts the bootstrap page tables to a more readily usable state
/// Also initializes physram allocator with currently used pages
unsafe fn bootstrap_setup(
    pde_paddr: u32,
    physram: &mut impl PhysramAllocator,
) -> (*mut PageDirectoryEntry, *mut PageTableEntry) {
    // Reserve some parts of the first 4MB in the allocator
    {
        // Zero page is used by the GPU
        physram.mark_allocated(0);
        physram.mark_allocated(pde_paddr);

        let kernel_data = crate::kernel_region();
        let kernel_paddr = kernel_data.as_ptr().mask(0x7fff_ffff) as u32;
        let kernel_pages = kernel_data.len().div_ceil(0x1000);
        for page in 0..kernel_pages {
            physram.mark_allocated(kernel_paddr + (page as u32) * 0x1000);
        }
    }

    // Setup PTE for 0xC000_0000 region
    let pte_paddr = physram.alloc().unwrap();

    // Restrict scope of the `pte` variable
    {
        // Bootstrap mappings have first 4MB at 0x8000_0000,
        // everything else is identity
        let pte = if pte_paddr > (1 << 22) {
            pte_paddr as *mut u32
        } else {
            (0x8000_0000 | pte_paddr) as *mut u32
        };
        let pte = &mut *(pte as *mut [PageTableEntry; 1024]);
        pte.fill(PageTableEntry(0));

        pte[0].set_frame_addr(pde_paddr);
        pte[0].set_allow_writes(true);
        pte[0].set_writethrough(true);
        pte[0].set_disable_cache(true);
        pte[0].set_present(true);

        pte[1].set_frame_addr(pte_paddr);
        pte[1].set_allow_writes(true);
        pte[1].set_writethrough(true);
        pte[1].set_disable_cache(true);
        pte[1].set_present(true);
    }

    let page_region_idx = pde_offset_from_addr(0xC000_0000);

    // Restrict `pde` scope
    {
        // Bootstrap page tables should be identity mapped with bit 31 set
        let pde = (0x8000_0000 | pde_paddr) as *mut PageDirectoryEntry;
        let pde = &mut *(pde as *mut [PageDirectoryEntry; 1024]);

        let mut page_region_pde = PDETable(0);

        page_region_pde.set_pte_address(pte_paddr);
        page_region_pde.set_allow_usermode(false);
        page_region_pde.set_allow_writes(true);
        page_region_pde.set_writethrough(true);
        page_region_pde.set_disable_cache(true);
        page_region_pde.set_present(true);
        pde[page_region_idx] = page_region_pde.into();
    }

    let pde = (0xC000_0000 | pde_paddr) as *mut PageDirectoryEntry;
    let pte = (0xC000_0000 | pte_paddr) as *mut PageTableEntry;
    (pde, pte)
}

impl Mapping {
    /// Creates a Mapping from the bootstrap page tables
    /// Safety:
    /// - cr3 must contain the bootstrap page tables
    pub unsafe fn from_bootstrap(physram: &mut impl PhysramAllocator) -> Self {
        let cr3 = CR3::current();
        let pde_paddr = cr3.get_pde_paddr();
        let (pde, pte) = bootstrap_setup(pde_paddr, physram);

        let pde = &mut *(pde as *mut [PageDirectoryEntry; 1024]);
        let pte = &mut *(pte as *mut [PageTableEntry; 1024]);
        let mapping_pte = ActivePageTable { pte };

        // FIXME: Remap kernel pages using PTE instead of large pages
        // FIXME: Unmap identity mapped RAM

        Self { cr3, pde, mapping_pte }
    }
}
