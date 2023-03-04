use nom::{
    combinator::map,
    multi::count,
    number::complete::{le_u16, le_u24},
};
use thiserror::Error;

pub use self::{
    background::{BackgroundData, BackgroundTileID},
    headers::{PrimaryHeader, SecondaryHeader, SpriteHeader, PRIMARY_HEADER_SIZE, SPRITE_HEADER_SIZE},
    object_layer::ObjectLayer,
    sprite_layer::SpriteLayer,
};
use crate::{
    compression::DecompressionError,
    disassembler::binary_block::{DataBlock, DataKind},
    snes_utils::{
        addr::{AddrInner, AddrSnes},
        rom_slice::SnesSlice,
    },
    RomDisassembly,
    RomError,
};

pub mod background;
pub mod headers;
pub mod object_layer;
pub mod secondary_entrance;
pub mod sprite_layer;

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum LevelParseError {
    #[error("Reading address of Layer1:\n- {0}")]
    Layer1AddressRead(RomError),
    #[error("Reading address of Layer2:\n- {0}")]
    Layer2AddressRead(RomError),
    #[error("Reading address of Sprite data:\n- {0}")]
    SpriteAddressRead(RomError),

    #[error("Isolating Layer2 data:\n- {0}")]
    Layer2Isolate(RomError),

    #[error("Reading Primary Header:\n- {0}")]
    PrimaryHeaderRead(RomError),
    #[error("Reading Secondary Header:\n- {0}")]
    SecondaryHeaderRead(RomError),
    #[error("Reading Sprite Header:\n- {0}")]
    SpriteHeaderRead(RomError),

    #[error("Reading Layer1 object data:\n- {0}")]
    Layer1Read(RomError),
    #[error("Parsing Layer2 object data:\n- {0}")]
    Layer2Read(RomError),
    #[error("Reading Layer2 background:\n- {0}")]
    Layer2BackgroundRead(DecompressionError),
    #[error("Reading Sprite data:\n- {0}")]
    SpriteRead(RomError),
}

// -------------------------------------------------------------------------------------------------

pub const LEVEL_COUNT: usize = 0x200;

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub enum Layer2Data {
    Background(BackgroundData),
    Objects(ObjectLayer),
}

#[derive(Clone)]
pub struct Level {
    pub primary_header:   PrimaryHeader,
    pub secondary_header: SecondaryHeader,
    pub sprite_header:    SpriteHeader,
    pub layer1:           ObjectLayer,
    pub layer2:           Layer2Data,
    pub sprite_layer:     SpriteLayer,
}

// -------------------------------------------------------------------------------------------------

impl Level {
    pub fn parse(disasm: &mut RomDisassembly, level_num: u32) -> Result<Self, LevelParseError> {
        let (primary_header, layer1) = Self::parse_ph_and_l1(disasm, level_num)?;
        let layer2 = Self::parse_l2(disasm, level_num)?;
        let (sprite_header, sprite_layer) = Self::parse_sh_and_sl(disasm, level_num)?;
        let secondary_header =
            SecondaryHeader::read_from_rom(disasm, level_num).map_err(LevelParseError::SecondaryHeaderRead)?;

        Ok(Level { primary_header, secondary_header, sprite_header, layer1, layer2, sprite_layer })
    }

    fn parse_ph_and_l1(
        disasm: &mut RomDisassembly, level_num: u32,
    ) -> Result<(PrimaryHeader, ObjectLayer), LevelParseError> {
        let l1_ptr_block =
            DataBlock { slice: SnesSlice::new(AddrSnes(0x05E000), 0x200 * 3), kind: DataKind::LevelPointersLayer1 };
        let ph_addr = disasm
            .rom_slice_at_block(l1_ptr_block, LevelParseError::Layer1AddressRead)?
            .parse(count(map(le_u24, AddrSnes), 0x200))?[level_num as usize];

        let ph_block =
            DataBlock { slice: SnesSlice::new(ph_addr, PRIMARY_HEADER_SIZE), kind: DataKind::LevelHeaderPrimary };
        let primary_header = {
            let bytes = disasm.rom_slice_at_block(ph_block, LevelParseError::PrimaryHeaderRead)?.as_bytes()?;
            PrimaryHeader::new(bytes)
        };

        let layer1 = disasm.parse_and_mark_data(
            ph_addr + PRIMARY_HEADER_SIZE as u32,
            DataKind::LevelLayer1Objects,
            LevelParseError::Layer1Read,
            |rom_view| rom_view.parse(ObjectLayer::parse),
        )?;

        Ok((primary_header, layer1))
    }

    fn parse_l2(disasm: &mut RomDisassembly, level_num: u32) -> Result<Layer2Data, LevelParseError> {
        const LAYER2_DATA: AddrSnes = AddrSnes(0x05E600);

        let l2_addr_block =
            DataBlock { slice: SnesSlice::new(LAYER2_DATA + (3 * level_num), 3), kind: DataKind::LevelPointersLayer2 };
        let l2_ptr = disasm
            .rom_slice_at_block(l2_addr_block, LevelParseError::Layer2AddressRead)?
            .parse(map(le_u24, AddrSnes))?;

        if l2_ptr.bank() == 0xFF {
            let background = disasm.parse_and_mark_data(
                l2_ptr.with_bank(0x0C),
                DataKind::LevelLayer2Background,
                LevelParseError::Layer2Isolate,
                |rom_view| {
                    let bytes = rom_view.as_bytes()?;
                    BackgroundData::read_from(bytes).map_err(LevelParseError::Layer2BackgroundRead)
                },
            )?;
            Ok(Layer2Data::Background(background))
        } else {
            let objects = disasm.parse_and_mark_data(
                l2_ptr + PRIMARY_HEADER_SIZE as u32,
                DataKind::LevelLayer2Objects,
                LevelParseError::Layer2Read,
                |rom_view| rom_view.parse(ObjectLayer::parse),
            )?;
            Ok(Layer2Data::Objects(objects))
        }
    }

    fn parse_sh_and_sl(
        disasm: &mut RomDisassembly, level_num: u32,
    ) -> Result<(SpriteHeader, SpriteLayer), LevelParseError> {
        const SPRITE_DATA: AddrSnes = AddrSnes(0x05EC00);

        let sprite_ptr_block =
            DataBlock { slice: SnesSlice::new(SPRITE_DATA + (2 * level_num), 2), kind: DataKind::LevelPointersSprite };
        let sh_addr = disasm.rom_slice_at_block(sprite_ptr_block, LevelParseError::SpriteAddressRead)?.parse(le_u16)?;
        let sh_addr = AddrSnes(sh_addr as AddrInner).with_bank(0x07);

        let sh_block =
            DataBlock { slice: SnesSlice::new(sh_addr, SPRITE_HEADER_SIZE), kind: DataKind::LevelHeaderSprites };
        let sprite_header =
            disasm.rom_slice_at_block(sh_block, LevelParseError::SpriteHeaderRead)?.parse(SpriteHeader::read_from)?;

        let sprite_layer = disasm.parse_and_mark_data(
            sh_addr + 1,
            DataKind::LevelSpriteLayer,
            LevelParseError::SpriteRead,
            |rom_view| rom_view.parse(SpriteLayer::parse),
        )?;

        Ok((sprite_header, sprite_layer))
    }
}
