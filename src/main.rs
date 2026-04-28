mod cpu;
mod memory;
use std::io::{self, Write};

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main() {
    let bios = std::fs::read("SCPH1001.BIN").expect("BIOS SCPH1001.BIN not found!");
    assert_eq!(bios.len(), 512 * 1024, "BIOS has to be 512 KB");

    let mut cpu = crate::cpu::Cpu::default();
    let mut mem = crate::memory::MemoryBus::with_bios(&bios);

    loop {
        wait_for_enter();
        cpu.step(&mut mem);
    }
}
