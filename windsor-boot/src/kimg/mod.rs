pub struct KernelImage {
    pub data: &'static [u8],
    pub load_addr: *mut u8,
    pub load_size: usize,
    pub checksum: [u8; 16],
    pub entrypoint: usize,
}

unsafe impl Sync for KernelImage {}

impl KernelImage {
    pub unsafe fn load_mem(&self) -> &'static mut [u8] {
        core::slice::from_raw_parts_mut(self.load_addr, self.load_size)
    }
}

pub trait ImageDecompressor {
    fn decompress_image(img: &mut KernelImage);
}

#[cfg(all(feature = "brotli", feature = "zstd"))]
compile_error!("Only supply one decompressor feature");

#[cfg(feature = "brotli")]
pub mod brotli;

#[cfg(feature = "brotli")]
pub type Decompressor = brotli::BrotliDecompressor;

#[cfg(feature = "zstd")]
pub mod zstd;

#[cfg(feature = "zstd")]
pub type Decompressor = zstd::ZstdDecompressor;
