pub fn init() {
    unsafe {
        super::io::write_u8(0x20, 0x15);
        super::io::write_u8(0x21, 0x20);
        super::io::write_u8(0x21, 0x04);
        super::io::write_u8(0x21, 0x01);
        super::io::write_u8(0x21, 0x00);
    }
}

pub fn reset() {
    unsafe {
        super::io::write_u8(0x20, 0x20);
    }
}
