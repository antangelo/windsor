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
mod bump;

#[macro_use]
extern crate alloc_no_stdlib as alloc;
use core::ops;

use core::panic::PanicInfo;
use hex_literal::hex;
use md5::{Md5, Digest};
use brotli_decompressor as brotli;
use brotli::{BrotliState, BrotliDecompressStream, BrotliResult, HuffmanCode};

use alloc::{Allocator, AllocatedStackMemory, StackAllocator, SliceWrapperMut, SliceWrapper, bzero};

const IMAGE_DATA: &'static [u8] = include_bytes!("../../windsor-kernel/windsor.bin.br");
const IMAGE_LOAD_ADDRESS: usize = 0x10_0000;
const IMAGE_LOAD_SIZE: usize = include_bytes!("../../windsor-kernel/windsor.bin").len();
const IMAGE_MD5: [u8; 16] = hex!("cfcd2fc9725b34ad34113344b1ce14fe");
const IMAGE_ENTRYPOINT: usize = 0x1037e0;

const fn const_hc() -> HuffmanCode {
    HuffmanCode {
        value: 0,
        bits: 0,
    }
}

declare_stack_allocator_struct!(GlobalAllocatedFreelist, 16, global);
define_allocator_memory_pool!(16, u8, [0; 1024 * 1024], global, u8_pool);
define_allocator_memory_pool!(16, u32, [0; 1024 * 1024], global, u32_pool);
define_allocator_memory_pool!(16, brotli_decompressor::HuffmanCode, [super::const_hc(); 1024 * 1024], global, hc_pool);

#[no_mangle]
pub extern "C" fn kenter() -> ! {
    cpu::gdt::load_gdt();
    pci::initialize_devices();
    pci::initialize_agp();

    let mut u8_alloc = GlobalAllocatedFreelist::<u8>::new_allocator(bzero);
    let mut u32_alloc = GlobalAllocatedFreelist::<u32>::new_allocator(bzero);
    let mut hc_alloc = GlobalAllocatedFreelist::<HuffmanCode>::new_allocator(bzero);

    unsafe {
        bind_global_buffers_to_allocator!(u8_alloc, u8_pool, u8);
        bind_global_buffers_to_allocator!(u32_alloc, u32_pool, u32);
        bind_global_buffers_to_allocator!(hc_alloc, hc_pool, HuffmanCode);
    }

    let mut brotli_state = BrotliState::new(u8_alloc, u32_alloc, hc_alloc);

    let output_buf = unsafe { &mut *(IMAGE_LOAD_ADDRESS as *mut [u8; IMAGE_LOAD_SIZE]) };
    let mut output_len = output_buf.len();
    let mut input_len = IMAGE_DATA.len();
    let mut input_offset = 0;
    let mut output_offset = 0;
    let mut written = 0;

    let result = BrotliDecompressStream(
        &mut input_len,
        &mut input_offset,
        IMAGE_DATA,
        &mut output_len,
        &mut output_offset,
        output_buf,
        &mut written,
        &mut brotli_state
        );

    match result {
        BrotliResult::ResultSuccess => {},
        _ => panic!("Decompression failed"),
    }

    let mut hasher = Md5::new();
    hasher.update(output_buf);
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
fn panic(_info: &PanicInfo) -> ! {
    // FIXME: Reboot the system if we end up here
    loop {}
}
