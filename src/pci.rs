use crate::cpu::io;

pub enum PCIBus {
    Bus0 = 0,
}

pub enum PCIDevice {
    Dev0 = 0,
    Dev1 = 1,
}

pub unsafe fn write_dword(bus: PCIBus, dev: PCIDevice, reg: u8, val: u32) {
    let mut base = 0x800_0000;

    base |= (((bus as u8) & 0xff) as u32) << 16;
    base |= (((dev as u8) & 0x1f) as u32) << 11;
    base |= (reg & 0xff) as u32;

    io::write_u32(0xcf8, base);
    io::write_u32(0xcfc, val);
}

pub unsafe fn read_dword(bus: PCIBus, dev: PCIDevice, reg: u8) -> u32 {
    let mut base = 0x800_0000;

    base |= (((bus as u8) & 0xff) as u32) << 16;
    base |= (((dev as u8) & 0x1f) as u32) << 11;
    base |= (reg & 0xff) as u32;

    io::write_u32(0xcf8, base);
    io::read_u32(0xcfc)
}

pub fn initialize_agp() {
    unsafe {
        let tmp = read_dword(PCIBus::Bus0, PCIDevice::Dev1, 0x54);
        write_dword(PCIBus::Bus0, PCIDevice::Dev1, 0x54, 0x8800_0000 | tmp);

        let tmp = read_dword(PCIBus::Bus0, PCIDevice::Dev0, 0x64);
        write_dword(PCIBus::Bus0, PCIDevice::Dev0, 0x64, 0x8800_0000 | tmp);

        let tmp = read_dword(PCIBus::Bus0, PCIDevice::Dev0, 0x6c);
        io::write_u32(0xcfc, tmp & 0xffff_fffe);
        io::write_u32(0xcfc, tmp);

        write_dword(PCIBus::Bus0, PCIDevice::Dev0, 0x80, 0x100);
    }
}
