TARGET_IMG=windsor.bin

all: $(TARGET_IMG)

$(TARGET_IMG):
	cargo objcopy --release -- -O binary $@
	truncate -s +1 $@

clean:
	rm -f $(TARGET_IMG)

.PHONY: $(TARGET_IMG) clean
