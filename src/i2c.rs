use super::smbus;

pub unsafe fn tx_register(addr: u8, reg: u8, data: u16) -> Result<(), ()> {
    smbus::write(addr, reg, smbus::SMBusSize::Word, data as u32)
}

pub unsafe fn tx_read(addr: u8, data: u8) -> Result<u32, ()> {
    smbus::read(addr, data, smbus::SMBusSize::Byte)
}

pub unsafe fn tx_word(addr: u8, data: u16) -> Result<(), ()> {
    smbus::write(
        addr,
        ((data >> 8) & 0xff) as u8,
        smbus::SMBusSize::Byte,
        (data & 0xff) as u32,
    )
}
