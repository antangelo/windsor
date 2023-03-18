use std::path::Path;

use colored::Colorize;

mod cargo;
mod binary;

const BOOT_WORKSPACE_NAME: &str = "windsor-boot";
const KRNL_WORKSPACE_NAME: &str = "windsor-kernel";
const TARGET: &str = "i686-unknown-none";

fn build_boot() -> Result<(), String> {
    let boot_args = vec!["--release"];
    let boot_path = std::path::Path::new(BOOT_WORKSPACE_NAME);

    println!("{} {}", "Building crate".green().bold(), BOOT_WORKSPACE_NAME);
    println!("{}: {}", "Created build args".green().bold(), boot_args.join(" "));

    cargo::build(boot_path, &boot_args).map_err(|e| e.to_string())?;

    println!("{}", "Creating output binary".green().bold());

    let output_binary = cargo::target_output_file(&boot_args, TARGET, BOOT_WORKSPACE_NAME);
    let output_path = Path::new(Path::new("windsor.bin"));
    binary::objcopy_bin(&output_binary, &output_path).map_err(|e| e.to_string())?;
    binary::pad_binary(&output_path, 262144)
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn build_kernel() -> Result<(), String> {
    let krnl_path = std::path::Path::new(KRNL_WORKSPACE_NAME);
    let krnl_args: Vec<&str> = vec![];

    println!("{} {}", "Building crate".green().bold(), KRNL_WORKSPACE_NAME);
    println!("{}: {}", "Created build args".green().bold(), krnl_args.join(" "));

    //cargo::build(krnl_path, &krnl_args).map_err(|e| e.to_string())?;

    Ok(())
}

fn main() -> Result<(), String> {
    let args = std::env::args();

    build_kernel()?;
    build_boot()?;

    Ok(())
}
