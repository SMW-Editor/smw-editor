use nom::{
    combinator::map,
    multi::count,
    number::complete::{le_u16, le_u24},
};

pub use self::{
    background::{BackgroundData, BackgroundTileID},
    headers::{PrimaryHeader, SecondaryHeader, SpriteHeader, PRIMARY_HEADER_SIZE, SPRITE_HEADER_SIZE},
    object_layer::ObjectLayer,
    sprite_layer::SpriteLayer,
};
use crate::{
    disassembler::binary_block::{DataBlock, DataKind},
    error::LevelParseError,
    snes_utils::{addr::AddrSnes, rom_slice::SnesSlice},
    RomDisassembly,
};

pub mod background;
pub mod headers;
pub mod object_layer;
pub mod secondary_entrance;
pub mod sprite_layer;

pub const LEVEL_COUNT: usize = 0x200;

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

impl Level {
    pub fn parse(disasm: &mut RomDisassembly, level_num: usize) -> Result<Self, LevelParseError> {
        let (primary_header, layer1) = Self::parse_ph_and_l1(disasm, level_num)?;
        let layer2 = Self::parse_l2(disasm, level_num)?;
        let (sprite_header, sprite_layer) = Self::parse_sh_and_sl(disasm, level_num)?;
        let secondary_header =
            SecondaryHeader::read_from_rom(disasm, level_num).map_err(LevelParseError::SecondaryHeaderRead)?;

        Ok(Level { primary_header, secondary_header, sprite_header, layer1, layer2, sprite_layer })
    }

    fn parse_ph_and_l1(
        disasm: &mut RomDisassembly, level_num: usize,
    ) -> Result<(PrimaryHeader, ObjectLayer), LevelParseError> {
        let l1_ptr_block =
            DataBlock { slice: SnesSlice::new(AddrSnes(0x05E000), 0x200 * 3), kind: DataKind::LevelPointersLayer1 };
        let ph_addr = disasm
            .rom_slice_at_block(l1_ptr_block, LevelParseError::Layer1AddressRead)?
            .parse(count(map(le_u24, AddrSnes::from), 0x200))?[level_num];

        let ph_block =
            DataBlock { slice: SnesSlice::new(ph_addr, PRIMARY_HEADER_SIZE), kind: DataKind::LevelHeaderPrimary };
        let primary_header = disasm.rom_slice_at_block(ph_block, LevelParseError::PrimaryHeaderRead)?.as_bytes()?;
        let primary_header = PrimaryHeader::new(primary_header);

        let mut layer1_block =
            DataBlock { slice: ph_block.slice.skip_forward(1).infinite(), kind: DataKind::LevelLayer1Objects };
        let (layer1, consumed_size) =
            disasm.rom_slice_at_block(layer1_block, LevelParseError::Layer1Read)?.parse(ObjectLayer::parse)?;
        layer1_block.slice.size = consumed_size;
        disasm.rom_slice_at_block(layer1_block, LevelParseError::Layer1Read)?;

        Ok((primary_header, layer1))
    }

    fn parse_l2(disasm: &mut RomDisassembly, level_num: usize) -> Result<Layer2Data, LevelParseError> {
        const LAYER2_DATA: AddrSnes = AddrSnes(0x05E600);

        let l2_addr_block =
            DataBlock { slice: SnesSlice::new(LAYER2_DATA + (3 * level_num), 3), kind: DataKind::LevelPointersLayer2 };
        let l2_ptr = disasm
            .rom_slice_at_block(l2_addr_block, LevelParseError::Layer2AddressRead)?
            .parse(map(le_u24, AddrSnes::from))?;

        if l2_ptr.bank() == 0xFF {
            let mut block = DataBlock {
                slice: SnesSlice::new(l2_ptr.with_bank(0x0C) + PRIMARY_HEADER_SIZE, usize::MAX),
                kind:  DataKind::LevelLayer2Background,
            };
            let layer2 = disasm.rom_slice_at_block(block, LevelParseError::Layer2Isolate)?.as_bytes()?;
            let (background, consumed_size) =
                BackgroundData::read_from(layer2).map_err(LevelParseError::Layer2BackgroundRead)?;
            block.slice.size = consumed_size;
            disasm.rom_slice_at_block(block, LevelParseError::Layer2Isolate)?;
            Ok(Layer2Data::Background(background))
        } else {
            let mut block = DataBlock {
                slice: SnesSlice::new(l2_ptr + PRIMARY_HEADER_SIZE, usize::MAX),
                kind:  DataKind::LevelLayer2Objects,
            };
            let (objects, consumed_size) =
                disasm.rom_slice_at_block(block, LevelParseError::Layer2Read)?.parse(ObjectLayer::parse)?;
            block.slice.size = consumed_size;
            disasm.rom_slice_at_block(block, LevelParseError::Layer2Read)?;
            Ok(Layer2Data::Objects(objects))
        }
    }

    fn parse_sh_and_sl(
        disasm: &mut RomDisassembly, level_num: usize,
    ) -> Result<(SpriteHeader, SpriteLayer), LevelParseError> {
        const SPRITE_DATA: AddrSnes = AddrSnes(0x05EC00);

        let sprite_ptr_block =
            DataBlock { slice: SnesSlice::new(SPRITE_DATA + (2 * level_num), 2), kind: DataKind::LevelPointersSprite };
        let sh_addr = disasm.rom_slice_at_block(sprite_ptr_block, LevelParseError::SpriteAddressRead)?.parse(le_u16)?;
        let sh_addr = AddrSnes(sh_addr as usize).with_bank(0x07);

        let sh_block =
            DataBlock { slice: SnesSlice::new(sh_addr, SPRITE_HEADER_SIZE), kind: DataKind::LevelHeaderSprites };
        let sprite_header =
            disasm.rom_slice_at_block(sh_block, LevelParseError::SpriteHeaderRead)?.parse(SpriteHeader::read_from)?;

        let mut sl_block =
            DataBlock { slice: SnesSlice::new(sh_addr + 1, usize::MAX), kind: DataKind::LevelSpriteLayer };
        let (sprite_layer, consumed_size) =
            disasm.rom_slice_at_block(sl_block, LevelParseError::SpriteRead)?.parse(SpriteLayer::parse)?;
        sl_block.slice.size = consumed_size;
        disasm.rom_slice_at_block(sl_block, LevelParseError::SpriteRead)?;

        Ok((sprite_header, sprite_layer))
    }
}
