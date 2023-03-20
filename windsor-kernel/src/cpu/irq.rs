use super::idt;

extern "C" {
    pub fn irq_entry_0();
    pub fn irq_entry_8();
}

pub static mut IDT: [u64; 256] = [0; 256];

pub static mut IDTR: idt::Descriptor = idt::Descriptor::zero();

pub fn setup_irq() {
    unsafe {
        for i in 0..=255 {
            IDT[i] = idt::Entry::new(irq_entry_0 as u32, idt::GateType::Interrupt, 0x8).entry();
        }

        IDT[8] = idt::Entry::new(irq_entry_8 as u32, idt::GateType::Interrupt, 0x8).entry();
        IDTR = idt::Descriptor::new(256 * 8, IDT.as_ptr() as u32);
    }
}

#[no_mangle]
pub unsafe extern "C" fn irq_0() {
    panic!("Unhandled exception");
}

#[no_mangle]
pub unsafe extern "C" fn irq_8() {
    panic!("Unhandled double fault");
}
