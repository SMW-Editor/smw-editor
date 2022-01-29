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
            .with_error_mapper(LevelParseError::Layer1AddressRead)
            .slice_lorom(l1_ptr_slice)?
            .parse(map(le_u24, AddrSnes::from))?;

        let ph_slice = SnesSlice::new(ph_addr, PRIMARY_HEADER_SIZE);
        let primary_header =
            rom.with_error_mapper(LevelParseError::PrimaryHeaderRead).slice_lorom(ph_slice)?.as_bytes()?;
        let primary_header = PrimaryHeader::new(primary_header);

        let layer1_slice = ph_slice.skip_forward(1).infinite();
        let layer1 =
            rom.with_error_mapper(LevelParseError::Layer1Read).slice_lorom(layer1_slice)?.parse(ObjectLayer::parse)?;

        Ok((primary_header, layer1))
    }

    fn parse_l2(rom: &Rom, level_num: usize) -> Result<Layer2Data, LevelParseError> {
        const LAYER2_DATA: AddrSnes = AddrSnes(0x05E600);

        let l2_addr_slice = SnesSlice::new(LAYER2_DATA + (3 * level_num), 3);
        let l2_ptr = rom
            .with_error_mapper(LevelParseError::Layer2AddressRead)
            .slice_lorom(l2_addr_slice)?
            .parse(map(le_u24, AddrSnes::from))?;

        if l2_ptr.bank() == 0xFF {
            let slice = SnesSlice::new(l2_ptr.with_bank(0x0C) + PRIMARY_HEADER_SIZE, usize::MAX);
            let layer2 = rom.with_error_mapper(LevelParseError::Layer2Isolate).slice_lorom(slice)?.as_bytes()?;
            let background = BackgroundData::read_from(layer2).map_err(LevelParseError::Layer2BackgroundRead)?;
            Ok(Layer2Data::Background(background))
        } else {
            let slice = SnesSlice::new(l2_ptr + PRIMARY_HEADER_SIZE, usize::MAX);
            let objects =
                rom.with_error_mapper(LevelParseError::Layer2Read).slice_lorom(slice)?.parse(ObjectLayer::parse)?;
            Ok(Layer2Data::Objects(objects))
        }
    }

    fn parse_sh_and_sl(rom: &Rom, level_num: usize) -> Result<(SpriteHeader, SpriteLayer), LevelParseError> {
        const SPRITE_DATA: AddrSnes = AddrSnes(0x05EC00);

        let sprite_ptr_slice = SnesSlice::new(SPRITE_DATA + (2 * level_num), 2);
        let sh_addr =
            rom.with_error_mapper(LevelParseError::SpriteAddressRead).slice_lorom(sprite_ptr_slice)?.parse(le_u16)?;
        let sh_addr = AddrSnes(sh_addr as usize).with_bank(0x07);

        let sh_slice = SnesSlice::new(sh_addr, SPRITE_HEADER_SIZE);
        let sprite_header = rom
            .with_error_mapper(LevelParseError::SpriteHeaderRead)
            .slice_lorom(sh_slice)?
            .parse(SpriteHeader::read_from)?;

        let sl_slice = SnesSlice::new(sh_addr + 1, usize::MAX);
        let sprite_layer =
            rom.with_error_mapper(LevelParseError::SpriteRead).slice_lorom(sl_slice)?.parse(SpriteLayer::parse)?;

        Ok((sprite_header, sprite_layer))
    }
}
