#![feature(proc_macro_tracked_env)]
#![feature(track_path)]

extern crate proc_macro;
use build_tool_lib::{binary, cargo, config};
use md5::{Digest, Md5};
use object::{Object, ObjectSegment};
use proc_macro::TokenStream;
use std::path::PathBuf;

fn kernel_path() -> PathBuf {
    if let Ok(path) = proc_macro::tracked_env::var(config::KERNEL_ELF_PATH_ENV) {
        return PathBuf::from(path);
    }

    let kernel_output = cargo::target_output_file(&[], config::TARGET, config::KRNL_WORKSPACE_NAME);
    PathBuf::from("..").join(kernel_output).canonicalize().unwrap()
}

#[proc_macro]
pub fn include_kernel(_item: TokenStream) -> TokenStream {
    let kernel_path = kernel_path();
    proc_macro::tracked_path::path(kernel_path.to_string_lossy());
    let kernel_elf = std::fs::read(kernel_path).unwrap();

    let kernel_obj = object::read::File::parse(kernel_elf.as_slice()).unwrap();
    let (kernel_data, _) = binary::objcopy(kernel_elf.as_slice(), false).unwrap();
    let load_addr = kernel_obj
        .segments()
        .map(|s| s.address())
        .min()
        .expect("Failed to compute kernel load address");

    let zstd_data = build_tool_lib::binary::compress_data(&kernel_data).unwrap();

    let mut hasher = Md5::new();
    hasher.update(kernel_data.as_slice());
    let md5_sum = hasher.finalize();

    let kimg = format!(
        "crate::kimg::KernelImage {{ \
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
        kernel_obj.entry()
    );

    kimg.parse().unwrap()
}
