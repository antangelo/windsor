use crate::cpu::io;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, FromPrimitive)]
enum XcodeOp {
    Peek = 0x02,
    Poke = 0x03,
    PokePci = 0x04,
    PeekPci = 0x05,
    AndOr = 0x06,
    Prefix = 0x07,
    BranchNE = 0x08,
    Jump = 0x09,
    OutB = 0x11,
    InB = 0x12,
    End = 0xee,
}

#[repr(C)]
#[repr(packed)]
struct Xcode {
    op: u8,
    arg1: i32,
    arg2: i32,
}

#[allow(dead_code)]
const STATIC_ASSERT_XCODE_SIZE: usize = (core::mem::size_of::<Xcode>() == 9) as usize - 1;

#[no_mangle]
#[link_section = ".hi_rom"]
pub unsafe extern "C" fn run_xcodes() {
    // Safety: low_rom code is placed at 0x1000,
    // thus xcodes can not be longer than that.
    // This makes this reference unique.
    let xcodes = &*(0xfffc_0080 as *const [Xcode; (0x1000 - 0x80) / core::mem::size_of::<Xcode>()]);
    let mut acc: i32 = 0;

    let mut idx = 0;
    loop {
        use XcodeOp::*;

        let (op, arg1, arg2) = {
            let xcode = &xcodes[idx as usize];
            idx += 1;

            let mut arg1 = xcode.arg1;
            let mut arg2 = xcode.arg2;

            let op = if xcode.op == Prefix as u8 {
                arg1 = arg2;
                arg2 = acc;
                xcode.arg1 as u8
            } else {
                xcode.op
            };

            let op = XcodeOp::from_u8(op).unwrap();
            (op, arg1, arg2)
        };

        match op {
            Peek => acc = *(arg1 as *const i32),
            Poke => *(arg1 as *mut i32) = arg2,
            PokePci => {
                io::write_u32(0xcf8, arg1 as u32);
                io::write_u32(0xcfc, arg2 as u32);
            }
            PeekPci => {
                io::write_u32(0xcf8, arg1 as u32);
                acc = io::read_u32(0xcfc) as i32;
            }
            AndOr => acc = (acc & arg1) | arg2,
            BranchNE => {
                if acc != arg1 {
                    idx += arg2 / 9;
                }
            }
            Jump => {
                idx += arg2 / 9;
            }
            OutB => io::write_u8(arg1 as u16, arg2 as u8),
            InB => acc = io::read_u8(arg1 as u16) as u32 as i32,
            End => break,
            Prefix => unreachable!(),
        }
    }
}
