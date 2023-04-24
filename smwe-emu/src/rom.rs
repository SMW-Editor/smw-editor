#![allow(clippy::identity_op)]

//! Storage for ROM files, mapper support, etc.

use std::collections::HashMap;

#[derive(Copy, Clone)]
pub enum Mapper {
    NoRom,
    LoRom,
    HiRom,
}

impl Mapper {
    pub fn map_to_file(&self, addr: usize) -> Option<usize> {
        match self {
            Mapper::NoRom => Some(addr),
            Mapper::LoRom => {
                if (addr&0xFE0000)==0x7E0000        //wram
                || (addr&0x408000)==0x000000        //hardware regs, ram mirrors, other strange junk
                || (addr&0x708000)==0x700000
                {
                    //sram (low parts of banks 70-7D)
                    None
                } else {
                    Some((addr & 0x7F0000) >> 1 | (addr & 0x7FFF))
                }
            }
            Mapper::HiRom => {
                if (addr&0xFE0000)==0x7E0000       //wram
                || (addr&0x408000)==0x000000
                {
                    //hardware regs, ram mirrors, other strange junk
                    None
                } else {
                    Some(addr & 0x3FFFFF)
                }
            }
        }
    }

    pub fn map_to_addr(&self, offset: usize) -> usize {
        match self {
            Mapper::NoRom => offset,
            Mapper::LoRom => {
                let in_bank = offset & 0x7FFF;
                let bank = offset >> 15;
                (bank << 16) + in_bank + 0x8000
            }
            Mapper::HiRom => offset | 0xC00000,
        }
    }
}

#[derive(Clone)]
pub struct Rom {
    buf:     Vec<u8>,
    mapper:  Mapper,
    symbols: HashMap<String, u32>,
}

impl Rom {
    pub fn new(buf: Vec<u8>) -> Self {
        Self { buf, mapper: Mapper::LoRom, symbols: HashMap::new() }
    }

    pub fn load_symbols(&mut self, data: &str) {
        for i in data.lines() {
            let i = if let Some(comment) = i.find(';') { &i[..comment] } else { i }.trim();
            if i.is_empty() {
                continue;
            }
            if let Some(v) = i.find(' ') {
                match u32::from_str_radix(&i[..v], 16) {
                    Ok(addr) => {
                        self.symbols.insert(i[v + 1..].to_string(), addr);
                    }
                    Err(_e) => {}
                }
            }
        }
    }

    pub fn resolve(&self, symbol: &str) -> Option<u32> {
        self.symbols.get(symbol).copied()
    }

    pub fn read(&self, addr: u32) -> Option<u8> {
        self.mapper.map_to_file(addr as _).and_then(|c| self.buf.get(c).copied())
    }

    pub fn read_u16(&self, addr: u32) -> Option<u16> {
        Some(u16::from_le_bytes([self.read(addr + 0)?, self.read(addr + 1)?]))
    }

    pub fn read_u32(&self, addr: u32) -> Option<u32> {
        Some(u32::from_le_bytes([self.read(addr + 0)?, self.read(addr + 1)?, self.read(addr + 2)?, 0]))
    }

    pub fn resize(&mut self, new_size: usize) {
        self.buf.resize(new_size, 0);
    }

    pub fn mapper(&self) -> Mapper {
        self.mapper
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    pub fn checksum(&self) -> u16 {
        // TODO: npo2 roms
        self.buf.iter().map(|c| *c as u16).sum()
    }
}
