pub mod object_layer;
pub mod pointer_tables;
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
pub struct Level {
    pub primary_header: PrimaryHeader,
    //pub secondary_header: secondary_header,
    pub layer1:         ObjectLayer,
    //pub layer2: ObjectLayer,
}

impl Level {
    pub fn from_rom_data(rom_data: &[u8], level_num: usize) -> IResult<&[u8], Self> {
        let (layer1, ph) = {
            let l1_ptr_addr = AddrPc::try_from(pointer_tables::LAYER1_DATA + (3 * level_num)).unwrap();
            let (_, ph_addr) = preceded!(rom_data, take!(l1_ptr_addr.0), le_u24)?;
            let ph_addr: usize = AddrPc::try_from(AddrSnes(ph_addr as usize)).unwrap().into();
            preceded!(rom_data, take!(ph_addr), take!(PRIMARY_HEADER_SIZE))?
        };

        let (_, primary_header) = PrimaryHeader::parse(ph)?;
        //let (_, secondary_header) = SecondaryHeader::parse(ph)?;
        let (_, layer1) = ObjectLayer::parse(layer1)?;
        //let (_, layer2) = ObjectLayer::parse(layer2)?;

        Ok((rom_data, Level {
            primary_header,
            //secondary_header,
            layer1,
            //layer2,
        }))
    }
}
