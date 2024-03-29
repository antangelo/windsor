#![no_std]
#![no_main]
#![feature(pointer_byte_offsets)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(ptr_mask)]
#![feature(int_roundings)]
#![feature(naked_functions)]

mod cpu;
mod encoder;
mod font;
mod i2c;
mod nv2a;
mod pci;
mod physram;
mod print;
mod smbus;

use core::panic::PanicInfo;

pub static mut PRINTER: Option<print::VGAPrinter> = None;

#[no_mangle]
#[naked]
pub unsafe extern "C" fn kenter() {
    core::arch::asm!(
        // Zero BSS
        "xor %eax, %eax",
        "mov $__start_bss_ram, %edi",
        "mov $__bss_size, %ecx",
        "shr $2, %ecx",
        "rep stosl",
        // Start kernel
        "mov $__kernel_stack, %esp",
        "mov %esp, %ebp",
        "jmp kmain",
        options(att_syntax, noreturn),
    );
}

extern "C" {
    static mut __start_code_ram: u32;
    static mut __kernel_stack: u32;
}

macro_rules! linker_var {
    ($id:ident) => {
        &$id as *const u32 as u32
    };
}

pub fn kernel_region() -> &'static [u8] {
    unsafe {
        let load_addr = linker_var!(__start_code_ram);
        let size = linker_var!(__kernel_stack) - load_addr;
        core::slice::from_raw_parts(load_addr as *const u8, size as usize)
    }
}

const FB_SIZE: u32 = 0x40_0000;
const FB_START: u32 = 0xf000_0000 | (64 * 1024 * 1024 - FB_SIZE);

fn clear_screen(vm: &encoder::VideoModeInfo, argb: u32) {
    let fb = unsafe {
        core::slice::from_raw_parts_mut(FB_START as *mut u32, (vm.height * vm.width) as usize)
    };
    fb.fill(argb);
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    unsafe {
        cpu::gdt::lgdt(&mut cpu::gdt::GDTR, cpu::gdt::GDT);
        cpu::irq::setup_irq();

        let mut pmm = physram::BitmapAlloc::new();
        let _mmu = cpu::mmu::Mapping::from_bootstrap(&mut pmm);
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

    cpu::pic::init();
    cpu::sti();

    let mut frame_count: u64 = 0;
    let colors = [0xff00_0000, 0xff7aa0ff];
    let text_colors = [print::COLOR_WHITE, print::COLOR_BLACK];
    let mut color_toggle = 0;

    loop {
        if gpu.pmc.intr.read() != 0 {
            frame_count += 1;
            unsafe { gpu.pcrtc.intr.write(0x1) };

            if frame_count % 60 == 0 {
                clear_screen(&video_mode, colors[color_toggle]);
                printer.reset();
                printer.print_string_bytes(text_colors[color_toggle], "windsor ".as_bytes());
                printer.print_string_bytes(text_colors[color_toggle], env!("CARGO_PKG_VERSION").as_bytes());

                color_toggle += 1;
                color_toggle %= 2;
            }
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
