mod coprocessor;
mod instrs;

use std::fmt;

use coprocessor::{Cop0, Coprocessor, Gte};

use super::memory::MemoryBus;
use instrs::{CopInstruction, Instruction};

#[derive(Debug)]
#[repr(u8)]
enum Exception {
    ExternalInterrupt = 0x00,
    TLBModification = 0x01,
    TLBLoad = 0x02,
    TLBStore = 0x03,
    AddressErrorDataLoad = 0x04,
    AddressErrorDataStore = 0x05,
    BusErrorInstructionFetch = 0x06,
    BusErrorData = 0x07,
    SystemCall = 0x08,
    Breakpoint = 0x09,
    ReservedInstruction = 0x0a,
    CoprocessorUnusable = 0x0b,
    ArithmeticOverflow = 0x0c,
}

const RETURN_ADDR: u8 = 31;

pub struct Cpu {
    gprs: [u32; 32],
    pc: u32,
    hi: u32,
    lo: u32,

    cop0: Cop0,
    gte: Gte,

    load_delay: Option<(u8, u32)>,
    jump_delay: Option<u32>,
    exception_pc: Option<u32>,
}

macro_rules! cop_dispatch {
    ($self:expr, $n:expr, $method:ident ( $($arg:expr),* )) => {
        match $n {
            0 => $self.cop0.$method($($arg),*),
            2 => $self.gte.$method($($arg),*),
            _ => unreachable!(),
        }
    };
}

impl Cpu {
    pub fn new(pc: u32, r28: u32, r29: u32, r30: u32) -> Self {
        let mut n = Self {
            gprs: [0u32; 32],
            pc,
            hi: 0,
            lo: 0,

            cop0: Cop0::new(),
            gte: Gte::new(),

            load_delay: None,
            jump_delay: None,
            exception_pc: None,
        };
        n.gprs[28] = r28;
        n.gprs[29] = r29;
        n.gprs[30] = r30;
        n
    }

    pub fn step(&mut self, mem: &mut MemoryBus) {
        assert!(self.pc.is_multiple_of(4), "PC is unaligned!!!");
        let pending_load = self.load_delay.take();
        let pending_jump = self.jump_delay.take();

        // let opcode = u32::from_le_bytes([
        //     mem[self.pc as usize],
        //     mem[self.pc.wrapping_add(1) as usize],
        //     mem[self.pc.wrapping_add(2) as usize],
        //     mem[self.pc.wrapping_add(3) as usize],
        // ]);
        let opcode = mem.read_word(self.pc);
        let disasm = Instruction::decode(opcode);
        print!("{}", self);
        println!("0x{:08X}: {:08X} -> {:?}\n", self.pc, opcode, disasm);
        self.execute(disasm, mem, pending_jump.is_some());

        if let Some(epc) = self.exception_pc.take() {
            self.pc = epc;
            return;
        }

        if let Some((reg, val)) = pending_load {
            self.write_reg(reg, val);
        }

        if let Some(pc) = pending_jump {
            self.pc = pc;
        } else {
            self.pc = self.pc.wrapping_add(4);
        }
    }

    fn read_reg(&self, idx: u8) -> u32 {
        self.gprs[idx as usize]
    }

    fn write_reg(&mut self, idx: u8, val: u32) {
        if idx != 0 {
            self.gprs[idx as usize] = val;
        }
    }

    fn execute(&mut self, opcode: Instruction, mem: &mut MemoryBus, load_delay: bool) {
        match opcode {
            Instruction::ADD { rs, rt, rd } => {
                let a = self.read_reg(rs) as i32;
                let b = self.read_reg(rt) as i32;
                match a.checked_add(b) {
                    Some(val) => self.write_reg(rd, val as u32),
                    None => self.trigger_exception(Exception::ArithmeticOverflow, load_delay),
                }
            }
            Instruction::ADDU { rs, rt, rd } => {
                self.write_reg(rd, self.read_reg(rs).wrapping_add(self.read_reg(rt)));
            }
            Instruction::SUB { rs, rt, rd } => {
                let a = self.read_reg(rs) as i32;
                let b = self.read_reg(rt) as i32;
                match a.checked_sub(b) {
                    Some(val) => self.write_reg(rd, val as u32),
                    None => self.trigger_exception(Exception::ArithmeticOverflow, load_delay),
                }
            }
            Instruction::SUBU { rs, rt, rd } => {
                self.write_reg(rd, self.read_reg(rs).wrapping_sub(self.read_reg(rt)));
            }
            Instruction::ADDI { rs, rt, imm } => {
                let a = self.read_reg(rs) as i32;
                let b = imm as i16 as i32;
                match a.checked_add(b) {
                    Some(val) => self.write_reg(rt, val as u32),
                    None => self.trigger_exception(Exception::ArithmeticOverflow, load_delay),
                }
            }
            Instruction::ADDIU { rs, rt, imm } => {
                let val = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                self.write_reg(rt, val);
            }
            Instruction::SLT { rs, rt, rd } => {
                let val = (self.read_reg(rs) as i32) < (self.read_reg(rt) as i32);
                self.write_reg(rd, val as u32);
            }
            Instruction::SLTU { rs, rt, rd } => {
                let val = self.read_reg(rs) < self.read_reg(rt);
                self.write_reg(rd, val as u32);
            }
            Instruction::SLTI { rs, rt, imm } => {
                let val = (self.read_reg(rs) as i32) < (imm as i16 as i32);
                self.write_reg(rt, val as u32);
            }
            Instruction::SLTIU { rs, rt, imm } => {
                let val = self.read_reg(rs) < (imm as i16 as i32 as u32);
                self.write_reg(rt, val as u32);
            }
            Instruction::AND { rs, rt, rd } => {
                let val = self.read_reg(rs) & self.read_reg(rt);
                self.write_reg(rd, val);
            }
            Instruction::OR { rs, rt, rd } => {
                let val = self.read_reg(rs) | self.read_reg(rt);
                self.write_reg(rd, val);
            }
            Instruction::XOR { rs, rt, rd } => {
                let val = self.read_reg(rs) ^ self.read_reg(rt);
                self.write_reg(rd, val);
            }
            Instruction::NOR { rs, rt, rd } => {
                let val = u32::MAX ^ (self.read_reg(rs) | self.read_reg(rt));
                self.write_reg(rd, val);
            }
            Instruction::ANDI { rs, rt, imm } => {
                let val = self.read_reg(rs) & imm as u32;
                self.write_reg(rt, val);
            }
            Instruction::ORI { rs, rt, imm } => {
                let val = self.read_reg(rs) | imm as u32;
                self.write_reg(rt, val);
            }
            Instruction::XORI { rs, rt, imm } => {
                let val = self.read_reg(rs) ^ imm as u32;
                self.write_reg(rt, val);
            }
            Instruction::SLLV { rs, rt, rd } => {
                self.write_reg(rd, self.read_reg(rt) << (self.read_reg(rs) & 0x1f));
            }
            Instruction::SRLV { rs, rt, rd } => {
                self.write_reg(rd, self.read_reg(rt) >> (self.read_reg(rs) & 0x1f));
            }
            Instruction::SRAV { rs, rt, rd } => {
                self.write_reg(
                    rd,
                    ((self.read_reg(rt) as i32) >> (self.read_reg(rs) & 0x1f)) as u32,
                );
            }
            Instruction::SLL { rt, rd, imm } => {
                self.write_reg(rd, self.read_reg(rt) << (imm & 0x1f));
            }
            Instruction::SRL { rt, rd, imm } => {
                self.write_reg(rd, self.read_reg(rt) >> (imm & 0x1f));
            }
            Instruction::SRA { rt, rd, imm } => {
                self.write_reg(rd, ((self.read_reg(rt) as i32) >> (imm & 0x1f)) as u32);
            }
            Instruction::LUI { rt, imm } => {
                self.write_reg(rt, (imm as u32) << 16);
            }
            Instruction::MULT { rs, rt } => {
                let val = self.read_reg(rs) as i32 as i64 * self.read_reg(rt) as i32 as i64;
                self.hi = (val >> 32) as u32;
                self.lo = val as u32;
            }
            Instruction::MULTU { rs, rt } => {
                let val = self.read_reg(rs) as u64 * self.read_reg(rt) as u64;
                self.hi = (val >> 32) as u32;
                self.lo = val as u32;
            }
            Instruction::DIV { rs, rt } => {
                let a = self.read_reg(rs) as i32;
                let b = self.read_reg(rt) as i32;
                if b == 0 {
                    self.hi = a as u32;
                    self.lo = if a < 0 { 1 } else { u32::MAX };
                } else {
                    let (quot, _) = a.overflowing_div(b);
                    let (rem, _) = a.overflowing_rem(b);
                    self.hi = rem as u32;
                    self.lo = quot as u32;
                }
            }
            Instruction::DIVU { rs, rt } => {
                let a = self.read_reg(rs);
                let b = self.read_reg(rt);
                if b == 0 {
                    self.hi = a;
                    self.lo = u32::MAX;
                } else {
                    self.hi = a % b;
                    self.lo = a / b;
                }
            }
            Instruction::MFHI { rd } => self.write_reg(rd, self.hi),
            Instruction::MFLO { rd } => self.write_reg(rd, self.lo),
            Instruction::MTHI { rs } => self.hi = self.read_reg(rs),
            Instruction::MTLO { rs } => self.lo = self.read_reg(rs),
            Instruction::LB { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                let val = mem.read_byte(addr);
                self.load_delay = Some((rt, val as i8 as i32 as u32));
            }
            Instruction::LBU { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                let val = mem.read_byte(addr);
                self.load_delay = Some((rt, val as u32));
            }
            Instruction::LH { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                if addr.is_multiple_of(2) {
                    let val = mem.read_halfword(addr);
                    self.load_delay = Some((rt, val as i16 as i32 as u32));
                } else {
                    self.trigger_exception(Exception::AddressErrorDataLoad, load_delay);
                }
            }
            Instruction::LHU { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                if addr.is_multiple_of(2) {
                    let val = mem.read_halfword(addr);
                    self.load_delay = Some((rt, val as u32));
                } else {
                    self.trigger_exception(Exception::AddressErrorDataLoad, load_delay);
                }
            }
            Instruction::LW { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                if addr.is_multiple_of(4) {
                    let val = mem.read_word(addr);
                    self.load_delay = Some((rt, val));
                } else {
                    self.trigger_exception(Exception::AddressErrorDataLoad, load_delay);
                }
            }
            Instruction::SB { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                mem.write_byte(addr, self.read_reg(rt) as u8);
            }
            Instruction::SH { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                if addr.is_multiple_of(2) {
                    mem.write_halfword(addr, self.read_reg(rt) as u16);
                } else {
                    self.trigger_exception(Exception::AddressErrorDataStore, load_delay);
                }
            }
            Instruction::SW { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                if addr.is_multiple_of(4) {
                    mem.write_word(addr, self.read_reg(rt));
                } else {
                    self.trigger_exception(Exception::AddressErrorDataStore, load_delay);
                }
            }
            Instruction::LWR { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                let aligned = addr & !3;
                let mem_word = mem.read_word(aligned);
                let cur = self.read_reg(rt);
                let val = match addr & 3 {
                    0 => mem_word,
                    1 => (cur & 0xff00_0000) | (mem_word >> 8),
                    2 => (cur & 0xffff_0000) | (mem_word >> 16),
                    3 => (cur & 0xffff_ff00) | (mem_word >> 24),
                    _ => unreachable!(),
                };
                self.load_delay = Some((rt, val))
            }
            Instruction::LWL { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                let aligned = addr & !3;
                let mem_word = mem.read_word(aligned);
                let cur = self.read_reg(rt);
                let val = match addr & 3 {
                    0 => (cur & 0x00ff_ffff) | (mem_word << 24),
                    1 => (cur & 0x0000_ffff) | (mem_word << 16),
                    2 => (cur & 0x0000_00ff) | (mem_word << 8),
                    3 => mem_word,
                    _ => unreachable!(),
                };
                self.load_delay = Some((rt, val));
            }
            Instruction::SWR { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                let aligned = addr & !3;
                let mem_word = mem.read_word(aligned);
                let reg = self.read_reg(rt);
                let val = match addr & 3 {
                    0 => reg,
                    1 => (mem_word & 0x0000_00ff) | (reg << 8),
                    2 => (mem_word & 0x0000_ffff) | (reg << 16),
                    3 => (mem_word & 0x00ff_ffff) | (reg << 24),
                    _ => unreachable!(),
                };
                mem.write_word(aligned, val);
            }
            Instruction::SWL { rs, rt, imm } => {
                let addr = self.read_reg(rs).wrapping_add(imm as i16 as i32 as u32);
                let aligned = addr & !3;
                let mem_word = mem.read_word(aligned);
                let reg = self.read_reg(rt);
                let val = match addr & 3 {
                    0 => (mem_word & 0xffff_ff00) | (reg >> 24),
                    1 => (mem_word & 0xffff_0000) | (reg >> 16),
                    2 => (mem_word & 0xff00_0000) | (reg >> 8),
                    3 => reg,
                    _ => unreachable!(),
                };
                mem.write_word(aligned, val);
            }
            Instruction::J { imm } => {
                self.jump_delay = Some((self.pc & 0xf000_0000) + (imm * 4));
            }
            Instruction::JAL { imm } => {
                self.jump_delay = Some((self.pc & 0xf000_0000) + (imm * 4));
                self.write_reg(RETURN_ADDR, self.pc.wrapping_add(8));
            }
            Instruction::JR { rs } => {
                self.jump_delay = Some(self.read_reg(rs));
            }
            Instruction::JALR { rs, rd } => {
                self.jump_delay = Some(self.read_reg(rs));
                self.write_reg(rd, self.pc.wrapping_add(8));
            }
            Instruction::BEQ { rs, rt, imm } => {
                if self.read_reg(rs) == self.read_reg(rt) {
                    let off = (imm as i16 as i32 * 4) as u32;
                    self.jump_delay = Some(self.pc.wrapping_add(4).wrapping_add(off))
                }
            }
            Instruction::BNE { rs, rt, imm } => {
                if self.read_reg(rs) != self.read_reg(rt) {
                    let off = (imm as i16 as i32 * 4) as u32;
                    self.jump_delay = Some(self.pc.wrapping_add(4).wrapping_add(off))
                }
            }
            Instruction::BLTZ { rs, imm } => {
                if (self.read_reg(rs) as i32) < 0 {
                    let off = (imm as i16 as i32 * 4) as u32;
                    self.jump_delay = Some(self.pc.wrapping_add(4).wrapping_add(off))
                }
            }
            Instruction::BGEZ { rs, imm } => {
                if (self.read_reg(rs) as i32) >= 0 {
                    let off = (imm as i16 as i32 * 4) as u32;
                    self.jump_delay = Some(self.pc.wrapping_add(4).wrapping_add(off))
                }
            }
            Instruction::BGTZ { rs, imm } => {
                if (self.read_reg(rs) as i32) > 0 {
                    let off = (imm as i16 as i32 * 4) as u32;
                    self.jump_delay = Some(self.pc.wrapping_add(4).wrapping_add(off))
                }
            }
            Instruction::BLEZ { rs, imm } => {
                if (self.read_reg(rs) as i32) <= 0 {
                    let off = (imm as i16 as i32 * 4) as u32;
                    self.jump_delay = Some(self.pc.wrapping_add(4).wrapping_add(off))
                }
            }
            Instruction::BLTZAL { rs, imm } => {
                self.write_reg(RETURN_ADDR, self.pc.wrapping_add(8));
                if (self.read_reg(rs) as i32) < 0 {
                    let off = (imm as i16 as i32 * 4) as u32;
                    self.jump_delay = Some(self.pc.wrapping_add(4).wrapping_add(off))
                }
            }
            Instruction::BGEZAL { rs, imm } => {
                self.write_reg(RETURN_ADDR, self.pc.wrapping_add(8));
                if (self.read_reg(rs) as i32) >= 0 {
                    let off = (imm as i16 as i32 * 4) as u32;
                    self.jump_delay = Some(self.pc.wrapping_add(4).wrapping_add(off))
                }
            }

            Instruction::COP { n, instr } => {
                if n != 0 && n != 2 {
                    self.trigger_exception(Exception::CoprocessorUnusable, load_delay);
                    return;
                }

                match instr {
                    CopInstruction::MFC { rt, rd } => {
                        let val = cop_dispatch!(self, n, read_reg(rd));
                        self.load_delay = Some((rt, val));
                    }
                    CopInstruction::CFC { rt, rd } => {
                        let val = cop_dispatch!(self, n, read_reg(rd));
                        self.load_delay = Some((rt, val));
                    }
                    CopInstruction::MTC { rt, rd } => {
                        let val = self.read_reg(rt);
                        cop_dispatch!(self, n, write_reg(rd, val));
                    }
                    CopInstruction::CTC { rt, rd } => {
                        let val = self.read_reg(rt);
                        cop_dispatch!(self, n, write_reg(rd, val));
                    }
                    CopInstruction::LWC { rs, rt, imm } => {
                        let addr = self.read_reg(rs).wrapping_add(imm as i16 as u32);
                        let val = mem.read_word(addr);
                        cop_dispatch!(self, n, write_reg(rt, val));
                    }
                    CopInstruction::SWC { rs, rt, imm } => {
                        let val = cop_dispatch!(self, n, read_reg(rt));
                        let addr = self.read_reg(rs).wrapping_add(imm as i16 as u32);
                        mem.write_word(addr, val);
                    }
                    CopInstruction::BCF { imm } => {
                        match n {
                            0 => self.trigger_exception(Exception::CoprocessorUnusable, load_delay),
                            2 => {}
                            _ => unreachable!(),
                        };
                    }
                    CopInstruction::BCT { imm } => {
                        match n {
                            0 => self.trigger_exception(Exception::CoprocessorUnusable, load_delay),
                            2 => {}
                            _ => unreachable!(),
                        };
                    }
                    CopInstruction::COP { cmd } => {
                        match n {
                            0 => self.cop0.exec(cmd),
                            2 => self.gte.exec(cmd),
                            _ => unreachable!(),
                        }
                        // Cop0 only
                    }

                    _ => {}
                }
            }

            Instruction::SYSCALL { comment } => {
                self.trigger_exception(Exception::SystemCall, load_delay)
            }
            Instruction::BREAK { comment } => {
                self.trigger_exception(Exception::Breakpoint, load_delay)
            }

            Instruction::ILLEGAL => {
                self.trigger_exception(Exception::ReservedInstruction, load_delay)
            }
            _ => todo!(),
        }
    }

    fn trigger_exception(&mut self, ex: Exception, load_delay: bool) {
        let new_pc = self.cop0.handle_exception(ex, self.pc, load_delay);
        self.exception_pc = Some(new_pc);
        // panic!("Exception {:?} at PC{:08X}", ex, self.pc);
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new(0xbfc0_0000, 0, 0, 0)
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const REG_NAMES: [&str; 32] = [
            "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5",
            "t6", "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "t8", "t9", "k0", "k1",
            "gp", "sp", "fp", "ra",
        ];
        writeln!(
            f,
            "PC: {:08X}   HI: {:08X}   LO: {:08X}",
            self.pc, self.hi, self.lo
        )?;

        for (i, reg) in self.gprs.iter().enumerate() {
            write!(f, "R{:02}({:>4}): {:08X}   ", i, REG_NAMES[i], reg)?;
            if (i + 1) % 4 == 0 {
                writeln!(f)?;
            }
        }

        if let Some((reg, val)) = self.load_delay {
            writeln!(f, "Load Delay Slot -> R{:02} = {:08X}", reg, val)?;
        }

        Ok(())
    }
}
