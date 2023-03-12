use cc::Build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=winsor-kernel/linker.ld");
    println!("cargo:rustc-link-arg=--script=windsor-kernel/linker.ld");

    for ent in glob::glob("asm/**/*.s")? {
        let path = ent?;
        let fname = path
            .with_extension("")
            .to_str()
            .ok_or("Cannot convert path to string")?
            .replace(std::path::MAIN_SEPARATOR, "_");

        Build::new()
            .file(&path)
            .flag("-ffreestanding")
            .compile(&fname);
        println!(
            "cargo:rerun-if-changed={}",
            path.to_str().ok_or("Cannot convert path to string")?
        );
    }

    Ok(())
}
