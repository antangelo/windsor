use super::cpu::io;

const I2C_PORT: u16 = 0xc000;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum SMBusSize {
    Byte = 1,
    Word = 2,
    DWord = 4,
}

pub fn reboot() -> ! {
    unsafe {
        write(0x10, 0x2, SMBusSize::Byte, 0x80).unwrap();
    }
    loop {}
}

unsafe fn try_write(addr: u8, reg: u8, size: SMBusSize, val: u32) -> bool {
    io::write_u32(I2C_PORT + 4, (addr as u32) << 1);
    io::write_u32(I2C_PORT + 8, reg as u32);

    match size {
        SMBusSize::Byte => {
            io::write_u32(I2C_PORT + 6, val & 0xff);
        }
        SMBusSize::Word => {
            io::write_u32(I2C_PORT + 6, val & 0xffff);
        }
        SMBusSize::DWord => {
            io::write_u32(I2C_PORT + 9, (val >> 0) & 0xff);
            io::write_u32(I2C_PORT + 9, (val >> 8) & 0xff);
            io::write_u32(I2C_PORT + 9, (val >> 16) & 0xff);
            io::write_u32(I2C_PORT + 9, (val >> 24) & 0xff);
            io::write_u32(I2C_PORT + 6, 4);
        }
    }

    // FIXME: Better variable name
    let tmp = io::read_u32(I2C_PORT);
    io::write_u32(I2C_PORT, tmp);

    match size {
        SMBusSize::Byte => io::write_u32(I2C_PORT + 2, 0x1a),
        SMBusSize::Word => io::write_u32(I2C_PORT + 2, 0x1b),
        SMBusSize::DWord => io::write_u32(I2C_PORT + 2, 0x1d),
    }

    let mut result = io::read_u8(I2C_PORT);
    while result & 0x36 == 0 {
        result = io::read_u8(I2C_PORT);
        core::hint::spin_loop();
    }

    result & 0x10 != 0
}

pub unsafe fn write(addr: u8, reg: u8, size: SMBusSize, val: u32) -> Result<(), ()> {
    while io::read_u32(I2C_PORT) & 0x800 != 0 {
        core::hint::spin_loop();
    }

    // FIXME: How many tries does this take really?
    for _ in 0..50 {
        if try_write(addr, reg, size, val) {
            return Ok(());
        }
    }

    Err(())
}
