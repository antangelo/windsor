pub mod gdt;
pub mod idt;
pub mod io;
pub mod irq;
pub mod mmu;
pub mod pic;

use core::arch::asm;

pub fn cli() {
    unsafe {
        asm!("cli");
    }
}

pub fn sti() {
    unsafe {
        asm!("sti");
    }
}
