#![allow(clippy::identity_op)]

pub mod compression;
pub mod disassembler;
pub mod graphics;
pub mod internal_header;
pub mod level;
pub mod objects;
pub mod snes_utils;

use std::{fs, path::Path};

use crate::{
    disassembler::{
        binary_block::{DataBlock, DataKind},
        RomDisassembly,
    },
    graphics::Gfx,
    internal_header::{InternalHeaderParseError, RegionCode, RomInternalHeader},
    level::{
        secondary_entrance::{SecondaryEntrance, SECONDARY_ENTRANCE_TABLE},
        Level,
        LEVEL_COUNT,
    },
    objects::tilesets::Tilesets,
    snes_utils::{
        addr::AddrSnes,
        rom::{Rom, RomError},
        rom_slice::SnesSlice,
    },
};

// -------------------------------------------------------------------------------------------------

pub struct SmwRom {
    pub disassembly:         RomDisassembly,
    pub internal_header:     RomInternalHeader,
    pub levels:              Vec<Level>,
    pub secondary_entrances: Vec<SecondaryEntrance>,
    pub gfx:                 Gfx,
    pub map16_tilesets:      Tilesets,
}

// -------------------------------------------------------------------------------------------------

impl SmwRom {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        log::info!("Reading ROM from file: {}", path.as_ref().display());

        let bytes = fs::read(path)?;
        let rom = Rom::new(bytes)?;
        let smw_rom = Self::from_rom(rom);

        if smw_rom.is_ok() {
            log::info!("Success parsing ROM");
        }

        smw_rom
    }

    pub fn from_rom(rom: Rom) -> anyhow::Result<Self> {
        log::info!("Parsing internal ROM header");
        let internal_header = RomInternalHeader::parse(&rom)?;

        log::info!("Creating disassembly map");
        let mut disassembly = RomDisassembly::new(rom, &internal_header);

        // Mark IRH
        disassembly.rom_slice_at_block(
            DataBlock {
                slice: SnesSlice::new(AddrSnes(0x00FFC0), internal_header::sizes::INTERNAL_HEADER),
                kind:  DataKind::InternalRomHeader,
            },
            |_| InternalHeaderParseError::NotFound,
        )?;

        log::info!("Parsing level data");
        let levels = Self::parse_levels(&mut disassembly)?;

        log::info!("Parsing secondary entrances");
        let secondary_entrances = Self::parse_secondary_entrances(&mut disassembly)?;

        log::info!("Parsing GFX files");
        let gfx = Gfx::parse(&mut disassembly, &levels, &internal_header)?;

        log::info!("Parsing Map16 tilesets");
        let map16_tilesets = Tilesets::parse(&mut disassembly)?;

        Ok(Self { disassembly, internal_header, levels, secondary_entrances, gfx, map16_tilesets })
    }

    fn parse_levels(disasm: &mut RomDisassembly) -> anyhow::Result<Vec<Level>> {
        let mut levels = Vec::with_capacity(LEVEL_COUNT);
        for level_num in 0..LEVEL_COUNT as u32 {
            let level = Level::parse(disasm, level_num)?;
            levels.push(level);
        }
        Ok(levels)
    }

    fn parse_secondary_entrances(disasm: &mut RomDisassembly) -> anyhow::Result<Vec<SecondaryEntrance>> {
        let mut secondary_entrances = Vec::with_capacity(SECONDARY_ENTRANCE_TABLE.size);
        for entrance_id in 0..SECONDARY_ENTRANCE_TABLE.size {
            let entrance = SecondaryEntrance::read_from_rom(disasm, entrance_id)?;
            secondary_entrances.push(entrance);
        }
        Ok(secondary_entrances)
    }
}
