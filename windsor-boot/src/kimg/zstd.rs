use core::cell::UnsafeCell;
use core::{alloc::GlobalAlloc, ops};

use alloc_no_stdlib::{
    bzero, AllocatedStackMemory, Allocator, SliceWrapper, SliceWrapperMut, StackAllocator,
};

use zstd::FrameDecoder;
use zstd::frame_decoder::{FrameDecoderError, BlockDecodingStrategy};
use zstd::io::{Read, Error, ErrorKind};

pub struct ZstdDecompressor;

declare_stack_allocator_struct!(GlobalAllocatedFreelist, 16, global);
define_allocator_memory_pool!(16, u8, [0; 1024 * 1024], global, u8_pool);

struct MemAllocator(
    Option<UnsafeCell<StackAllocator<'static, u8, GlobalAllocatedFreelist<'static, u8>>>>,
);

unsafe impl Sync for MemAllocator {}

unsafe impl GlobalAlloc for MemAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let u8_alloc = unsafe { &mut *(self.0.as_ref().unwrap().get()) };
        assert_ne!(layout.size(), 0);

        let mut cell = u8_alloc.alloc_cell(layout.size());
        cell.as_mut_ptr()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let u8_alloc = unsafe { &mut *(self.0.as_ref().unwrap().get()) };

        let cell = core::slice::from_raw_parts_mut(ptr, layout.size());
        let mem = AllocatedStackMemory { mem: cell };
        u8_alloc.free_cell(mem);
    }
}

impl MemAllocator {
    fn init(&mut self) {
        let mut u8_alloc = GlobalAllocatedFreelist::<u8>::new_allocator(bzero);
        unsafe {
            bind_global_buffers_to_allocator!(u8_alloc, u8_pool, u8);
        }

        self.0 = Some(UnsafeCell::new(u8_alloc));
    }
}

#[global_allocator]
static mut ALLOCATOR: MemAllocator = MemAllocator(None);

struct ZstdDecoder<R: Read> {
    decoder: FrameDecoder,
    source: R,
}

impl<R: Read> ZstdDecoder<R> {
    fn new(mut source: R) -> Result<Self, FrameDecoderError> {
        let mut decoder = FrameDecoder::new();
        decoder.init(&mut source)?;
        Ok(Self { decoder, source })
    }
}

// This is copied from StreamingDecoder, but does not format an error
// message to save on space
impl<R: Read> Read for ZstdDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if self.decoder.is_finished() && self.decoder.can_collect() == 0 {
            return Ok(0);
        }

        while self.decoder.can_collect() < buf.len() && !self.decoder.is_finished() {
            //More bytes can be decoded
            let additional_bytes_needed = buf.len() - self.decoder.can_collect();
            match self.decoder.decode_blocks(
                &mut self.source,
                BlockDecodingStrategy::UptoBytes(additional_bytes_needed),
                ) {
                Ok(_) => { /*Nothing to do*/ }
                Err(_) => {
                    let err = Error::from(ErrorKind::Other);
                    return Err(err);
                }
            }
        }

        self.decoder.read(buf)
    }
}

impl ZstdDecompressor {
    fn decompress_status(img: &mut super::KernelImage) -> Option<()> {
        let mut load_mem = unsafe { img.load_mem() };
        let mut stream = ZstdDecoder::new(&mut img.data).ok()?;
        Read::read_exact(&mut stream, &mut load_mem).ok()?;

        Some(())
    }
}

impl super::ImageDecompressor for ZstdDecompressor {
    fn decompress_image(img: &mut super::KernelImage) {
        unsafe {
            ALLOCATOR.init();
        }

        if Self::decompress_status(img).is_none() {
            panic!("Decompression failed");
        }
    }
}
