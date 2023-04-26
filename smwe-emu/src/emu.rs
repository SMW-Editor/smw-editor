#![allow(clippy::identity_op)]

use std::{collections::HashSet, rc::Rc};

use wdc65816::{Cpu, Mem};

use crate::rom::Rom;

#[derive(Debug, Clone)]
pub struct CheckedMem {
    pub cart:       Rc<Rom>,
    pub wram:       Vec<u8>,
    pub regs:       Vec<u8>,
    pub vram:       Vec<u8>,
    pub cgram:      Vec<u8>,
    pub extram:     Vec<u8>,
    pub uninit:     HashSet<usize>,
    pub error:      Option<u32>,
    pub err_value:  Option<u8>,
    pub last_store: Option<u32>,
}

impl CheckedMem {
    pub fn new(rom: Rc<Rom>) -> Self {
        Self {
            cart:       rom,
            wram:       Vec::from([0; 0x20000]),
            regs:       Vec::from([0; 0x6000]),
            vram:       Vec::from([0; 0x10000]),
            cgram:      Vec::from([0; 0x200]),
            extram:     Vec::from([0; 0x10000]),
            uninit:     HashSet::new(),
            error:      None,
            err_value:  None,
            last_store: None,
        }
    }


    pub fn load_u8(&mut self, addr: u32) -> u8 {
        self.load(addr)
    }
    pub fn store_u8(&mut self, addr: u32, value: u8) {
        self.store(addr, value)
    }
    pub fn load_u16(&mut self, addr: u32) -> u16 {
        let l = self.load(addr);
        let h = self.load(addr + 1);
        u16::from_le_bytes([l, h])
    }

    pub fn load_u24(&mut self, addr: u32) -> u32 {
        let l = self.load(addr);
        let h = self.load(addr + 1);
        let b = self.load(addr + 2);
        u32::from_le_bytes([l, h, b, 0])
    }

    pub fn store_u16(&mut self, addr: u32, val: u16) {
        let val = val.to_le_bytes();
        self.store(addr, val[0]);
        self.store(addr + 1, val[1]);
    }

    pub fn store_u24(&mut self, addr: u32, val: u32) {
        let val = val.to_le_bytes();
        self.store(addr, val[0]);
        self.store(addr + 1, val[1]);
        self.store(addr + 2, val[2]);
    }

    pub fn process_dma_ch(&mut self, ch: u32) {
        let a = self.load_u24(0x4302 + ch);
        let size = self.load_u16(0x4305 + ch) as u32;
        let b = self.load(0x4301 + ch);
        let params = self.load(0x4300 + ch);
        // TODO: turn this into reg writes
        if b == 0x18 {
            let dest = self.load_u16(0x2116) as u32;
            //println!("DMA size {:04X}: VRAM ${:02X}:{:04X} => ${:04X}", size, a_bank, a, dest);
            if params & 0x8 != 0 { // fill transfer
                 /*let value = self.load(a_bank, a);
                 for i in dest..dest+size {
                     self.vram[i as usize * 2] = value;
                 }*/
            } else {
                for i in 0..size {
                    self.vram[(dest * 2 + i) as usize] = self.load(a + i);
                }
            }
            self.store_u16(0x2116, (dest + size) as u16);
        } else if false && b == 0x19 {
            let _dest = self.load_u16(0x2116);
            //println!("DMA size {:04X}: VRAMh ${:02X}:{:04X} => ${:04X}", size, a_bank, a, dest);
            if params & 0x8 != 0 { // fill transfer
                 /*let value = self.load(a_bank, a);
                 for i in dest..dest+size {
                     self.vram[i as usize * 2] = value;
                 }*/
            }
        } else if b == 0x22 {
            let dest = self.load(0x2121) as u32;
            // cgram
            for i in 0..size {
                self.cgram[(dest * 2 + i) as usize] = self.load(a + i);
            }
            self.store_u16(0x2121, (dest + size) as u16);
        } else {
            println!("DMA size {size:04X}: ${b:02X} ${a:06X}");
        }
    }

    pub fn process_dma(&mut self) {
        let dma = self.load(0x420B);
        if dma != 0 {
            for i in 0..8 {
                if dma & (1 << i) != 0 {
                    self.process_dma_ch(i * 0x10);
                }
            }
            self.store(0x420B, 0);
        }
    }

    pub fn map(&mut self, addr: u32, write: Option<u8>) -> u8 {
        let track_uninit = false;
        let bank = addr >> 16;
        let mutable = if bank & 0xFE == 0x7E {
            let ptr = (addr & 0x1FFFF) as usize;
            if track_uninit {
                if write.is_none() && !self.uninit.contains(&ptr) {
                    println!("Uninit read: ${:06X}", 0x7E0000 + ptr);
                }
                self.uninit.insert(ptr);
            }
            &mut self.wram[ptr]
        } else if bank == 0x60 {
            let ptr = (addr & 0xFFFF) as usize;
            &mut self.extram[ptr]
        } else if addr & 0xFFFF < 0x2000 {
            let ptr = (addr & 0x1FFF) as usize;
            if track_uninit {
                if write.is_none() && !self.uninit.contains(&ptr) {
                    println!("Uninit read: ${:06X}", 0x7E0000 + ptr);
                }
                self.uninit.insert(ptr);
            }
            &mut self.wram[ptr]
        } else if addr & 0xFFFF < 0x8000 {
            let ptr = (addr & 0x7FFF) as usize;
            if track_uninit {
                if write.is_none() && !self.uninit.contains(&ptr) {
                    //println!("Uninit read: ${:04X}", ptr);
                }
                self.uninit.insert(ptr);
            }
            // TODO: be more accurate
            if let Some(value) = write {
                if ptr == 0x2118 {
                    let addr = self.load_u16(0x2116);
                    self.vram[(addr as usize) * 2 + 0] = value;
                } else if ptr == 0x2119 {
                    let addr = self.load_u16(0x2116);
                    self.vram[(addr as usize) * 2 + 1] = value;
                    self.store_u16(0x2116, addr + 1);
                }
            }
            &mut self.regs[ptr - 0x2000]
        } else if addr & 0xFFFF >= 0x8000 {
            if let Some(c) = self.cart.read(addr) {
                return c;
            } else {
                self.error = Some(addr);
                self.err_value.get_or_insert(0)
            }
        } else {
            self.error = Some(addr);
            self.err_value.get_or_insert(0)
        };
        if let Some(c) = write {
            *mutable = c;
        }
        *mutable
    }
}
impl Mem for CheckedMem {
    #[allow(clippy::let_and_return)]
    fn load(&mut self, addr: u32) -> u8 {
        let value = self.map(addr, None);
        // println!("ld ${:06X} = {:02X}", addr, value);
        value
    }

    fn store(&mut self, addr: u32, value: u8) {
        //println!("st ${:06X} = {:02X}", addr, value);
        self.map(addr, Some(value));
        self.last_store = Some(addr);
    }
}

pub fn fetch_anim_frame(cpu: &mut Cpu<CheckedMem>) -> u64 {
    cpu.s = 0x1FF;
    cpu.pc = 0x2000;
    cpu.pbr = 0x00;
    cpu.dbr = 0x00;
    cpu.trace = false;
    // quasi-loader bytecode
    let routines = [
        "CODE_05BB39", // set up frames
        "CODE_00A390", // upload them
    ];
    let mut addr = 0x2000;
    for symbol in routines {
        cpu.mem.store(addr, 0x22);
        cpu.mem.store_u24(addr + 1, cpu.mem.cart.resolve(symbol).unwrap_or_else(|| panic!("no symbol: {symbol}")));
        addr += 4;
    }
    let mut cy = 0;
    loop {
        cy += cpu.dispatch() as u64;
        //if cy > cy_limit { break; }
        if cpu.ill {
            println!("ILLEGAL INSTR");
            break;
        }
        if cpu.pc == addr as u16 {
            break;
        }
        cpu.mem.process_dma();
    }
    cy
}

pub fn exec_sprites(cpu: &mut Cpu<CheckedMem>) -> u64 {
    let now = std::time::Instant::now();
    cpu.emulation = false;
    /*
    cpu.mem.store(0x9E, id);
    cpu.mem.store(0xAA, 0x80);
    cpu.mem.store(0xE4, 0x80);
    cpu.mem.store(0x14C8, 1);*/
    //cpu.x = slot as u16;
    cpu.s = 0x1FF;
    cpu.pc = 0x2000;
    cpu.pbr = 0x00;
    cpu.dbr = 0x00;
    cpu.trace = false;
    // quasi-loader bytecode
    let routines = [
        "CODE_01808C",
    ];
    let mut addr = 0x2000;
    for i in routines {
        cpu.mem.store(addr, 0x22);
        cpu.mem.store_u24(addr + 1, cpu.mem.cart.resolve(i).unwrap_or_else(|| panic!("no symbol: {}", i)));
        addr += 4;
    }
    let mut cy = 0;
    loop {
        cy += cpu.dispatch() as u64;
        //if cy > cy_limit { break; }
        if cpu.ill {
            println!("ILLEGAL INSTR");
            break;
        }
        if cpu.pc == addr as u16 {
            break;
        }
        cpu.mem.process_dma();
    }
    println!("took {}µs", now.elapsed().as_micros());
    cy
}
pub fn decompress_sublevel(cpu: &mut Cpu<CheckedMem>, id: u16) -> u64 {
    let now = std::time::Instant::now();
    cpu.emulation = false;
    // set submap
    cpu.mem.store(0x1F11, (id >> 8) as _);
    cpu.mem.store(0x141A, 1);
    cpu.s = 0x1FF;
    cpu.pc = 0x2000;
    cpu.pbr = 0x00;
    cpu.dbr = 0x00;
    cpu.trace = false;
    // quasi-loader bytecode
    let routines = [
        "CODE_00A993",     // init layer 3 / sp0
        "CODE_00B888",     // init gfx32/33
        "CODE_05D796",     // init pointers
        "CODE_05801E",     // decompress level
        "UploadSpriteGFX", // upload graphics
        "LoadPalette",     // init palette
        "CODE_00922F",     // upload palette
    ];
    let mut addr = 0x2000;
    for i in routines {
        cpu.mem.store(addr, 0x22);
        cpu.mem.store_u24(addr + 1, cpu.mem.cart.resolve(i).unwrap_or_else(|| panic!("no symbol: {}", i)));
        addr += 4;
    }
    let mut cy = 0;
    loop {
        cy += cpu.dispatch() as u64;
        //if cy > cy_limit { break; }
        if cpu.ill {
            println!("ILLEGAL INSTR");
            break;
        }
        if cpu.pc == 0xD8B7 && cpu.pbr == 0x05 {
            cpu.mem.store_u16(0xE, id);
        }
        if cpu.pc == addr as u16 {
            break;
        }
        cpu.mem.process_dma();
    }
    println!("took {}µs", now.elapsed().as_micros());
    cy
}
