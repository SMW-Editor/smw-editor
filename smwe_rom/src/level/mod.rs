pub mod background;
pub mod headers;
pub mod object_layer;
pub mod secondary_entrance;
pub mod sprite_layer;

use std::convert::TryFrom;

use nom::{
    bytes::complete::take,
    combinator::map,
    number::complete::{le_u16, le_u24},
    sequence::preceded,
};

pub use self::{
    background::{BackgroundData, BackgroundTileID},
    headers::{PrimaryHeader, SecondaryHeader, SpriteHeader, PRIMARY_HEADER_SIZE, SPRITE_HEADER_SIZE},
    object_layer::ObjectLayer,
    sprite_layer::SpriteLayer,
};
use crate::{
    addr::{AddrPc, AddrSnes},
    error::{LevelParseError, ParseErr},
};

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
    pub fn parse(rom_data: &[u8], level_num: usize) -> Result<Self, LevelParseError> {
        let (primary_header, layer1) = Self::parse_ph_and_l1(rom_data, level_num)?;
        let layer2 = Self::parse_l2(rom_data, level_num)?;
        let (sprite_header, sprite_layer) = Self::parse_sh_and_sl(rom_data, level_num)?;
        let secondary_header =
            SecondaryHeader::read_from_rom(rom_data, level_num).map_err(LevelParseError::SecondaryHeaderRead)?;

        Ok(Level { primary_header, secondary_header, sprite_header, layer1, layer2, sprite_layer })
    }

    fn parse_ph_and_l1(rom_data: &[u8], level_num: usize) -> Result<(PrimaryHeader, ObjectLayer), LevelParseError> {
        const LAYER1_DATA: AddrSnes = AddrSnes(0x05E000);

        let l1_ptr_addr: usize = AddrPc::try_from(LAYER1_DATA + (3 * level_num))
            .map_err(|_| LevelParseError::Layer1AddressConversion)?
            .into();

        let mut read_ph_addr = preceded(take(l1_ptr_addr), map(le_u24, AddrSnes::from));
        let (_, ph_addr) = read_ph_addr(rom_data).map_err(|_: ParseErr| LevelParseError::Layer1AddressRead)?;

        let ph_addr: usize =
            AddrPc::try_from(ph_addr).map_err(|_| LevelParseError::PrimaryHeaderAddressConversion)?.into();

        let mut read_ph = preceded(take(ph_addr), take(PRIMARY_HEADER_SIZE));
        let (layer1, primary_header) = read_ph(rom_data).map_err(|_: ParseErr| LevelParseError::Layer1Isolate)?;

        let (_, primary_header) =
            PrimaryHeader::read_from(primary_header).map_err(|_| LevelParseError::PrimaryHeaderRead)?;
        let (_, layer1) = ObjectLayer::parse(layer1).map_err(|_| LevelParseError::Layer1Read)?;
        Ok((primary_header, layer1))
    }

    fn parse_l2(rom_data: &[u8], level_num: usize) -> Result<Layer2Data, LevelParseError> {
        const LAYER2_DATA: AddrSnes = AddrSnes(0x05E600);

        let l2_ptr_addr: usize = AddrPc::try_from(LAYER2_DATA + (3 * level_num))
            .map_err(|_| LevelParseError::Layer2PtrAddressConversion)?
            .into();

        let mut read_l2_addr = preceded(take(l2_ptr_addr), map(le_u24, AddrSnes::from));
        let (_, l2_ptr) = read_l2_addr(rom_data).map_err(|_: ParseErr| LevelParseError::Layer2AddressRead)?;

        let isolate_l2 = |addr| {
            let addr: usize = AddrPc::try_from(addr).map_err(|_| LevelParseError::Layer2AddressConversion)?.into();
            take(addr + PRIMARY_HEADER_SIZE)(rom_data).map_err(|_: ParseErr| LevelParseError::Layer2Isolate)
        };

        if (l2_ptr.0 >> 16) == 0xFF {
            let (layer2, _) = isolate_l2((l2_ptr & 0xFFFF) | 0x0C0000)?;
            let background = BackgroundData::read_from(layer2).map_err(LevelParseError::Layer2BackgroundRead)?;
            Ok(Layer2Data::Background(background))
        } else {
            let (layer2, _) = isolate_l2(l2_ptr)?;
            let (_, objects) = ObjectLayer::parse(layer2).map_err(|_| LevelParseError::Layer2Read)?;
            Ok(Layer2Data::Objects(objects))
        }
    }

    fn parse_sh_and_sl(rom_data: &[u8], level_num: usize) -> Result<(SpriteHeader, SpriteLayer), LevelParseError> {
        const SPRITE_DATA: AddrSnes = AddrSnes(0x05EC00);

        let sprite_ptr_addr: usize = AddrPc::try_from(SPRITE_DATA + (2 * level_num))
            .map_err(|_| LevelParseError::SpritePtrAddressConversion)?
            .into();

        let mut read_sh_addr = preceded(take(sprite_ptr_addr), le_u16);
        let (_, sh_addr) = read_sh_addr(rom_data).map_err(|_: ParseErr| LevelParseError::SpriteAddressRead)?;

        let sh_addr = AddrSnes((sh_addr as u32 | 0x07_0000) as usize);
        let sh_addr: usize = AddrPc::try_from(sh_addr).map_err(|_| LevelParseError::SpriteAddressConversion)?.into();

        let mut read_sh = preceded(take(sh_addr), take(PRIMARY_HEADER_SIZE));
        let (sprite_layer, sh) = read_sh(rom_data).map_err(|_: ParseErr| LevelParseError::SpriteIsolate)?;

        let (_, sprite_header) = SpriteHeader::read_from(sh).map_err(|_| LevelParseError::SpriteHeaderRead)?;
        let (_, sprite_layer) = SpriteLayer::parse(sprite_layer).map_err(|_| LevelParseError::SpriteRead)?;
        Ok((sprite_header, sprite_layer))
    }
}
