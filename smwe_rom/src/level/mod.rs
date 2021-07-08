pub mod background;
pub mod headers;
pub mod object_layer;
pub mod secondary_entrance;
pub mod sprite_layer;

use std::convert::TryFrom;

use nom::{
    count,
    map,
    number::complete::{le_u16, le_u24},
    preceded,
    take,
    IResult,
};

pub use self::{
    background::{BackgroundData, BackgroundTileID},
    headers::{PrimaryHeader, SecondaryHeader, SpriteHeader, PRIMARY_HEADER_SIZE, SPRITE_HEADER_SIZE},
    object_layer::ObjectLayer,
    sprite_layer::SpriteLayer,
};
use crate::addr::{AddrPc, AddrSnes};

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
    pub fn parse(rom_data: &[u8], level_num: usize) -> IResult<&[u8], Self> {
        pub const LAYER1_DATA: AddrSnes = AddrSnes(0x05E000);
        pub const LAYER2_DATA: AddrSnes = AddrSnes(0x05E600);
        pub const SPRITE_DATA: AddrSnes = AddrSnes(0x05EC00);

        log::info!("Parsing level {:X}", level_num);

        log::info!("Isolating raw data of Layer1 and Primary Header");
        let (layer1, ph) = {
            let l1_ptr_addr: usize = AddrPc::try_from(LAYER1_DATA + (3 * level_num)).unwrap().into();
            let (_, ph_addr) = preceded!(rom_data, take!(l1_ptr_addr), le_u24)?;
            let ph_addr = AddrSnes(ph_addr as usize);
            let ph_addr: usize = AddrPc::try_from(ph_addr).unwrap().into();
            preceded!(rom_data, take!(ph_addr), take!(PRIMARY_HEADER_SIZE))?
        };

        log::info!("Isolating raw data of Layer2 and Secondary Header");
        let (layer2, is_l2_background) = {
            let l2_ptr_table_addr: usize = AddrPc::try_from(LAYER2_DATA).unwrap().into();
            let (_, l2_ptr_table) =
                preceded!(rom_data, take!(l2_ptr_table_addr), count!(map!(le_u24, AddrSnes::from), 3 * LEVEL_COUNT))?;
            let l2_ptr = l2_ptr_table[level_num];

            let isolate_l2 = |addr| {
                let addr: usize = AddrPc::try_from(addr).unwrap().into();
                take!(rom_data, addr + PRIMARY_HEADER_SIZE)
            };

            if (l2_ptr.0 >> 16) == 0xFF {
                (isolate_l2((l2_ptr & 0xFFFF) | 0x0C0000)?.0, true)
            } else {
                (isolate_l2(l2_ptr)?.0, false)
            }
        };

        log::info!("Isolating raw data of Sprite Layer and Sprite Header");
        let (sprite_layer, sh) = {
            let sp_ptr_table_addr: usize = AddrPc::try_from(SPRITE_DATA).unwrap().into();
            let (_, sh_addr) = preceded!(rom_data, take!(sp_ptr_table_addr), le_u16)?;
            let sh_addr = AddrSnes((sh_addr as u32 | 0x07_0000) as usize);
            let sh_addr: usize = AddrPc::try_from(sh_addr).unwrap().into();
            preceded!(rom_data, take!(sh_addr), take!(PRIMARY_HEADER_SIZE))?
        };

        log::info!("Parsing Primary Header");
        let (_, primary_header) = PrimaryHeader::read_from(ph)?;
        log::info!("Parsing Secondary Header");
        let (_, secondary_header) = SecondaryHeader::read_from_rom(rom_data, level_num)?;
        log::info!("Parsing Sprite Header");
        let (_, sprite_header) = SpriteHeader::read_from(sh)?;

        log::info!("Parsing Layer1");
        let (_, layer1) = ObjectLayer::parse(layer1)?;

        let layer2 = if is_l2_background {
            log::info!("Parsing Layer2: background");
            let background = BackgroundData::read_from(layer2).unwrap(); // TODO: replace with error
            Layer2Data::Background(background)
        } else {
            log::info!("Parsing Layer2: objects");
            let (_, objects) = ObjectLayer::parse(layer2)?;
            Layer2Data::Objects(objects)
        };

        log::info!("Parsing Sprite Layer");
        let (_, sprite_layer) = SpriteLayer::parse(sprite_layer)?;

        Ok((rom_data, Level { primary_header, secondary_header, sprite_header, layer1, layer2, sprite_layer }))
    }
}
