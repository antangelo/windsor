#![no_std]
#![no_main]
#![feature(pointer_byte_offsets)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(naked_functions)]

mod cpu;
mod i2c;
mod pci;
mod smbus;
mod kimg;

#[macro_use]
extern crate alloc_no_stdlib as alloc;

use core::panic::PanicInfo;
use hex_literal::hex;
use kimg::ImageDecompressor;
use md5::{Md5, Digest};

const IMAGE_DATA: &'static [u8] = include_bytes!("../../windsor-kernel/windsor-dbg.bin.zst");
const IMAGE_LOAD_ADDRESS: usize = 0x10_0000;
const IMAGE_LOAD_SIZE: usize = include_bytes!("../../windsor-kernel/windsor-dbg.bin").len();
const IMAGE_MD5: [u8; 16] = hex!("df2ac8ea035d20bc1154879aa329f474");
const IMAGE_ENTRYPOINT: usize = 0x105430;

#[no_mangle]
pub extern "C" fn kenter() -> ! {
    cpu::gdt::load_gdt();
    pci::initialize_devices();
    pci::initialize_agp();

    let mut kimg = kimg::KernelImage {
        data: IMAGE_DATA,
        load_mem: unsafe { core::slice::from_raw_parts_mut(IMAGE_LOAD_ADDRESS as *mut u8, IMAGE_LOAD_SIZE) },
        checksum: IMAGE_MD5,
        entrypoint: IMAGE_ENTRYPOINT,
    };

    kimg::Decompressor::decompress_image(&mut kimg);

    let mut hasher = Md5::new();
    hasher.update(kimg.load_mem);
    let md5_sum = hasher.finalize();

    if md5_sum != IMAGE_MD5.into() {
        panic!("MD5 Mismatch");
    }

    unsafe {
        core::arch::asm!("jmp eax", in("eax") IMAGE_ENTRYPOINT);
    }

    loop {}
}

#[panic_handler]
#[inline(never)]
fn panic(_info: &PanicInfo) -> ! {
    // FIXME: Reboot the system if we end up here
    loop {}
}
