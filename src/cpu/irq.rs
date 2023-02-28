use super::idt;

extern "C" {
    pub fn irq_entry_0();
    pub fn irq_entry_1();
    pub fn irq_entry_2();
    pub fn irq_entry_3();
    pub fn irq_entry_4();
    pub fn irq_entry_5();
    pub fn irq_entry_6();
    pub fn irq_entry_7();
    pub fn irq_entry_8();
}

pub static mut IDT: [u64; 256] = [0; 256];

pub static mut IDTR: idt::Descriptor = idt::Descriptor {
    size: 256 * 8,
    offset: 0,
    pad: 0,
};

pub fn setup_irq() {
    unsafe {
        for i in 0..=255 {
            IDT[i] = idt::Entry::new(irq_entry_0 as u32, idt::GateType::Interrupt, 0x8).entry();
        }

        IDTR.offset = IDT.as_ptr() as u32;
    }
}

#[no_mangle]
pub unsafe extern "C" fn irq_0() {
    panic!("IRQ 0");
}

#[no_mangle]
pub unsafe extern "C" fn irq_1() {
    panic!("IRQ 1");
}

#[no_mangle]
pub unsafe extern "C" fn irq_2() {
    panic!("IRQ 2");
}

#[no_mangle]
pub unsafe extern "C" fn irq_3() {
    panic!("IRQ 3");
}

#[no_mangle]
pub unsafe extern "C" fn irq_4() {
    panic!("IRQ 4");
}

#[no_mangle]
pub unsafe extern "C" fn irq_5() {
    panic!("IRQ 5");
}

#[no_mangle]
pub unsafe extern "C" fn irq_6() {
    panic!("IRQ 6");
}

#[no_mangle]
pub unsafe extern "C" fn irq_7() {
    panic!("IRQ 7");
}

#[no_mangle]
pub unsafe extern "C" fn irq_8() {
    panic!("Double fault");
}
