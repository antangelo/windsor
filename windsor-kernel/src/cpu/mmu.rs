use core::arch::asm;

pub fn wbinvd() {
    unsafe {
        asm!("wbinvd");
    }
}

/// Enables virtual memory
/// Requires that cr3 is set to a valid page table
pub unsafe fn enable() {
    asm!(
        // Enable PG and WP
        "mov eax, cr0",
        "or eax, 0x80010000",
        "mov cr0, eax",

        // Enable 4MB pages
        "mov eax, cr4",
        "or eax, 0x10",
        "mov cr4, eax",
        out("eax") _
        );
}
