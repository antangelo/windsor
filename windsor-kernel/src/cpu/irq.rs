use super::idt;

pub static mut IDT: [u64; 256] = [0; 256];
pub static mut IDTR: idt::Descriptor = idt::Descriptor::zero();

pub fn setup_irq() {
    unsafe {
        for i in 0..=255 {
            IDT[i] = idt::Entry::new(irq_unhandled as u32, idt::GateType::Interrupt, 0x8).raw_value();
        }

        IDT[8] = idt::Entry::new(irq_gpf as u32, idt::GateType::Interrupt, 0x8).raw_value();
        IDTR = idt::Descriptor::new(256 * 8, IDT.as_ptr() as u32);

        idt::lidt(&IDTR);
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackFrameValue {
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
}

#[repr(C)]
pub struct StackFrame {
    value: StackFrameValue,
}

impl StackFrame {
    fn as_mut(&mut self) -> &mut StackFrameValue {
        &mut self.value
    }

    fn as_ref(&self) -> &StackFrameValue {
        &self.value
    }
}

pub extern "x86-interrupt" fn irq_unhandled(_sf: StackFrame) {
    //panic!("Unhandled interrupt");
}

pub extern "x86-interrupt" fn irq_gpf(_sf: StackFrame, _code: u32) {
    panic!("Unhandled GPF");
}
