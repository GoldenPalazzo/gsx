pub struct MemoryBus {}

impl MemoryBus {
    pub fn read_byte(&self, addr: u32) -> u8 {
        0
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        u16::from_le_bytes([self.read_byte(addr), self.read_byte(addr.wrapping_add(1))])
    }

    pub fn read_word(&self, addr: u32) -> u32 {
        u32::from_le_bytes([
            self.read_byte(addr),
            self.read_byte(addr.wrapping_add(1)),
            self.read_byte(addr.wrapping_add(2)),
            self.read_byte(addr.wrapping_add(3)),
        ])
    }

    pub fn write_byte(&mut self, addr: u32, val: u8) {}
    pub fn write_halfword(&mut self, addr: u32, val: u16) {
        let bytes = val.to_le_bytes();
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr.wrapping_add(1), bytes[1]);
    }
    pub fn write_word(&mut self, addr: u32, val: u32) {
        let bytes = val.to_le_bytes();
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr.wrapping_add(1), bytes[1]);
        self.write_byte(addr.wrapping_add(2), bytes[2]);
        self.write_byte(addr.wrapping_add(3), bytes[3]);
    }
}
