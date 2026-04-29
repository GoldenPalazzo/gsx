use crate::cpu::Exception;

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
    const SSR: usize = 12;
    const CAUSE: usize = 13;
    const EPC: usize = 14;

    pub fn new() -> Self {
        Self { regs: [0u32; 64] }
    }

    pub fn handle_exception(&mut self, ex: Exception, pc: u32) -> u32 {
        self.regs[Self::EPC] = pc;
        self.regs[Self::CAUSE] |= (ex as u32) << 2;
        self.push_iec_kuc(0b00);
        self.get_exception_handlers()
    }

    #[inline(always)]
    fn push_iec_kuc(&mut self, kuc_iec: u32) {
        let new_enable = ((self.regs[Self::SSR] & 0xf) << 2) | (kuc_iec & 0x3);
        self.regs[Self::SSR] = (self.regs[Self::SSR] & 0xffff_ff00) | new_enable;
    }

    #[inline(always)]
    fn pop_iec_kuc(&mut self) {
        let new_enable = (self.regs[Self::SSR] & 0x3c) >> 2;
        self.regs[Self::SSR] = (self.regs[Self::SSR] & 0xffff_fff0) | new_enable;
    }

    #[inline(always)]
    fn get_exception_handlers(&self) -> u32 {
        match (self.regs[Self::SSR] & (1 << 22)) != 0 {
            true => 0xbfc00180,
            false => 0x80000080,
        }
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
                self.pop_iec_kuc();
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
