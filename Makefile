all:
	cargo build --release
	llvm-objcopy -O binary --strip-all target/i686-unknown-none/release/windsor windsor.bin
	truncate -s +1 windsor.bin
