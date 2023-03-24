#![no_std]
#![feature(restricted_std)]
extern crate std;

use build_tool_lib::{binary, cargo, config};
use colored::Colorize;
use std::path::Path;

use std::string::{String, ToString};
use std::boxed::Box;
use std::vec;
use std::vec::Vec;
use std::println;

fn rom_utilization(boot_image_sz: u32, kernel_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let kernel_data = std::fs::read(kernel_path)?;
    let (kernel_data, size) = binary::objcopy(&kernel_data, false)?;
    let kernel_data = binary::compress_data(&kernel_data)?;
    let kernel_size = kernel_data.len() as u32;

    println!("Uncompressed kernel size: {}", size);
    println!("Compressed kernel size: {}", kernel_size);
    println!("Bootloader size: {}", boot_image_sz - kernel_size - 512);
    println!("ROM Size (raw): {}", boot_image_sz);

    let rom_size = (262144 - 512) as f32;
    let rom_used = (boot_image_sz - 512) as f32;
    println!("ROM Utilization: {}", rom_used / rom_size * 100.0);

    Ok(())
}

fn build_boot(bargs: &Vec<String>, kernel_path: &String) -> Result<u32, String> {
    let mut boot_args = vec!["--profile=opt-size"];
    boot_args.extend(bargs.iter().map(|s| s.as_str()).filter(|s| *s != "--release"));

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
    let len = binary::objcopy_bin(&output_binary, &output_path).map_err(|e| e.to_string())?;
    binary::pad_binary(&output_path, 262144).map_err(|e| e.to_string())?;

    Ok(len)
}

fn build_kernel(kargs: &Vec<String>) -> Result<String, String> {
    let krnl_path = std::path::Path::new(config::KRNL_WORKSPACE_NAME);
    let krnl_args: Vec<&str> = kargs.iter().map(|s| s.as_str()).collect();
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

fn build(kargs: &Vec<String>, bargs: &Vec<String>) -> Result<(), String> {
    let kernel_elf_file = build_kernel(&kargs)?;
    let image_size = build_boot(&bargs, &kernel_elf_file)?;
    rom_utilization(image_size, Path::new(&kernel_elf_file)).map_err(|e| e.to_string())?;
    Ok(())
}

fn clean() -> Result<(), String> {
    cargo::clean(Path::new(config::BOOT_WORKSPACE_NAME)).map_err(|e| e.to_string())?;
    cargo::clean(Path::new(config::KRNL_WORKSPACE_NAME)).map_err(|e| e.to_string())?;
    std::fs::remove_file(config::OUTPUT_BINARY).map_err(|e| e.to_string())?;
    Ok(())
}

fn parse_args(args: &Vec<String>) -> Result<(Vec<String>, Vec<String>), String> {
    let mut kernel_args: Vec<String> = vec![];
    let mut boot_args: Vec<String> = vec![];

    if args.len() >= 2 {
        let mut for_boot = false;

        for i in 1..args.len() {
            let arg = &args[i];
            if i == 1 && !arg.starts_with("-") {
                continue;
            }

            let into = if for_boot {
                for_boot = false;
                &mut boot_args
            } else {
                &mut kernel_args
            };

            match arg.as_str() {
                "-B" => for_boot = true,
                "--release" => into.push(arg.clone()),
                a => return Err(std::format!("Unknown argument {}", a)),
            }
        }
    }

    Ok((kernel_args, boot_args))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    let (kernel_args, boot_args) = parse_args(&args)?;

    if args.len() <= 1 {
        return build(&kernel_args, &boot_args);
    }

    if args[1] == "clean" {
        return clean();
    }

    build(&kernel_args, &boot_args)
}
