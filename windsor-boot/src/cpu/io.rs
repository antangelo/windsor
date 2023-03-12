use core::arch::asm;

pub unsafe fn write_u32(port: u16, val: u32) {
    asm!("out dx, eax", in("eax") val, in("dx") port);
}

pub unsafe fn read_u32(port: u16) -> u32 {
    let out: u32;
    asm!("in eax, dx", out("eax") out, in("dx") port);
    out
}

pub unsafe fn read_u8(port: u16) -> u8 {
    let out: u8;
    asm!("in al, dx", out("al") out, in("dx") port);
    out
}

pub unsafe fn write_u8(port: u16, val: u8) {
    asm!("out dx, al", in("al") val, in("dx") port);
}
