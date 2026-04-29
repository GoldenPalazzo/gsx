#[inline(always)]
fn get_rs(opcode: u32) -> u8 {
    ((opcode >> 21) & 0x1f).try_into().unwrap()
}
#[inline(always)]
fn get_rt(opcode: u32) -> u8 {
    ((opcode >> 16) & 0x1f).try_into().unwrap()
}
#[inline(always)]
fn get_rd(opcode: u32) -> u8 {
    ((opcode >> 11) & 0x1f).try_into().unwrap()
}
#[inline(always)]
fn get_imm5(opcode: u32) -> u8 {
    ((opcode >> 6) & 0x1f).try_into().unwrap()
}
#[inline(always)]
fn get_imm16(opcode: u32) -> u16 {
    (opcode & 0xffff).try_into().unwrap()
}
#[inline(always)]
fn get_imm25(opcode: u32) -> u32 {
    opcode & 0x01ff_ffff
}
#[inline(always)]
fn get_imm26(opcode: u32) -> u32 {
    opcode & 0x03ff_ffff
}
#[inline(always)]
fn get_comment(opcode: u32) -> u32 {
    (opcode >> 6) & 0x000f_ffff
}
#[inline(always)]
fn get_primary(opcode: u32) -> u8 {
    ((opcode >> 26) & 0x3f).try_into().unwrap()
}
#[inline(always)]
fn get_secondary(opcode: u32) -> u8 {
    (opcode & 0x3f).try_into().unwrap()
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum CopInstruction {
    MFC { rt: u8, rd: u8 },
    CFC { rt: u8, rd: u8 },
    MTC { rt: u8, rd: u8 },
    CTC { rt: u8, rd: u8 },
    BCF { imm: u16 },
    BCT { imm: u16 },
    LWC { rs: u8, rt: u8, imm: u16 },
    SWC { rs: u8, rt: u8, imm: u16 },

    // Only COP0
    RFE,
    TLBR,
    TLBWI,
    TLBWR,
    TLBP,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum Instruction {
    BLTZ { rs: u8, imm: u16 },
    BGEZ { rs: u8, imm: u16 },
    BLTZAL { rs: u8, imm: u16 },
    BGEZAL { rs: u8, imm: u16 },
    J { imm: u32 },
    JAL { imm: u32 },
    BEQ { rs: u8, rt: u8, imm: u16 },
    BNE { rs: u8, rt: u8, imm: u16 },
    BLEZ { rs: u8, imm: u16 },
    BGTZ { rs: u8, imm: u16 },
    ADDI { rs: u8, rt: u8, imm: u16 },
    ADDIU { rs: u8, rt: u8, imm: u16 },
    SLTI { rs: u8, rt: u8, imm: u16 },
    SLTIU { rs: u8, rt: u8, imm: u16 },
    ANDI { rs: u8, rt: u8, imm: u16 },
    ORI { rs: u8, rt: u8, imm: u16 },
    XORI { rs: u8, rt: u8, imm: u16 },
    LUI { rt: u8, imm: u16 },
    COP { n: u8, instr: CopInstruction },
    LB { rs: u8, rt: u8, imm: u16 },
    LH { rs: u8, rt: u8, imm: u16 },
    LWL { rs: u8, rt: u8, imm: u16 },
    LW { rs: u8, rt: u8, imm: u16 },
    LBU { rs: u8, rt: u8, imm: u16 },
    LHU { rs: u8, rt: u8, imm: u16 },
    LWR { rs: u8, rt: u8, imm: u16 },
    SB { rs: u8, rt: u8, imm: u16 },
    SH { rs: u8, rt: u8, imm: u16 },
    SWL { rs: u8, rt: u8, imm: u16 },
    SW { rs: u8, rt: u8, imm: u16 },
    SWR { rs: u8, rt: u8, imm: u16 },

    // Special instructions
    SLL { rt: u8, rd: u8, imm: u8 },
    SRL { rt: u8, rd: u8, imm: u8 },
    SRA { rt: u8, rd: u8, imm: u8 },
    SLLV { rs: u8, rt: u8, rd: u8 },
    SRLV { rs: u8, rt: u8, rd: u8 },
    SRAV { rs: u8, rt: u8, rd: u8 },
    JR { rs: u8 },
    JALR { rs: u8, rd: u8 },
    SYSCALL { comment: u32 },
    BREAK { comment: u32 },
    MFHI { rd: u8 },
    MTHI { rs: u8 },
    MFLO { rd: u8 },
    MTLO { rs: u8 },
    MULT { rs: u8, rt: u8 },
    MULTU { rs: u8, rt: u8 },
    DIV { rs: u8, rt: u8 },
    DIVU { rs: u8, rt: u8 },
    ADD { rs: u8, rt: u8, rd: u8 },
    ADDU { rs: u8, rt: u8, rd: u8 },
    SUB { rs: u8, rt: u8, rd: u8 },
    SUBU { rs: u8, rt: u8, rd: u8 },
    AND { rs: u8, rt: u8, rd: u8 },
    OR { rs: u8, rt: u8, rd: u8 },
    XOR { rs: u8, rt: u8, rd: u8 },
    NOR { rs: u8, rt: u8, rd: u8 },
    SLT { rs: u8, rt: u8, rd: u8 },
    SLTU { rs: u8, rt: u8, rd: u8 },

    ILLEGAL,
    RESERVED,
}

impl Instruction {
    pub fn decode(opcode: u32) -> Self {
        match get_primary(opcode) {
            0x00 => Self::decode_special(opcode),
            0x01 => Self::decode_bcondz(opcode),
            0x02 => Self::J {
                imm: get_imm26(opcode),
            },
            0x03 => Self::JAL {
                imm: get_imm26(opcode),
            },
            0x04 => Self::BEQ {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x05 => Self::BNE {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x06 => Self::BLEZ {
                rs: get_rs(opcode),
                imm: get_imm16(opcode),
            },
            0x07 => Self::BGTZ {
                rs: get_rs(opcode),
                imm: get_imm16(opcode),
            },
            0x08..=0x0e => Self::decode_aluimm(opcode),
            0x0f => Self::LUI {
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x20..=0x27 => Self::decode_load(opcode),
            0x28..=0x2f => Self::decode_store(opcode),
            0x10..=0x1f | 0x30..=0x3f => Self::decode_cop(opcode),
            _ => Self::ILLEGAL,
        }
    }

    #[inline(always)]
    fn decode_special(opcode: u32) -> Self {
        match get_secondary(opcode) {
            0x00 => Self::SLL {
                rt: get_rt(opcode),
                rd: get_rd(opcode),
                imm: get_imm5(opcode),
            },
            0x02 => Self::SRL {
                rt: get_rt(opcode),
                rd: get_rd(opcode),
                imm: get_imm5(opcode),
            },
            0x03 => Self::SRA {
                rt: get_rt(opcode),
                rd: get_rd(opcode),
                imm: get_imm5(opcode),
            },
            0x04 => Self::SLLV {
                rt: get_rt(opcode),
                rd: get_rd(opcode),
                rs: get_rs(opcode),
            },
            0x06 => Self::SRLV {
                rt: get_rt(opcode),
                rd: get_rd(opcode),
                rs: get_rs(opcode),
            },
            0x07 => Self::SRAV {
                rt: get_rt(opcode),
                rd: get_rd(opcode),
                rs: get_rs(opcode),
            },
            0x08 => Self::JR { rs: get_rs(opcode) },
            0x09 => Self::JALR {
                rs: get_rs(opcode),
                rd: get_rd(opcode),
            },
            0x0c => Self::SYSCALL {
                comment: get_comment(opcode),
            },
            0x0d => Self::BREAK {
                comment: get_comment(opcode),
            },
            0x10 => Self::MFHI { rd: get_rd(opcode) },
            0x11 => Self::MTHI { rs: get_rs(opcode) },
            0x12 => Self::MFLO { rd: get_rd(opcode) },
            0x13 => Self::MTLO { rs: get_rs(opcode) },
            0x18 => Self::MULT {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
            },
            0x19 => Self::MULTU {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
            },
            0x1a => Self::DIV {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
            },
            0x1b => Self::DIVU {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
            },
            0x20 => Self::ADD {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                rd: get_rd(opcode),
            },
            0x21 => Self::ADDU {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                rd: get_rd(opcode),
            },
            0x22 => Self::SUB {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                rd: get_rd(opcode),
            },
            0x23 => Self::SUBU {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                rd: get_rd(opcode),
            },
            0x24 => Self::AND {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                rd: get_rd(opcode),
            },
            0x25 => Self::OR {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                rd: get_rd(opcode),
            },
            0x26 => Self::XOR {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                rd: get_rd(opcode),
            },
            0x27 => Self::NOR {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                rd: get_rd(opcode),
            },
            0x2a => Self::SLT {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                rd: get_rd(opcode),
            },
            0x2b => Self::SLTU {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                rd: get_rd(opcode),
            },
            _ => Self::ILLEGAL,
        }
    }

    #[inline(always)]
    fn decode_bcondz(opcode: u32) -> Self {
        match get_rt(opcode) {
            0x00 => Self::BLTZ {
                rs: get_rs(opcode),
                imm: get_imm16(opcode),
            },
            0x01 => Self::BGEZ {
                rs: get_rs(opcode),
                imm: get_imm16(opcode),
            },
            0x10 => Self::BLTZAL {
                rs: get_rs(opcode),
                imm: get_imm16(opcode),
            },
            0x11 => Self::BGEZAL {
                rs: get_rs(opcode),
                imm: get_imm16(opcode),
            },
            // TODO: untreated dupes
            _ => Self::ILLEGAL,
        }
    }

    #[inline(always)]
    fn decode_aluimm(opcode: u32) -> Self {
        match get_primary(opcode) {
            0x08 => Self::ADDI {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x09 => Self::ADDIU {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x0a => Self::SLTI {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x0b => Self::SLTIU {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },

            0x0c => Self::ANDI {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x0d => Self::ORI {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x0e => Self::XORI {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn decode_load(opcode: u32) -> Self {
        match get_primary(opcode) {
            0x20 => Self::LB {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x21 => Self::LH {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x22 => Self::LWL {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x23 => Self::LW {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x24 => Self::LBU {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x25 => Self::LHU {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x26 => Self::LWR {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            _ => Self::ILLEGAL,
        }
    }

    #[inline(always)]
    fn decode_store(opcode: u32) -> Self {
        match get_primary(opcode) {
            0x28 => Self::SB {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x29 => Self::SH {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x2a => Self::SWL {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x2b => Self::SW {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            0x2e => Self::SWR {
                rs: get_rs(opcode),
                rt: get_rt(opcode),
                imm: get_imm16(opcode),
            },
            _ => Self::ILLEGAL,
        }
    }

    #[inline(always)]
    fn decode_cop(opcode: u32) -> Self {
        let rs = get_rs(opcode);
        let n = get_primary(opcode) & 3;
        match get_primary(opcode) & 0x38 {
            0x10 => match rs {
                0 => Instruction::COP {
                    n,
                    instr: CopInstruction::MFC {
                        rt: get_rt(opcode),
                        rd: get_rd(opcode),
                    },
                },
                2 => Instruction::COP {
                    n,
                    instr: CopInstruction::CFC {
                        rt: get_rt(opcode),
                        rd: get_rd(opcode),
                    },
                },
                4 => Instruction::COP {
                    n,
                    instr: CopInstruction::MTC {
                        rt: get_rt(opcode),
                        rd: get_rd(opcode),
                    },
                },
                6 => Instruction::COP {
                    n,
                    instr: CopInstruction::CTC {
                        rt: get_rt(opcode),
                        rd: get_rd(opcode),
                    },
                },
                8 => Instruction::COP {
                    n,
                    instr: match get_rt(opcode) {
                        0 => CopInstruction::BCF {
                            imm: get_imm16(opcode),
                        },
                        1 => CopInstruction::BCT {
                            imm: get_imm16(opcode),
                        },
                        _ => unreachable!(),
                    },
                },
                0x10..=0x1f => Instruction::COP {
                    n,
                    instr: if n == 0 {
                        match get_secondary(opcode) {
                            1 => CopInstruction::TLBR,
                            2 => CopInstruction::TLBWI,
                            6 => CopInstruction::TLBWR,
                            8 => CopInstruction::TLBP,
                            16 => CopInstruction::RFE,
                            _ => unreachable!(),
                        }
                    } else {
                        todo!(
                            "Not implemented COPn imm25 (COP{} {:08X})",
                            n,
                            get_imm25(opcode)
                        )
                    },
                },
                _ => unreachable!(),
            },
            0x30 => Instruction::COP {
                n,
                instr: CopInstruction::LWC {
                    rs,
                    rt: get_rt(opcode),
                    imm: get_imm16(opcode),
                },
            },
            0x38 => Instruction::COP {
                n,
                instr: CopInstruction::SWC {
                    rs,
                    rt: get_rt(opcode),
                    imm: get_imm16(opcode),
                },
            },
            _ => unreachable!(
                "Instruction {:08X} ({:032b}) not recognized",
                opcode, opcode
            ),
        }
    }
}
