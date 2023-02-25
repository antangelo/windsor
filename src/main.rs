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

const FB_SIZE: u32 = 0x40_0000;
const FB_START: u32 = 0xf000_0000 | (64 * 1024 * 1024 - FB_SIZE);

struct VGAPrinter {
    cursor_x: u32,
    cursor_y: u32,
}

impl VGAPrinter {
}

fn clear_screen(vm: &encoder::VideoModeInfo) {
    let mut addr = FB_START;

    for _ in 0..vm.height {
        for n in 0..vm.width {
            unsafe { *((addr + 4 * n) as *mut u32) = 0xff7aa0ff; }
        }

        addr += 4 * vm.width;
    }
}

#[no_mangle]
pub extern "C" fn kenter() -> ! {
    pci::initialize_devices();
    pci::initialize_agp();
    let gpu = nv2a::get_device();

    let video_mode = gpu.init(FB_START);

    clear_screen(&video_mode);

    loop {
        if gpu.pmc.intr.read() != 0 {
            unsafe { gpu.pcrtc.intr.write(0x1) };
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
