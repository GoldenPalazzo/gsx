pub trait Coprocessor {
    // fn mfc(&self, rd: u8) -> u32;
    // fn cfc(&self, rd: u8) -> u32;
    // fn mtc(&mut self, rt_val: u32, rd: u8);
    // fn ctc(&mut self, rt_val: u32, rd: u8);
    fn read_reg(&self, reg: u8) -> u32;
    fn write_reg(&mut self, reg: u8, val: u32);
    fn exec(&mut self, cmd: u32);
}

pub struct Cop0 {
    regs: [u32; 64],
}

impl Cop0 {
    pub fn new() -> Self {
        Self { regs: [0u32; 64] }
    }
}

impl Coprocessor for Cop0 {
    fn read_reg(&self, reg: u8) -> u32 {
        match reg {
            0..=63 => self.regs[reg as usize],
            _ => u32::MAX,
        }
    }

    fn write_reg(&mut self, reg: u8, val: u32) {
        if let 0..=63 = reg {
            self.regs[reg as usize] = val
        }
    }
    fn exec(&mut self, cmd: u32) {}
}

pub struct Gte {
    regs: [u32; 64],
}

impl Gte {
    pub fn new() -> Self {
        Self { regs: [0u32; 64] }
    }
}

impl Coprocessor for Gte {
    fn read_reg(&self, reg: u8) -> u32 {
        match reg {
            0..=63 => self.regs[reg as usize],
            _ => u32::MAX,
        }
    }

    fn write_reg(&mut self, reg: u8, val: u32) {
        if let 0..=63 = reg {
            self.regs[reg as usize] = val
        }
    }
    fn exec(&mut self, cmd: u32) {}
}
