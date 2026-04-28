mod instrs;
mod regs;

use instrs::Instruction;

pub struct Cpu {
    gprs: [u32; 32],
    pc: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            gprs: [0u32; 32],
            pc: 0,
        }
    }

    pub fn step(&self, mem: &[u8]) {
        let opcode = u32::from_le_bytes([
            mem[self.pc as usize],
            mem[self.pc.wrapping_add(1) as usize],
            mem[self.pc.wrapping_add(2) as usize],
            mem[self.pc.wrapping_add(3) as usize],
        ]);
        let disasm = Instruction::decode(opcode);
        println!("{:08X} {} ({:?})", self.pc, opcode, disasm);
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
