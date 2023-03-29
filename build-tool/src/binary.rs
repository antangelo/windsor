use object::{
    read::elf::{FileHeader, ProgramHeader},
    LittleEndian,
};
use std::{boxed::Box, io::Write, path::Path, println, vec, vec::Vec};

pub fn objcopy(data: &[u8], verbose: bool) -> Result<(Vec<u8>, u32), Box<dyn std::error::Error>> {
    let elf = object::elf::FileHeader32::<LittleEndian>::parse(data)?;
    let segments = elf.program_headers(LittleEndian, data)?;

    let load_addr = segments
        .iter()
        .filter(|s| s.p_type(LittleEndian) & object::elf::PT_LOAD != 0)
        .filter(|s| s.p_filesz(LittleEndian) > 0)
        .map(|s| s.p_paddr(LittleEndian))
        .min()
        .ok_or("Unable to compute load address")? as usize;

    let limit = segments
        .iter()
        .filter(|s| s.p_type(LittleEndian) & object::elf::PT_LOAD != 0)
        .filter(|s| s.p_filesz(LittleEndian) > 0)
        .map(|s| s.p_paddr(LittleEndian) + s.p_memsz(LittleEndian))
        .max()
        .ok_or("Unable to compute limit")? as usize;

    if verbose {
        println!("Base address: {:08x}", load_addr);
        println!("Image size: {:08x}", limit - load_addr);
        println!("Writing sections");
    }

    let mut bin_size = 0;

    let mut output_data = vec![0u8; limit - load_addr];
    for segment in segments {
        let paddr = segment.p_paddr(LittleEndian) as usize;

        if segment.p_type(LittleEndian) & object::elf::PT_LOAD == 0 {
            if verbose {
                println!("[{:08x}] Skipping unloadable segment", paddr);
            }
            continue;
        }

        let fsize = segment.p_filesz(LittleEndian);
        if fsize == 0 {
            if verbose {
                println!("[{:08x}] Skipping segment with no data", paddr);
            }
            continue;
        }

        let file_addr = paddr - load_addr;
        let size = segment.p_memsz(LittleEndian) as usize;

        let data = segment.data(LittleEndian, data);
        if let Ok(data) = data {
            if verbose {
                println!("[{:08x}] Writing segment of size {:08x}", paddr, data.len());
            }

            // FIXME: Pad up? Or just leave BSS for something else to zero?
            //assert_eq!(data.len(), size);
            if verbose && data.len() != size {
                println!("Warning! Data length is not the same as memory length!");
            }
            output_data[file_addr..(file_addr + data.len())].copy_from_slice(data);
            bin_size += data.len() as u32;
        }
    }

    Ok((output_data, bin_size))
}

pub fn objcopy_bin(exe: &Path, output: &Path) -> Result<u32, Box<dyn std::error::Error>> {
    let obj: Vec<u8> = std::fs::read(exe)?;

    let (output_data, size) = objcopy(obj.as_slice(), true)?;
    std::fs::write(output, output_data)?;

    Ok(size)
}

pub fn pad_binary(binary: &Path, desired_size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::options()
        .read(true)
        .write(true)
        .open(binary)?;

    let desired_size: u64 = desired_size.try_into()?;
    file.set_len(desired_size)?;
    Ok(())
}

pub fn compress_data(input: &Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut zstd_data = vec![];
    let mut zstd_enc = zstd::stream::Encoder::new(&mut zstd_data, 3)?;
    zstd_enc.write_all(input.as_slice())?;
    zstd_enc.finish()?;

    Ok(zstd_data)
}
