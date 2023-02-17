# Windsor

## Building

Windsor binaries require post-processing to be usable. Rust's build post-processing leaves quite a bit to be desired,
so there are several methods that work.

### With cargo-binutils

```sh
$ cargo install cargo-binutils
$ rustup component add llvm-tools-preview
$ make
```

### With `llvm-objcopy`

Post-processing can also be applied manually with llvm-objcopy.

Due to an LLVM bug, the size of the output ROM is off by one byte, so it needs to be extended manually.

```sh
$ cargo build --release
$ llvm-objcopy -O binary ./target/i686-unknown-none/release/windsor windsor.bin
$ truncate -s +1 windsor.bin
```
