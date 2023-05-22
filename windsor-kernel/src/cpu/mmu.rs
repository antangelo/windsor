use core::arch::asm;
use proc_bitfield::bitfield;

unsafe fn invalidate_page(vaddr: *const u8) {
    asm!("invlpg {}", in(reg) vaddr);
}

unsafe fn invalidate_all() {
    asm!("
    wbinvd
    mov eax, cr3
    mov cr3, eax
    ", out("eax") _);
}

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
pub struct PDEDirect(pub u32) {
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
pub struct PageTableEntry(pub u32) {
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
pub struct PDETable(pub u32) {
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

    pt_paddr_intl: u32 @ 12..=31,
}
);

impl PDETable {
    unsafe fn set_pt_address(&mut self, pte_paddr: u32) {
        self.set_pt_paddr_intl(pte_paddr >> 12);
    }

    fn pt_address(&self) -> u32 {
        self.pt_paddr_intl() << 12
    }
}

pub enum PDEType {
    None,
    Table(PDETable),
    Direct(PDEDirect),
}

bitfield!(
#[derive(Copy, Clone)]
pub struct PageDirectoryEntry(pub u32) {
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
    pub fn to_pde(&self) -> PDEType {
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
pub struct CR3(pub u32) {
    register: u32 @ ..,

    pub writethrough: bool @ 3,
    pub disable_cache: bool @ 4,

    pd_paddr_raw: u32 @ 12..=31,
}
);

impl CR3 {
    pub unsafe fn current() -> Self {
        let cr3: u32;
        asm!("mov eax, cr3", out("eax") cr3);
        Self(cr3)
    }

    pub fn get_pd_paddr(&self) -> u32 {
        self.pd_paddr_raw() << 12
    }
}

pub struct Mapping {
    cr3: CR3,
    pd: &'static mut [PageDirectoryEntry; 1024],
    mapping_pt: &'static mut [PageTableEntry; 1024],

    // Keep at most 8 PTEs mapped in as active at once
    // This is a map indexed by the page number
    // (i.e. i for 0xC000_2000 + i * 0x1000)
    // and returns the physical PTE address mapped there,
    // or none.
    // This is only acting as a cache, no active_pte_indexes
    // should be in use at any particular time.
    active_pt_indexes: [Option<u32>; 8],
}

pub struct ActivePageTable<'tab> {
    // This field forces a mutable borrow of Mapping to be held
    // while mutating page tables mapped under it.
    // This prevents the table from being evicted under us.
    mapping: &'tab mut Mapping,

    idx: usize,
    pt: &'tab mut [PageTableEntry; 1024],
}

impl<'tab> ActivePageTable<'tab> {
    fn unmap(mut self) {
        self.mapping.mapping_pt[self.idx + 2].set_entry(0);
        self.mapping.active_pt_indexes[self.idx] = None;
        unsafe {
            invalidate_page(self.pt.as_ptr() as *const u8);
        }
    }

    fn paddr(&self) -> u32 {
        self.mapping.active_pt_indexes[self.idx].unwrap()
    }

    fn pte_mut(&mut self, vaddr: u32) -> &mut PageTableEntry {
        let idx = (vaddr >> 12) & 0x3ff;
        &mut self.pt[idx as usize]
    }

    unsafe fn map_vaddr(
        &mut self,
        vaddr: u32,
        paddr: u32,
        can_write: bool,
        writethrough: bool,
        cacheable: bool,
    ) {
        let pte = self.pte_mut(vaddr);
        pte.set_frame_addr(paddr);
        pte.set_allow_writes(can_write);
        pte.set_writethrough(writethrough);
        pte.set_disable_cache(!cacheable);
        pte.set_pat(false);
        pte.set_allow_usermode(false);
        pte.set_present(true);
    }

    unsafe fn unmap_vaddr(&mut self, vaddr: u32) {
        let pte = self.pte_mut(vaddr);
        pte.set_entry(0);
    }
}

/// Adjusts the bootstrap page tables to a more readily usable state
/// Also initializes physram allocator with currently used pages
unsafe fn bootstrap_setup(
    pd_paddr: u32,
    physram: &mut impl PhysramAllocator,
) -> (*mut PageDirectoryEntry, *mut PageTableEntry) {
    // Reserve some parts of the first 4MB in the allocator
    {
        // Zero page is used by the GPU
        physram.mark_allocated(0);
        physram.mark_allocated(pd_paddr);

        let kernel_data = crate::kernel_region();
        let kernel_paddr = kernel_data.as_ptr().mask(0x7fff_ffff) as u32;
        let kernel_pages = kernel_data.len().div_ceil(0x1000);
        for page in 0..kernel_pages {
            physram.mark_allocated(kernel_paddr + (page as u32) * 0x1000);
        }
    }

    // Setup PTE for 0xC000_0000 region
    let pt_paddr = physram.alloc().unwrap();

    // Restrict scope of the `pte` variable
    {
        // Bootstrap mappings have first 4MB at 0x8000_0000,
        // everything else is identity
        let pte = if pt_paddr > (1 << 22) {
            pt_paddr as *mut u32
        } else {
            (0x8000_0000 | pt_paddr) as *mut u32
        };
        let pte = &mut *(pte as *mut [PageTableEntry; 1024]);
        pte.fill(PageTableEntry(0));

        pte[0].set_frame_addr(pd_paddr);
        pte[0].set_allow_writes(true);
        pte[0].set_writethrough(true);
        pte[0].set_disable_cache(true);
        pte[0].set_present(true);

        pte[1].set_frame_addr(pt_paddr);
        pte[1].set_allow_writes(true);
        pte[1].set_writethrough(true);
        pte[1].set_disable_cache(true);
        pte[1].set_present(true);
    }

    let page_region_idx = pde_offset_from_addr(0xC000_0000);

    // Restrict `pde` scope
    {
        // Bootstrap page tables should be identity mapped with bit 31 set
        let pde = (0x8000_0000 | pd_paddr) as *mut PageDirectoryEntry;
        let pde = &mut *(pde as *mut [PageDirectoryEntry; 1024]);

        let mut page_region_pde = PDETable(0);

        page_region_pde.set_pt_address(pt_paddr);
        page_region_pde.set_allow_usermode(false);
        page_region_pde.set_allow_writes(true);
        page_region_pde.set_writethrough(true);
        page_region_pde.set_disable_cache(true);
        page_region_pde.set_present(true);
        pde[page_region_idx] = page_region_pde.into();
    }

    let pde = 0xC000_0000 as *mut PageDirectoryEntry;
    let pte = 0xC000_1000 as *mut PageTableEntry;
    (pde, pte)
}

impl Mapping {
    /// Creates a Mapping from the bootstrap page tables
    /// Safety:
    /// - cr3 must contain the bootstrap page tables
    pub unsafe fn from_bootstrap(physram: &mut impl PhysramAllocator) -> Self {
        let cr3 = CR3::current();
        let pd_paddr = cr3.get_pd_paddr();
        let (pd, pt) = bootstrap_setup(pd_paddr, physram);

        let pd = &mut *(pd as *mut [PageDirectoryEntry; 1024]);
        let mapping_pt = &mut *(pt as *mut [PageTableEntry; 1024]);

        let mut mapping = Self {
            cr3,
            pd,
            mapping_pt,
            active_pt_indexes: [None; 8],
        };

        // Remap kernel pages using PTE instead of large pages
        {
            let mut pt = mapping.new_pt_mapped(physram);
            pt.map_vaddr(0x8000_0000, 0x0, true, false, true);
            pt.map_vaddr(0x8000_0000 | pd_paddr, pd_paddr, true, true, false);

            let kernel_data = crate::kernel_region();
            let kernel_vaddr = kernel_data.as_ptr() as u32;
            let kernel_paddr = kernel_data.as_ptr().mask(0x7fff_ffff) as u32;
            let kernel_pages = kernel_data.len().div_ceil(0x1000);
            for pg in 0..kernel_pages {
                pt.map_vaddr(
                    kernel_vaddr + (pg as u32) * 0x1000,
                    kernel_paddr + (pg as u32) * 0x1000,
                    true,
                    false,
                    true,
                );
            }

            let mut kernel_pde = PDETable(0);
            kernel_pde.set_allow_writes(true);
            kernel_pde.set_writethrough(true);
            kernel_pde.set_disable_cache(true);
            kernel_pde.set_global(false);
            kernel_pde.set_allow_usermode(false);
            kernel_pde.set_pt_address(pt.paddr());
            kernel_pde.set_present(true);

            let pde = mapping.pde_walk_mut(0x8000_0000);
            *pde = kernel_pde.into();

            invalidate_all();
        }

        // FIXME: Account for 128MB
        let identity_mapped_pages = 64 / 4;
        for i in 1..identity_mapped_pages {
            mapping.pd[i].set_entry(0);
        }

        mapping
    }

    fn pde_walk(&self, vaddr: u32) -> &PageDirectoryEntry {
        let pde_index = vaddr >> 22;
        &self.pd[pde_index as usize]
    }

    fn pde_walk_mut(&mut self, vaddr: u32) -> &mut PageDirectoryEntry {
        let pde_index = vaddr >> 22;
        &mut self.pd[pde_index as usize]
    }

    fn new_pt_mapped<'a, 'b>(
        &'a mut self,
        pm: &'b mut impl PhysramAllocator,
    ) -> ActivePageTable<'a> {
        let frame = pm.alloc().unwrap();
        let active_pte = self.map_pt(frame);
        active_pte.pt.fill(PageTableEntry(0));
        active_pte
    }

    unsafe fn map_pt_vaddr(&mut self, frame: u32, idx: usize) -> ActivePageTable {
        let mapping_ent = &mut self.mapping_pt[idx + 2];

        mapping_ent.set_frame_addr(frame);
        mapping_ent.set_allow_writes(true);
        mapping_ent.set_writethrough(true);
        mapping_ent.set_disable_cache(true);
        mapping_ent.set_allow_usermode(false);
        mapping_ent.set_present(true);

        let pt_addr = 0xC000_2000 + idx * 0x1000;
        let pt = &mut *(pt_addr as *mut [PageTableEntry; 1024]);
        ActivePageTable {
            mapping: self,
            idx,
            pt,
        }
    }

    /// Maps a PTE, returning a handle to the PTE in virtual address space
    fn map_pt(&mut self, pt_frame: u32) -> ActivePageTable {
        let mut free_idx = None;
        for idx in 0..(self.active_pt_indexes.len()) {
            let active_pt = &mut self.active_pt_indexes[idx];

            if let Some(frame) = active_pt {
                if *frame == pt_frame {
                    return unsafe { self.map_pt_vaddr(pt_frame, idx) };
                }
            } else {
                free_idx = Some(idx);
            }
        }

        if let Some(idx) = free_idx {
            self.active_pt_indexes[idx] = Some(pt_frame);
            unsafe { self.map_pt_vaddr(pt_frame, idx) }
        } else {
            // Evict a random entry based on the allocated frame
            let idx = ((pt_frame >> 12) % 8) as usize;
            self.active_pt_indexes[idx] = Some(pt_frame);
            unsafe { self.map_pt_vaddr(pt_frame, idx) }
        }
    }
}
