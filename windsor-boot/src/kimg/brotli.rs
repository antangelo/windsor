use brotli::{BrotliDecompressStream, BrotliResult, BrotliState, HuffmanCode};
use brotli_decompressor as brotli;
use core::ops;

use alloc::{
    bzero, AllocatedStackMemory, Allocator, SliceWrapper, SliceWrapperMut, StackAllocator,
};

const fn const_hc() -> HuffmanCode {
    HuffmanCode { value: 0, bits: 0 }
}

declare_stack_allocator_struct!(GlobalAllocatedFreelist, 16, global);
define_allocator_memory_pool!(16, u8, [0; 1024 * 1024], global, u8_pool);
define_allocator_memory_pool!(16, u32, [0; 1024 * 1024], global, u32_pool);
define_allocator_memory_pool!(
    16,
    brotli_decompressor::HuffmanCode,
    [super::const_hc(); 1024 * 1024],
    global,
    hc_pool
);

pub struct BrotliDecompressor;

impl super::ImageDecompressor for BrotliDecompressor {
    fn decompress_image(img: &mut super::KernelImage) {
        let mut u8_alloc = GlobalAllocatedFreelist::<u8>::new_allocator(bzero);
        let mut u32_alloc = GlobalAllocatedFreelist::<u32>::new_allocator(bzero);
        let mut hc_alloc = GlobalAllocatedFreelist::<HuffmanCode>::new_allocator(bzero);

        unsafe {
            bind_global_buffers_to_allocator!(u8_alloc, u8_pool, u8);
            bind_global_buffers_to_allocator!(u32_alloc, u32_pool, u32);
            bind_global_buffers_to_allocator!(hc_alloc, hc_pool, HuffmanCode);
        }

        let mut brotli_state = BrotliState::new(u8_alloc, u32_alloc, hc_alloc);

        let mut output_len = img.load_mem.len();
        let mut input_len = img.data.len();
        let mut input_offset = 0;
        let mut output_offset = 0;
        let mut written = 0;

        let result = BrotliDecompressStream(
            &mut input_len,
            &mut input_offset,
            img.data,
            &mut output_len,
            &mut output_offset,
            img.load_mem,
            &mut written,
            &mut brotli_state,
        );

        match result {
            BrotliResult::ResultSuccess => {}
            _ => panic!("Decompression failed"),
        }
    }
}
