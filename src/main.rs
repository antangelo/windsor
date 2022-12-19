#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
fn kenter() -> ! {
    let mut i = 0;
    loop {
        i = foo(i);
    }
}

pub fn foo(i: i32) -> i32 {
    return i + 1;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
