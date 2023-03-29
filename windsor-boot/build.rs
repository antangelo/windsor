fn main() {
    println!("cargo:rerun-if-changed=rom.ld");
    println!("cargo:rustc-link-arg=--script=rom.ld");
}
