mod cpu;
mod memory;
use std::env;
use std::fs;
use std::io::{self, Write};

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file.bin/exe>", args[0]);
        return;
    }
    let fpath = &args[1];
    let data = fs::read(fpath).expect("Can't read file");
    let bios = std::fs::read("SCPH1001.BIN").expect("BIOS SCPH1001.BIN not found!");

    let mut cpu;
    let mut mem = crate::memory::MemoryBus::with_bios(&bios);
    if &data[0..8] == b"PS-X EXE" {
        let (pc, r28, r29_30) = mem.load_psexe(&data);
        cpu = crate::cpu::Cpu::new(pc, r28, r29_30, r29_30);
    } else {
        cpu = crate::cpu::Cpu::default();
    }

    loop {
        wait_for_enter();
        cpu.step(&mut mem);
    }
}
