#[cfg(test)]
#[allow(dead_code)]
mod testing {
    use crate::cpu::{Cpu, coprocessor::Coprocessor};
    use crate::memory::BusInterface;
    use serde::Deserialize;
    use std::path::{Path, PathBuf};

    #[derive(Deserialize)]
    struct LoadDelay {
        slot: bool,
        take: bool,
        target: u32,
    }

    #[derive(Deserialize)]
    struct BranchDelay {
        target: i64, // -1 = None
        val: u32,
    }

    #[derive(Deserialize)]
    struct Delay {
        load: LoadDelay,
        branch: BranchDelay,
    }

    #[derive(Deserialize)]
    #[allow(non_snake_case)]
    struct CpuState {
        R: [u32; 32],
        hi: u32,
        lo: u32,
        EPC: u32,
        CAUSE: u32,
        PC: u32,
        delay: Delay,
    }

    #[derive(Deserialize)]
    struct MemAccess {
        addr: u32,
        val: u32,
        sz: u8,
    }

    pub struct MockBus {
        mem: Box<[u8; u32::MAX as usize]>,
    }

    impl BusInterface for MockBus {
        fn read_byte(&self, addr: u32) -> u8 {
            self.mem[addr as usize]
        }

        fn read_halfword(&self, addr: u32) -> u16 {
            u16::from_le_bytes([self.read_byte(addr), self.read_byte(addr.wrapping_add(1))])
        }

        fn read_word(&self, addr: u32) -> u32 {
            u32::from_le_bytes([
                self.read_byte(addr),
                self.read_byte(addr.wrapping_add(1)),
                self.read_byte(addr.wrapping_add(2)),
                self.read_byte(addr.wrapping_add(3)),
            ])
        }

        fn write_byte(&mut self, addr: u32, val: u8) {
            self.mem[addr as usize] = val;
        }
        fn write_halfword(&mut self, addr: u32, val: u16) {
            let bytes = val.to_le_bytes();
            self.write_byte(addr, bytes[0]);
            self.write_byte(addr.wrapping_add(1), bytes[1]);
        }
        fn write_word(&mut self, addr: u32, val: u32) {
            let bytes = val.to_le_bytes();
            self.write_byte(addr, bytes[0]);
            self.write_byte(addr.wrapping_add(1), bytes[1]);
            self.write_byte(addr.wrapping_add(2), bytes[2]);
            self.write_byte(addr.wrapping_add(3), bytes[3]);
        }
    }

    impl MockBus {
        pub fn new() -> Self {
            Self {
                mem: vec![0u8; u32::MAX as usize]
                    .into_boxed_slice()
                    .try_into()
                    .unwrap(),
            }
        }
    }

    #[derive(Deserialize)]
    struct TestCase {
        name: String,
        #[serde(rename = "final")]
        expected: CpuState,
        initial: CpuState,
        cycles: Vec<MemAccess>,
    }

    fn make_mem(test: &TestCase) -> MockBus {
        let mut mem = MockBus::new();
        for cycle in &test.cycles {
            match cycle.sz {
                1 => mem.write_byte(cycle.addr, cycle.val as u8),
                2 => mem.write_halfword(cycle.addr, cycle.val as u16),
                4 => mem.write_word(cycle.addr, cycle.val),
                _ => {}
            }
        }
        mem
    }

    fn apply_state(cpu: &mut Cpu, state: &CpuState) {
        cpu.gprs = state.R;
        // cpu.gprs[0] = 0;
        cpu.pc = state.PC;
        cpu.hi = state.hi;
        cpu.lo = state.lo;
        cpu.cop0.write_reg(13, state.CAUSE);
        cpu.cop0.write_reg(14, state.EPC);

        cpu.load_delay = if state.delay.branch.target >= 0 {
            Some((state.delay.branch.target as u8, state.delay.branch.val))
        } else {
            None
        };
        cpu.in_branch_delay = state.delay.load.slot;
        cpu.taken_branch = if state.delay.load.take {
            Some(state.delay.load.target)
        } else {
            None
        };

        cpu.exception_pc = None;
    }

    #[derive(Debug)]
    struct Mismatch {
        field: String,
        got: String,
        expected: String,
    }

    fn compare_state(cpu: &Cpu, mem: &MockBus, test: &TestCase) -> Vec<Mismatch> {
        let exp = &test.expected;
        let mut mismatches = Vec::new();

        macro_rules! check {
            ($label:expr, $got:expr, $expected:expr) => {
                if $got != $expected {
                    mismatches.push(Mismatch {
                        field: $label.to_string(),
                        got: $got,
                        expected: $expected,
                    });
                }
            };
        }

        check!("PC", format!("{:08X}", cpu.pc), format!("{:08X}", exp.PC));
        check!("HI", format!("{:08X}", cpu.hi), format!("{:08X}", exp.hi));
        check!("LO", format!("{:08X}", cpu.lo), format!("{:08X}", exp.lo));
        check!(
            "COP0.CAUSE",
            format!("{:08X}", cpu.cop0.read_reg(13)),
            format!("{:08X}", exp.CAUSE)
        );
        check!(
            "COP0.EPC",
            format!("{:08X}", cpu.cop0.read_reg(14)),
            format!("{:08X}", exp.EPC)
        );

        for i in 1..32usize {
            check!(
                format!("R{:02}", i),
                format!("{:08X}", cpu.gprs[i]),
                format!("{:08X}", exp.R[i])
            );
        }

        let exp_branch = if exp.delay.branch.target >= 0 {
            Some((exp.delay.branch.target as u8, exp.delay.branch.val))
        } else {
            None
        };
        if cpu.load_delay != exp_branch {
            mismatches.push(Mismatch {
                field: "load_delay".to_string(),
                got: format!("{:?}", cpu.load_delay),
                expected: format!("{:?}", exp_branch),
            });
        }

        for cycle in &test.cycles {
            if cycle.addr != test.initial.PC {
                let got = match cycle.sz {
                    1 => mem.read_byte(cycle.addr) as u32,
                    2 => mem.read_halfword(cycle.addr) as u32,
                    4 => mem.read_word(cycle.addr),
                    _ => continue,
                };
                check!(
                    format!("mem[{:08X}]", cycle.addr),
                    format!("{:08X}", got),
                    format!("{:08X}", cycle.val)
                );
            }
        }

        mismatches
    }

    pub fn run_tests_from_file(json_path: &Path) -> (usize, usize) {
        let raw = std::fs::read_to_string(json_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", json_path.display(), e));

        let tests: Vec<TestCase> = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("JSON parse error in {}: {}", json_path.display(), e));

        let mut passed = 0;
        let mut failed = 0;

        for test in &tests {
            let mut mem = make_mem(test);
            let mut cpu = Cpu::new(test.initial.PC, 0, 0, 0);
            apply_state(&mut cpu, &test.initial);

            cpu.step(&mut mem);

            let mismatches = compare_state(&cpu, &mem, test);
            if mismatches.is_empty() {
                passed += 1;
            } else {
                failed += 1;
                eprintln!("FAIL: {}", test.name);
                for m in &mismatches {
                    eprintln!("  {} => got {}, expected {}", m.field, m.got, m.expected);
                }
            }
        }

        (passed, failed)
    }

    fn tests_dir() -> PathBuf {
        std::env::var("R3000_TESTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("tests/r3000"))
    }

    macro_rules! instr_test {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let path = tests_dir().join($file);
                if !path.exists() {
                    panic!("{}: file not found", $file);
                }
                let (passed, failed) = run_tests_from_file(&path);
                println!("{}: {}/{} passed", $file, passed, passed + failed);
                assert_eq!(failed, 0, "{} tests failed in {}", failed, $file);
            }
        };
    }

    // ALU
    instr_test!(test_add, "ADD.json");
    instr_test!(test_addu, "ADDU.json");
    instr_test!(test_addi, "ADDI.json");
    instr_test!(test_addiu, "ADDIU.json");
    instr_test!(test_sub, "SUB.json");
    instr_test!(test_subu, "SUBU.json");
    instr_test!(test_and, "AND.json");
    instr_test!(test_or, "OR.json");
    instr_test!(test_xor, "XOR.json");
    instr_test!(test_nor, "NOR.json");
    instr_test!(test_andi, "ANDI.json");
    instr_test!(test_ori, "ORI.json");
    instr_test!(test_xori, "XORI.json");
    instr_test!(test_lui, "LUI.json");
    instr_test!(test_slt, "SLT.json");
    instr_test!(test_sltu, "SLTU.json");
    instr_test!(test_slti, "SLTI.json");
    instr_test!(test_sltiu, "SLTIU.json");

    // Shift
    instr_test!(test_sll, "SLL.json");
    instr_test!(test_srl, "SRL.json");
    instr_test!(test_sra, "SRA.json");
    instr_test!(test_sllv, "SLLV.json");
    instr_test!(test_srlv, "SRLV.json");
    instr_test!(test_srav, "SRAV.json");

    // Mul/Div
    instr_test!(test_mult, "MULT.json");
    instr_test!(test_multu, "MULTU.json");
    instr_test!(test_div, "DIV.json");
    instr_test!(test_divu, "DIVU.json");
    instr_test!(test_mfhi, "MFHI.json");
    instr_test!(test_mflo, "MFLO.json");
    instr_test!(test_mthi, "MTHI.json");
    instr_test!(test_mtlo, "MTLO.json");

    // Load/Store
    instr_test!(test_lb, "LB.json");
    instr_test!(test_lbu, "LBU.json");
    instr_test!(test_lh, "LH.json");
    instr_test!(test_lhu, "LHU.json");
    instr_test!(test_lw, "LW.json");
    instr_test!(test_lwl, "LWL.json");
    instr_test!(test_lwr, "LWR.json");
    instr_test!(test_sb, "SB.json");
    instr_test!(test_sw, "SW.json");
    instr_test!(test_swl, "SWL.json");
    instr_test!(test_swr, "SWR.json");

    // Branch/Jump
    instr_test!(test_j, "J.json");
    instr_test!(test_jal, "JAL.json");
    instr_test!(test_jr, "JR.json");
    instr_test!(test_jalr, "JALR.json");
    instr_test!(test_beq, "BEQ.json");
    instr_test!(test_bne, "BNE.json");
    instr_test!(test_blez, "BLEZ.json");
    instr_test!(test_bgtz, "BGTZ.json");
}
