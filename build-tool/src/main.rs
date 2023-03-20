#![no_std]
#![feature(restricted_std)]
extern crate std;

use build_tool_lib::{binary, cargo, config};
use colored::Colorize;
use std::path::Path;

use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;
use std::println;

fn build_boot(kernel_path: &String) -> Result<(), String> {
    let boot_args = vec!["--profile=opt-size"];
    let boot_path = std::path::Path::new(config::BOOT_WORKSPACE_NAME);

    let mut boot_envs: Vec<(String, String)> = std::env::vars().collect();
    let kernel_path = std::fs::canonicalize(kernel_path).map_err(|e| e.to_string())?;
    boot_envs.push((
        String::from(config::KERNEL_ELF_PATH_ENV),
        String::from(kernel_path.into_os_string().to_string_lossy()),
    ));

    println!(
        "{} {}",
        "Building crate".green().bold(),
        config::BOOT_WORKSPACE_NAME
    );
    println!(
        "{}: {}",
        "Created build args".green().bold(),
        boot_args.join(" ")
    );

    cargo::build(boot_path, &boot_args, &boot_envs).map_err(|e| e.to_string())?;

    println!(
        "{} {}",
        "Creating output binary".green().bold(),
        config::OUTPUT_BINARY
    );

    let output_binary =
        cargo::target_output_file(&boot_args, config::TARGET, config::BOOT_WORKSPACE_NAME);
    let output_path = Path::new(config::OUTPUT_BINARY);
    binary::objcopy_bin(&output_binary, &output_path).map_err(|e| e.to_string())?;
    binary::pad_binary(&output_path, 262144).map_err(|e| e.to_string())?;

    Ok(())
}

fn build_kernel() -> Result<String, String> {
    let krnl_path = std::path::Path::new(config::KRNL_WORKSPACE_NAME);
    let krnl_args: Vec<&str> = vec![];
    let krnl_envs: Vec<(String, String)> = std::env::vars().collect();

    println!(
        "{} {}",
        "Building crate".green().bold(),
        config::KRNL_WORKSPACE_NAME
    );
    println!(
        "{}: {}",
        "Created build args".green().bold(),
        krnl_args.join(" ")
    );

    cargo::build(krnl_path, &krnl_args, &krnl_envs).map_err(|e| e.to_string())?;

    cargo::target_output_file(&krnl_args, config::TARGET, config::KRNL_WORKSPACE_NAME)
        .into_os_string()
        .to_str()
        .map(|s| String::from(s))
        .ok_or(String::from("Failed to get kernel target output path"))
}

fn build() -> Result<(), String> {
    let kernel_elf_file = build_kernel()?;
    build_boot(&kernel_elf_file)?;
    Ok(())
}

fn clean() -> Result<(), String> {
    cargo::clean(Path::new(config::BOOT_WORKSPACE_NAME)).map_err(|e| e.to_string())?;
    cargo::clean(Path::new(config::KRNL_WORKSPACE_NAME)).map_err(|e| e.to_string())?;
    std::fs::remove_file(config::OUTPUT_BINARY).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() <= 1 {
        return build();
    }

    if args[1] == "clean" {
        return clean();
    }

    build()
}
