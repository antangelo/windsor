#![no_std]
#![no_main]
#![feature(pointer_byte_offsets)]

mod cpu;
mod encoder;
mod i2c;
mod nv2a;
mod pci;
mod smbus;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn kenter() -> ! {
    nv2a::set_pcrtc_start_addr(0xf0000000 | (64 * 0x10_0000 - 0x40_0000));
    pci::initialize_agp();
    nv2a::init_gpu();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
