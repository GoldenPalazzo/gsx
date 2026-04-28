const RAM_BEGIN: u32 = 0;
const RAM_SIZE: u32 = 2 * 1024 * 1024;
const RAM_END: u32 = RAM_BEGIN + RAM_SIZE - 1;
const RAM_MIRROR_MASK: u32 = RAM_SIZE - 1;
const RAM_MIRROR_END: u32 = (RAM_BEGIN + RAM_SIZE * 4) - 1;
const BIOS_BEGIN: u32 = 0x1FC0_0000;
const BIOS_SIZE: u32 = 512 * 1024;
const BIOS_END: u32 = BIOS_BEGIN + BIOS_SIZE - 1;
const SCRATCH_BEGIN: u32 = 0x1F80_0000;
const SCRATCH_SIZE: u32 = 1024;
const SCRATCH_END: u32 = SCRATCH_BEGIN + SCRATCH_SIZE - 1;
const CACHECTL_BEGIN: u32 = 0xFFFE_0000;
const CACHECTL_SIZE: u32 = 512;
const CACHECTL_END: u32 = CACHECTL_BEGIN + CACHECTL_SIZE - 1;

pub struct MemoryBus {
    main_ram: Box<[u8; RAM_SIZE as usize]>,
    bios_rom: Box<[u8; BIOS_SIZE as usize]>,
    scratchpad: Box<[u8; SCRATCH_SIZE as usize]>,
}

impl MemoryBus {
    pub fn new() -> Self {
        Self {
            main_ram: Box::new([0u8; RAM_SIZE as usize]),
            bios_rom: Box::new([0u8; BIOS_SIZE as usize]),
            scratchpad: Box::new([0u8; SCRATCH_SIZE as usize]),
        }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        let addr = Self::mask_address(addr);
        match addr {
            RAM_BEGIN..=RAM_MIRROR_END => self.main_ram[(addr & RAM_MIRROR_MASK) as usize],
            BIOS_BEGIN..=BIOS_END => self.bios_rom[(addr - BIOS_BEGIN) as usize],
            SCRATCH_BEGIN..=SCRATCH_END => self.scratchpad[(addr - SCRATCH_BEGIN) as usize],
            CACHECTL_BEGIN..=CACHECTL_END => 0xff,
            _ => todo!("Address {:08X} not mapped yet", addr),
        }
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        u16::from_le_bytes([self.read_byte(addr), self.read_byte(addr.wrapping_add(1))])
    }

    pub fn read_word(&self, addr: u32) -> u32 {
        u32::from_le_bytes([
            self.read_byte(addr),
            self.read_byte(addr.wrapping_add(1)),
            self.read_byte(addr.wrapping_add(2)),
            self.read_byte(addr.wrapping_add(3)),
        ])
    }

    pub fn write_byte(&mut self, addr: u32, val: u8) {
        let addr = Self::mask_address(addr);
        match addr {
            RAM_BEGIN..=RAM_MIRROR_END => self.main_ram[(addr & RAM_MIRROR_MASK) as usize] = val,
            BIOS_BEGIN..=BIOS_END => {}
            SCRATCH_BEGIN..=SCRATCH_END => self.scratchpad[(addr - SCRATCH_BEGIN) as usize] = val,
            CACHECTL_BEGIN..=CACHECTL_END => {}
            _ => todo!("Address {:08X} not mapped yet", addr),
        }
    }
    pub fn write_halfword(&mut self, addr: u32, val: u16) {
        let bytes = val.to_le_bytes();
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr.wrapping_add(1), bytes[1]);
    }
    pub fn write_word(&mut self, addr: u32, val: u32) {
        let bytes = val.to_le_bytes();
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr.wrapping_add(1), bytes[1]);
        self.write_byte(addr.wrapping_add(2), bytes[2]);
        self.write_byte(addr.wrapping_add(3), bytes[3]);
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
