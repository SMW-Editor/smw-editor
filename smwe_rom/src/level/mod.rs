use nom::{
    combinator::map,
    number::complete::{le_u16, le_u24},
};

pub use self::{
    background::{BackgroundData, BackgroundTileID},
    headers::{PrimaryHeader, SecondaryHeader, SpriteHeader, PRIMARY_HEADER_SIZE, SPRITE_HEADER_SIZE},
    object_layer::ObjectLayer,
    sprite_layer::SpriteLayer,
};
use crate::{
    error::LevelParseError,
    snes_utils::{addr::AddrSnes, rom::Rom, rom_slice::SnesSlice},
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
    pub fn parse(rom: &Rom, level_num: usize) -> Result<Self, LevelParseError> {
        let (primary_header, layer1) = Self::parse_ph_and_l1(rom, level_num)?;
        let layer2 = Self::parse_l2(rom, level_num)?;
        let (sprite_header, sprite_layer) = Self::parse_sh_and_sl(rom, level_num)?;
        let secondary_header =
            SecondaryHeader::read_from_rom(rom, level_num).map_err(LevelParseError::SecondaryHeaderRead)?;

        Ok(Level { primary_header, secondary_header, sprite_header, layer1, layer2, sprite_layer })
    }

    fn parse_ph_and_l1(rom: &Rom, level_num: usize) -> Result<(PrimaryHeader, ObjectLayer), LevelParseError> {
        const LAYER1_DATA: AddrSnes = AddrSnes(0x05E000);

        let l1_ptr_slice = SnesSlice::new(LAYER1_DATA + (3 * level_num), 3);
        let ph_addr = rom
            .parse_slice_lorom(l1_ptr_slice, map(le_u24, AddrSnes::from))
            .map_err(LevelParseError::Layer1AddressRead)?;

        let ph_slice = SnesSlice::new(ph_addr, PRIMARY_HEADER_SIZE);
        let primary_header = rom.slice_lorom(ph_slice).map_err(LevelParseError::PrimaryHeaderRead)?;
        let primary_header = PrimaryHeader::new(primary_header);

        let layer1_slice = ph_slice.skip_forward(1).infinite();
        let layer1 = rom.parse_slice_lorom(layer1_slice, ObjectLayer::parse).map_err(LevelParseError::Layer1Read)?;

        Ok((primary_header, layer1))
    }

    fn parse_l2(rom: &Rom, level_num: usize) -> Result<Layer2Data, LevelParseError> {
        const LAYER2_DATA: AddrSnes = AddrSnes(0x05E600);

        let l2_addr_slice = SnesSlice::new(LAYER2_DATA + (3 * level_num), 3);
        let l2_ptr = rom
            .parse_slice_lorom(l2_addr_slice, map(le_u24, AddrSnes::from))
            .map_err(LevelParseError::Layer2AddressRead)?;

        if (l2_ptr.0 >> 16) == 0xFF {
            let slice = SnesSlice::new(((l2_ptr & 0xFFFF) | 0x0C0000) + PRIMARY_HEADER_SIZE, 0);
            let layer2 = rom.slice_lorom(slice).map_err(LevelParseError::Layer2Isolate)?;
            let background = BackgroundData::read_from(layer2).map_err(LevelParseError::Layer2BackgroundRead)?;
            Ok(Layer2Data::Background(background))
        } else {
            let slice = SnesSlice::new(l2_ptr + PRIMARY_HEADER_SIZE, 0);
            let objects = rom.parse_slice_lorom(slice, ObjectLayer::parse).map_err(LevelParseError::Layer2Read)?;
            Ok(Layer2Data::Objects(objects))
        }
    }

    fn parse_sh_and_sl(rom: &Rom, level_num: usize) -> Result<(SpriteHeader, SpriteLayer), LevelParseError> {
        const SPRITE_DATA: AddrSnes = AddrSnes(0x05EC00);

        let sprite_ptr_slice = SnesSlice::new(SPRITE_DATA + (2 * level_num), 2);
        let sh_addr = rom.parse_slice_lorom(sprite_ptr_slice, le_u16).map_err(LevelParseError::SpriteAddressRead)?;
        let sh_addr = AddrSnes((sh_addr as u32 | 0x07_0000) as usize);

        let sh_slice = SnesSlice::new(sh_addr, SPRITE_HEADER_SIZE);
        let sprite_header =
            rom.parse_slice_lorom(sh_slice, SpriteHeader::read_from).map_err(LevelParseError::SpriteHeaderRead)?;

        let sl_slice = SnesSlice::new(sh_addr + 1, 0);
        let sprite_layer = rom.parse_slice_lorom(sl_slice, SpriteLayer::parse).map_err(LevelParseError::SpriteRead)?;

        Ok((sprite_header, sprite_layer))
    }
}
