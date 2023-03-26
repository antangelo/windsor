use core::arch::asm;

/// Creates a simple PDE with 4MB mappings
/// and fully open access
pub const fn new_pde(paddr: u32) -> u32 {
    let paddr = paddr & 0xffc0_0000;

    paddr |
        (1 << 7) | // PS
        (1 << 4) | // PCD
        0xb // PWT, W, P
}

pub unsafe fn set_cr3(pde_addr: *const u32) {
    let cr3 = ((pde_addr as u32) & 0xffff_f000) | (1 << 4) | (1 << 3); // PCD, PWT

    asm!("mov cr3, eax", in("eax") cr3);
}

pub unsafe fn enable() {
    asm!(
    // Enable 4MB pages
    "mov eax, cr4",
    "or eax, 0x10",
    "mov cr4, eax",

    // Enable PG and WP
    "mov eax, cr0",
    "or eax, 0x80010000",
    "mov cr0, eax",

    out("eax") _
    );
}

/// We want to set up a bootstrap virtual memory environment
/// so that we can load the kernel into its rightful address
/// without it having to reload itself
/// As such these mappings are not particularly efficient,
/// they just need to be small and easy to set up
pub fn initialize() {
    let pde = 0xf000 as *mut u32;
    let page_size = 1 << 22; // 4MB

    // FIXME: Account for 128MB?
    let pages_to_identity_map = 64 / 4; // 64 MB / 4 MB

    unsafe {
        // Identity map all but the first 4MB of RAM
        for i in 1..pages_to_identity_map {
            let paddr = (i * page_size) as u32;
            pde.add(i).write_volatile(new_pde(paddr));
        }

        // Map 4MB to 0x8001_0000
        let kernel_base_pde_idx = 0x8001_0000 / page_size;
        pde.add(kernel_base_pde_idx).write_volatile(new_pde(0));

        // Identity map MMIO devices above 0xFD00_0000
        let mut curr_paddr: u32 = 0xF000_0000;
        while curr_paddr >= 0xF000_0000 {
            let pde_idx = curr_paddr / (page_size as u32);
            pde.add(pde_idx as usize)
                .write_volatile(new_pde(curr_paddr));
            curr_paddr = curr_paddr.wrapping_add(page_size as u32);
        }

        set_cr3(pde);
        enable();
    }
}
