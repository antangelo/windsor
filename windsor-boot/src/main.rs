#![no_std]
#![no_main]
#![feature(pointer_byte_offsets)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]

mod cpu;
mod i2c;
mod kimg;
mod pci;
mod smbus;

#[macro_use]
extern crate alloc_no_stdlib as alloc;

use core::panic::PanicInfo;
use hex_literal::hex;
use kimg::ImageDecompressor;
use md5::{Digest, Md5};

static mut KIMAGE: kimg::KernelImage = build_macros::include_kernel!();

#[no_mangle]
pub extern "C" fn kenter() -> ! {
    cpu::gdt::load_gdt();
    pci::initialize_devices();
    pci::initialize_agp();

    let mut kimg = unsafe { &mut KIMAGE };

    {
        kimg::Decompressor::decompress_image(&mut kimg);
    }

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
    }

    loop {}
}

#[panic_handler]
#[inline(never)]
fn panic(_info: &PanicInfo) -> ! {
    // FIXME: Reboot the system if we end up here
    loop {}
}
