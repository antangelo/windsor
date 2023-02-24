use autopad::autopad;
use volatile_register::RW;

autopad!(
#[repr(C)]
pub struct PRMVIO {
    0x3c4 => pub seq_idx: RW<u8>,
    pub seq_data: RW<u8>,

    0x3ce => pub graph_idx: RW<u8>,
    pub graph_data: RW<u8>,
}
);

impl PRMVIO {
    pub unsafe fn seq(&mut self, idx: u8, data: u8) {
        self.seq_idx.write(idx);
        self.seq_data.write(data);
    }

    pub unsafe fn graph(&mut self, idx: u8, data: u8) {
        self.graph_idx.write(idx);
        self.graph_data.write(data);
    }

    pub fn init(&mut self) {
        unsafe {
            self.seq(0, 0x03);
            self.seq(1, 0x21);
            self.seq(2, 0x0f);
            self.seq(3, 0x00);
            self.seq(4, 0x06);
            self.graph(0, 0);
            self.graph(1, 0);
            self.graph(2, 0);
            self.graph(3, 0);
            self.graph(4, 0);
            self.graph(5, 0x40);
            self.graph(6, 0x05);
            self.graph(7, 0x0f);
            self.graph(8, 0xff);
        }
    }
}
