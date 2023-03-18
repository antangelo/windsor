extern crate proc_macro;
use std::{io::Write, path::PathBuf};
use proc_macro::TokenStream;
use build_tool_lib::{config, cargo, binary};
use object::{Object, ObjectSegment};
use md5::{Md5, Digest};

fn kernel_path() -> PathBuf {
    if let Ok(path) = std::env::var(config::KERNEL_ELF_PATH_ENV) {
        return PathBuf::from(path);
    }

    cargo::target_output_file(&[], config::TARGET, config::KRNL_WORKSPACE_NAME)
}

#[proc_macro]
pub fn include_kernel(_item: TokenStream) -> TokenStream {
    let kernel_elf = std::fs::read(kernel_path()).unwrap();

    let kernel_obj = object::read::File::parse(kernel_elf.as_slice()).unwrap();
    let kernel_data = binary::objcopy(kernel_elf.as_slice()).unwrap();
    let load_addr = kernel_obj.segments()
        .map(|s| s.address())
        .min()
        .expect("Failed to compute kernel load address");

    let mut zstd_data = vec![];
    let mut zstd_enc = zstd::stream::Encoder::new(&mut zstd_data, 3).unwrap();
    zstd_enc.write_all(kernel_data.as_slice()).unwrap();
    zstd_enc.finish().unwrap();

    let mut hasher = Md5::new();
    hasher.update(kernel_data.as_slice());
    let md5_sum = hasher.finalize();

    let kimg = format!("crate::kimg::KernelImage {{ \
            data: &{:?}, \
            load_addr: {} as *mut u8, \
            load_size: {}, \
            checksum: {:?}, \
            entrypoint: {}, \
            }}",
            zstd_data.as_slice(),
            load_addr,
            kernel_data.len(),
            md5_sum,
            kernel_obj.entry());

    kimg.parse().unwrap()
}
