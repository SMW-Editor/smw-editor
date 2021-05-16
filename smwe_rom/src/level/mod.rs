pub mod object_layer;
pub mod primary_header;
pub mod secondary_header;

use std::convert::TryFrom;

use nom::{number::complete::le_u24, preceded, take, IResult};

pub use self::{
    object_layer::ObjectLayer,
    primary_header::{PrimaryHeader, PRIMARY_HEADER_SIZE},
    secondary_header::SecondaryHeader,
};
use crate::addr::{AddrPc, AddrSnes};

pub const LEVEL_COUNT: usize = 0x200;

#[derive(Clone)]
pub enum Layer2Data {
    Background,
    Objects(ObjectLayer),
}

#[derive(Clone)]
pub struct Level {
    pub primary_header: PrimaryHeader,
    //pub secondary_header: SecondaryHeader,
    pub layer1:         ObjectLayer,
    pub layer2:         Layer2Data,
}

impl Level {
    pub fn parse(rom_data: &[u8], level_num: usize) -> IResult<&[u8], Self> {
        pub const LAYER1_DATA: AddrSnes = AddrSnes(0x05E000);
        pub const LAYER2_DATA: AddrSnes = AddrSnes(0x05E600);
        pub const _SPRITE_DATA: AddrSnes = AddrSnes(0x05EC00);

        let (layer1, ph) = {
            let l1_ptr_addr: usize = AddrPc::try_from(LAYER1_DATA + (3 * level_num)).unwrap().into();
            let (_, ph_addr) = preceded!(rom_data, take!(l1_ptr_addr), le_u24)?;
            let ph_addr = AddrSnes(ph_addr as usize);
            let ph_addr: usize = AddrPc::try_from(ph_addr).unwrap().into();
            preceded!(rom_data, take!(ph_addr), take!(PRIMARY_HEADER_SIZE))?
        };

        let (layer2, is_l2_background) = {
            let l2_ptr_addr: usize = AddrPc::try_from(LAYER2_DATA + (3 * level_num)).unwrap().into();
            let isolate_l2 = |addr| take!(rom_data, addr + PRIMARY_HEADER_SIZE);
            if (l2_ptr_addr >> 16) == 0xFF {
                (isolate_l2((l2_ptr_addr & 0xFFFF) | 0x0C0000)?.0, true)
            } else {
                (isolate_l2(l2_ptr_addr)?.0, false)
            }
        };

        let (_, primary_header) = PrimaryHeader::parse(ph)?;
        //let (_, secondary_header) = SecondaryHeader::parse(rom_data, level_num)?;
        let (_, layer1) = ObjectLayer::parse(layer1)?;
        let layer2 = if is_l2_background {
            Layer2Data::Background
        } else {
            let (_, objects) = ObjectLayer::parse(layer2)?;
            Layer2Data::Objects(objects)
        };

        Ok((rom_data, Level {
            primary_header,
            //secondary_header,
            layer1,
            layer2,
        }))
    }
}
