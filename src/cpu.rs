mod coprocessor;
mod execute;
mod instrs;
mod test;

use std::fmt;

use super::memory::BusInterface;
use coprocessor::{Cop0, Coprocessor, Gte};
use instrs::{CopInstruction, Instruction};

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
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

pub struct Cpu {
    gprs: [u32; 32],
    pc: u32,
    hi: u32,
    lo: u32,

    cop0: Cop0,
    gte: Gte,

    curr_opcode: u32,
    load_delay: Option<(u8, u32)>,
    pending_load: Option<(u8, u32)>,

    last_written_reg: Option<u8>,
    taken_branch: Option<u32>,
    in_branch_delay: bool, // necessary for cop0r13 bit 30
    exception_pc: Option<u32>,
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

            curr_opcode: 0,
            load_delay: None,
            pending_load: None,

            last_written_reg: None,
            taken_branch: None,
            in_branch_delay: false,
            exception_pc: None,
        };
        n.gprs[28] = r28;
        n.gprs[29] = r29;
        n.gprs[30] = r30;
        n
    }

    pub fn step<T: BusInterface>(&mut self, mem: &mut T) {
        assert!(self.pc.is_multiple_of(4), "PC is unaligned!!!");

        // Decode
        self.curr_opcode = mem.read_word(self.pc);
        let disasm = Instruction::decode(self.curr_opcode);

        print!("{}", self);
        println!(
            "0x{:08X}: {:08X} -> {:?}",
            self.pc, self.curr_opcode, disasm
        );

        self.pending_load = self.load_delay.take();
        let pending_branch = self.taken_branch;
        let old_branch_delay = self.in_branch_delay;

        self.execute(disasm, mem);

        println!(
            "old_branch_delay={} new_branch_delay={}",
            old_branch_delay, self.in_branch_delay
        );
        let last_reg = self.last_written_reg.take().unwrap_or(u8::MAX);

        if old_branch_delay && self.in_branch_delay {
            self.in_branch_delay = false;
            self.taken_branch = None;
            if let Some(pc) = pending_branch {
                self.pc = pc;
            } else {
                self.pc = self.pc.wrapping_add(4);
            }
        } else {
            self.pc = self.pc.wrapping_add(4);
        }
        if let Some(epc) = self.exception_pc.take() {
            self.in_branch_delay = false;
            self.pc = epc;
        }

        if let Some((reg, val)) = self.pending_load.take()
            && reg != last_reg
        {
            println!("Applying load delay slot! r{} val={:08X}", reg, val);
            if reg != 0 {
                self.gprs[reg as usize] = val;
            }
        }
    }

    fn read_reg(&self, idx: u8) -> u32 {
        self.gprs[idx as usize]
    }

    fn write_reg(&mut self, idx: u8, val: u32) {
        if idx != 0 {
            self.gprs[idx as usize] = val;
            self.last_written_reg = Some(idx);
        }
    }

    fn write_load_delay(&mut self, idx: u8, val: u32) {
        self.load_delay = Some((idx, val));
        self.last_written_reg = Some(idx);
    }

    fn trigger_exception(&mut self, ex: Exception) {
        println!("Exception {:?} at PC {:08X}", ex, self.pc);
        let new_pc = self.cop0.handle_exception(
            ex,
            self.curr_opcode,
            self.pc,
            self.in_branch_delay,
            self.taken_branch.is_some(),
        );
        self.exception_pc = Some(new_pc);
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

        if let Some(pc) = self.taken_branch {
            writeln!(f, "In branch delay -> PC = {:08X}", pc)?;
        }

        Ok(())
    }
}
