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
    fn exec(&mut self, cmd: u32) {
        match cmd & 0x3f {
            0b010000 => {
                todo!("RFE")
            }
            _ => unreachable!("Invalid cmd {:08X}", cmd),
        }
    }
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
    fn exec(&mut self, cmd: u32) {
        // todo!("{:08X}", cmd)
        let real_cmd = cmd & 0x3f;
        let saturate = (cmd & (1 << 10)) != 0;
        let mvma_trans = (cmd >> 13) & 3;
        let mvma_mul_vec = (cmd >> 15) & 3;
        let mvma_mul_matr = (cmd >> 17) & 3;
        let shift_fac = (cmd & (1 << 19)) != 0;
        println!(
            "Silent stub COP2 {:025b} {} {} {} {} {} {}",
            cmd, real_cmd, saturate, mvma_trans, mvma_mul_vec, mvma_mul_matr, shift_fac
        );
        // silent stub
        // panic!(
        //     "{:025b} {} {} {} {} {} {} ",
        //     cmd, real_cmd, saturate, mvma_trans, mvma_mul_vec, mvma_mul_matr, shift_fac
        // );
    }
}
