/* SPDX-License-Identifier: GPL-2.0-only WITH Linux-syscall-note */

#![no_std]
#![no_main]

use core::panic::PanicInfo;

const ASTRING: &str = "Hello World";

#[no_mangle]
fn kenter() -> ! {
    let mut i = 0;
    loop {
        i = foo(i);
        bar(ASTRING);
    }
}

pub fn foo(i: i32) -> i32 {
    return i + 1;
}

pub fn bar(s: &str) {
    panic!("{}", s);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
