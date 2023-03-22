#![no_std]
#![no_main]
#![feature(pointer_byte_offsets)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]

mod cpu;
mod encoder;
mod font;
mod i2c;
mod nv2a;
mod pci;
mod print;
mod smbus;

use core::panic::PanicInfo;

const FB_SIZE: u32 = 0x40_0000;
const FB_START: u32 = 0xf000_0000 | (64 * 1024 * 1024 - FB_SIZE);

fn clear_screen(vm: &encoder::VideoModeInfo, argb: u32) {
    let addr = FB_START;

    for y in 0..vm.height {
        let addr = addr + 4 * vm.width * y;
        for n in 0..vm.width {
            let addr = addr + 4 * n;
            unsafe {
                *(addr as *mut u32) = argb;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    unsafe {
        cpu::gdt::lgdt(&mut cpu::gdt::GDTR, cpu::gdt::GDT);

        cpu::irq::setup_irq();
        cpu::idt::lidt(&cpu::irq::IDTR);
    }

    pci::initialize_devices();
    pci::initialize_agp();

    let encoder = encoder::Model::detect();
    let av_mode = encoder::AVMode::detect();
    let video_mode = av_mode.get_video_mode(&encoder).unwrap();
    //clear_screen(&video_mode, 0xff7aa0ff);
    clear_screen(&video_mode, 0xff00_0000);

    let gpu = nv2a::get_device();
    gpu.init(FB_START);

    let mut printer = print::VGAPrinter::new(FB_START as *mut u32, &video_mode);
    printer.print_string_bytes(print::COLOR_WHITE, "windsor ".as_bytes());
    printer.print_string_bytes(print::COLOR_WHITE, env!("CARGO_PKG_VERSION").as_bytes());

    //cpu::sti();

    loop {
        if gpu.pmc.intr.read() != 0 {
            unsafe { gpu.pcrtc.intr.write(0x1) };
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        core::ptr::write_bytes(FB_START as *mut u8, 0xff, FB_SIZE as usize);
    }

    let encoder = encoder::Model::detect();
    let av_mode = encoder::AVMode::detect();
    let video_mode = av_mode.get_video_mode(&encoder).unwrap();

    let mut printer = print::VGAPrinter::new(FB_START as *mut u32, &video_mode);
    printer.print_string_bytes(print::COLOR_BLACK, b"Kernel panic!\n\n");

    if let Some(args) = info.message() {
        if let Some(msg) = args.as_str() {
            printer.print_string_bytes(print::COLOR_BLACK, msg.as_bytes());
        }
    }

    loop {}
}
