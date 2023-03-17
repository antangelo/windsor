pub struct KernelImage {
    pub data: &'static [u8],
    pub load_mem: &'static mut [u8],
    pub checksum: [u8; 16],
    pub entrypoint: usize,
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
