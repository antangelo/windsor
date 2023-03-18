use std::path::Path;
use object::{LittleEndian, read::elf::{FileHeader, ProgramHeader}};

pub fn objcopy_bin(exe: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let obj: Vec<u8> = std::fs::read(exe)?;
    let elf = object::elf::FileHeader32::<LittleEndian>::parse(obj.as_slice())?;
    let segments = elf.program_headers(LittleEndian, obj.as_slice())?;

    let load_addr = segments.iter()
        .filter(|s| s.p_type(LittleEndian) & object::elf::PT_LOAD != 0)
        .filter(|s| s.p_filesz(LittleEndian) > 0)
        .map(|s| s.p_paddr(LittleEndian))
        .min()
        .ok_or("Unable to compute load address")? as usize;

    let limit = segments.iter()
        .filter(|s| s.p_type(LittleEndian) & object::elf::PT_LOAD != 0)
        .filter(|s| s.p_filesz(LittleEndian) > 0)
        .map(|s| s.p_paddr(LittleEndian) + s.p_memsz(LittleEndian))
        .max()
        .ok_or("Unable to compute limit")? as usize;

    println!("Base address: {:08x}", load_addr);
    println!("Image size: {:08x}", limit - load_addr);

    let mut output_data = vec![0u8; limit - load_addr];

    println!("Writing sections");
    for segment in segments {
        let paddr = segment.p_paddr(LittleEndian) as usize;

        if segment.p_type(LittleEndian) & object::elf::PT_LOAD == 0 {
            println!("[{:08x}] Skipping unloadable segment", paddr);
            continue;
        }

        let fsize = segment.p_filesz(LittleEndian);
        if fsize == 0 {
            println!("[{:08x}] Skipping segment with no data", paddr);
            continue;
        }

        let file_addr = paddr - load_addr;
        let size = segment.p_memsz(LittleEndian) as usize;

        let data = segment.data(LittleEndian, obj.as_slice());
        if let Ok(data) = data {
            println!("[{:08x}] Writing segment of size {:08x}", paddr, data.len());
            assert_eq!(data.len(), size);
            output_data[file_addr..(file_addr + data.len())].copy_from_slice(data);
        }
    }

    std::fs::write(output, output_data)?;

    Ok(())
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
