#![no_std]
#![no_main]
#![feature(const_mut_refs)]

mod cpu;
mod kimg;
mod smbus;
mod xcode;
mod asm;

#[macro_use]
extern crate alloc_no_stdlib as alloc;

use core::panic::PanicInfo;
use kimg::ImageDecompressor;

#[cfg(feature = "checksum")]
use md5::{Digest, Md5};

static mut KIMAGE: kimg::KernelImage = build_macros::include_kernel!();

#[no_mangle]
pub extern "C" fn kenter() -> ! {
    cpu::gdt::load_gdt();
    cpu::mmu::initialize();

    let mut kimg = unsafe { &mut KIMAGE };
    kimg::Decompressor::decompress_image(&mut kimg);

    #[cfg(feature = "checksum")]
    {
        let mut hasher = Md5::new();
        hasher.update(unsafe { kimg.load_mem() });
        let md5_sum = hasher.finalize();

        if md5_sum != kimg.checksum.into() {
            panic!("MD5 Mismatch");
        }
    }

    unsafe {
        core::arch::asm!("jmp eax", in("eax") kimg.entrypoint);
        core::hint::unreachable_unchecked();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    smbus::reboot();
}
