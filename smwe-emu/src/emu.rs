use wdc65816::{Cpu, Mem};
use std::collections::HashSet;
use crate::rom::Rom;

#[derive(Clone)]
pub struct CheckedMem<'a> {
    pub cart: &'a Rom,
    pub wram: Vec<u8>,
    pub regs: Vec<u8>,
    pub vram: Vec<u8>,
    pub extram: Vec<u8>,
    pub uninit: HashSet<usize>,
    pub error: Option<u32>,
    pub err_value: Option<u8>,
    pub last_store: Option<u32>,
}

impl<'a> CheckedMem<'a> {
    pub fn load_u16(&mut self, addr: u32) -> u16 {
        let l = self.load(addr) as u16;
        let h = self.load(addr + 1) as u16;
        l | (h << 8)
    }
    pub fn load_u24(&mut self, addr: u32) -> u32 {
        let l = self.load(addr) as u32;
        let h = self.load(addr + 1) as u32;
        let b = self.load(addr + 2) as u32;
        l | (h << 8) | (b << 16)
    }
    pub fn process_dma_ch(&mut self, ch: u32) {
        let a = self.load_u24(0x4302 + ch);
        let size = self.load_u16(0x4305 + ch) as u32;
        let b = self.load(0x4301 + ch);
        let params = self.load(0x4300 + ch);
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
                    self.vram[(dest*2 + i) as usize] = self.load(a + i);
                }
            }
        } else if b == 0x19 {
            let _dest = self.load_u16(0x2116);
            //println!("DMA size {:04X}: VRAMh ${:02X}:{:04X} => ${:04X}", size, a_bank, a, dest);
            if params & 0x8 != 0 { // fill transfer
                /*let value = self.load(a_bank, a);
                for i in dest..dest+size {
                    self.vram[i as usize * 2] = value;
                }*/
            }
        } else {
            println!("DMA size {size:04X}: ${b:02X} ${a:06X}");
        }
    }
    pub fn process_dma(&mut self) {
        let dma = self.load(0x420B);
        if dma != 0 {
            for i in 0..8 {
                if dma & (1<<i) != 0 {
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
        } else if addr < 0x2000 {
            let ptr = (addr & 0x1FFF) as usize;
            if track_uninit {
                if write.is_none() && !self.uninit.contains(&ptr) {
                    println!("Uninit read: ${:06X}", 0x7E0000 + ptr);
                }
                self.uninit.insert(ptr);
            }
            &mut self.wram[ptr]
        } else if addr < 0x8000 {
            let ptr = (addr & 0x7FFF) as usize;
            if track_uninit {
                if write.is_none() && !self.uninit.contains(&ptr) {
                    //println!("Uninit read: ${:04X}", ptr);
                }
                self.uninit.insert(ptr);
            }
            &mut self.regs[ptr-0x2000]
        } else if addr > 0x8000 {
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
impl<'a> Mem for CheckedMem<'a> {
    fn load(&mut self, addr: u32) -> u8 {
        self.map(addr, None)
    }
    fn store(&mut self, addr: u32, value: u8) {
        //println!("store ${:02X}:{:04X} = {:02X}", bank, addr, value);
        self.map(addr, Some(value));
        self.last_store = Some(addr);
    }
}
