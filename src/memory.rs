use tracing::{debug, info, warn};

const RAM_BEGIN: u32 = 0;
const RAM_SIZE: u32 = 2 * 1024 * 1024;
const RAM_END: u32 = RAM_BEGIN + RAM_SIZE - 1;
const RAM_MIRROR_MASK: u32 = RAM_SIZE - 1;
const RAM_MIRROR_END: u32 = (RAM_BEGIN + RAM_SIZE * 4) - 1;

const EXP1_BEGIN: u32 = 0x1F00_0000;
const EXP1_SIZE: u32 = 8 * 1024 * 1024;
const EXP1_END: u32 = EXP1_BEGIN + EXP1_SIZE - 1;

const BIOS_BEGIN: u32 = 0x1FC0_0000;
const BIOS_SIZE: u32 = 512 * 1024;
const BIOS_END: u32 = BIOS_BEGIN + BIOS_SIZE - 1;

const SCRATCH_BEGIN: u32 = 0x1F80_0000;
const SCRATCH_SIZE: u32 = 1024;
const SCRATCH_END: u32 = SCRATCH_BEGIN + SCRATCH_SIZE - 1;

const IOPORTS_BEGIN: u32 = 0x1F801000;
const IOPORTS_SIZE: u32 = 8 * 1024;
const IOPORTS_END: u32 = IOPORTS_BEGIN + IOPORTS_SIZE - 1;

const CACHECTL_BEGIN: u32 = 0xFFFE_0000;
const CACHECTL_SIZE: u32 = 512;
const CACHECTL_END: u32 = CACHECTL_BEGIN + CACHECTL_SIZE - 1;

pub trait BusInterface {
    fn read_byte(&self, addr: u32) -> u8;
    fn read_halfword(&self, addr: u32) -> u16;
    fn read_word(&self, addr: u32) -> u32;
    fn write_byte(&mut self, addr: u32, val: u8);
    fn write_halfword(&mut self, addr: u32, val: u16);
    fn write_word(&mut self, addr: u32, val: u32);
}

pub struct MemoryBus {
    main_ram: Box<[u8; RAM_SIZE as usize]>,
    exp1_ram: Box<[u8; EXP1_SIZE as usize]>,
    bios_rom: Box<[u8; BIOS_SIZE as usize]>,
    scratchpad: Box<[u8; SCRATCH_SIZE as usize]>,
}

impl BusInterface for MemoryBus {
    fn read_byte(&self, addr: u32) -> u8 {
        let addr = Self::mask_address(addr);
        match addr {
            RAM_BEGIN..=RAM_MIRROR_END => self.main_ram[(addr & RAM_MIRROR_MASK) as usize],
            EXP1_BEGIN..=EXP1_END => self.exp1_ram[(addr - EXP1_BEGIN) as usize],
            BIOS_BEGIN..=BIOS_END => self.bios_rom[(addr - BIOS_BEGIN) as usize],
            SCRATCH_BEGIN..=SCRATCH_END => self.scratchpad[(addr - SCRATCH_BEGIN) as usize],
            IOPORTS_BEGIN..=IOPORTS_END => 0xff,
            CACHECTL_BEGIN..=CACHECTL_END => 0xff,
            _ => todo!("Address {:08X} not mapped yet", addr),
        }
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
        let addr = Self::mask_address(addr);
        match addr {
            RAM_BEGIN..=RAM_MIRROR_END => self.main_ram[(addr & RAM_MIRROR_MASK) as usize] = val,
            BIOS_BEGIN..=BIOS_END => {}
            EXP1_BEGIN..=EXP1_END => self.exp1_ram[(addr - EXP1_BEGIN) as usize] = val,
            SCRATCH_BEGIN..=SCRATCH_END => self.scratchpad[(addr - SCRATCH_BEGIN) as usize] = val,
            IOPORTS_BEGIN..=IOPORTS_END => warn!("IO write {:08X} = {:08X}", addr, val),
            CACHECTL_BEGIN..=CACHECTL_END => {}
            _ => todo!("Address {:08X} not mapped yet", addr),
        }
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

impl MemoryBus {
    pub fn new() -> Self {
        Self {
            main_ram: vec![0u8; RAM_SIZE as usize]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            exp1_ram: vec![0u8; EXP1_SIZE as usize]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            bios_rom: vec![0u8; BIOS_SIZE as usize]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            scratchpad: vec![0u8; SCRATCH_SIZE as usize]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        }
    }

    pub fn with_bios(bios: &[u8]) -> Self {
        assert_eq!(bios.len(), BIOS_SIZE as usize, "BIOS has to be 512 KB");
        let mut bus = Self::new();
        bus.bios_rom.copy_from_slice(bios);
        bus
    }

    pub fn load_psexe(&mut self, exe: &[u8]) -> (u32, u32, u32) {
        debug!("Loading PS-X EXE...");
        let initial_pc = u32::from_le_bytes(exe[0x10..0x14].try_into().unwrap());
        let initial_r28 = u32::from_le_bytes(exe[0x14..0x18].try_into().unwrap());
        let dest_addr = u32::from_le_bytes(exe[0x18..0x1c].try_into().unwrap());
        let filesize = u32::from_le_bytes(exe[0x1c..0x20].try_into().unwrap());
        let memfill_start = u32::from_le_bytes(exe[0x28..0x2c].try_into().unwrap());
        let memfill_size = u32::from_le_bytes(exe[0x2c..0x30].try_into().unwrap());
        let initial_r29_r30 = u32::from_le_bytes(exe[0x30..0x34].try_into().unwrap())
            .wrapping_add(u32::from_le_bytes(exe[0x34..0x38].try_into().unwrap()));
        for byte in 0..filesize {
            let addr = dest_addr.wrapping_add(byte);
            self.write_byte(addr, exe[0x800 + byte as usize]);
        }
        debug!(
            "Loaded {} bytes ({:08X} to {:08X})",
            filesize,
            dest_addr,
            dest_addr.wrapping_add(filesize)
        );

        for byte in 0..memfill_size {
            let addr = memfill_start.wrapping_add(byte);
            self.write_byte(addr, 0);
        }
        debug!(
            "Cleaned {} bytes ({:08X} to {:08X})",
            memfill_size,
            memfill_start,
            memfill_start.wrapping_add(memfill_size)
        );
        debug!(
            "Loaded PS-X EXE\nPC: {:08X} R28: {:08X} R29: {:08X} R30: {:08X}",
            initial_pc, initial_r28, initial_r29_r30, initial_r29_r30
        );

        (initial_pc, initial_r28, initial_r29_r30)
    }

    #[inline(always)]
    fn mask_address(addr: u32) -> u32 {
        const REGION_MASK: [u32; 8] = [
            0x1FFF_FFFF,
            0x1FFF_FFFF,
            0x1FFF_FFFF,
            0x1FFF_FFFF,
            0x1FFF_FFFF,
            0x1FFF_FFFF,
            0xFFFF_FFFF,
            0xFFFF_FFFF,
        ];
        addr & REGION_MASK[(addr >> 29) as usize]
    }
}
