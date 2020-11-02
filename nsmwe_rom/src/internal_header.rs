use crate::addr::AddressPC;

pub use address_spaces::*;

use crate::{
    get_byte_at,
    get_word_at,
    get_slice_at,
};

use num_enum::TryFromPrimitive;
use std::convert::{
    TryFrom,
    TryInto,
};

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum MapMode {
    LoRom       = 0b100000,
    HiRom       = 0b100001,
    ExHiRom     = 0b100010,
    ExLoRom     = 0b100100,
    FastLoRom   = 0b110000,
    FastHiRom   = 0b110001,
    FastExHiRom = 0b110010,
    FastExLoRom = 0b110100,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum RomType {
    Rom          = 0x00,
    RomRam       = 0x01,
    RomRamSram   = 0x02,

    RomDsp     = 0x03,
    RomSuperFx = 0x13,
    RomObc1    = 0x23,
    RomSa1     = 0x33,
    RomSdd1    = 0x43,
    RomOther   = 0xE3,
    RomCustom  = 0xF3,

    RomDspRam     = 0x04,
    RomSuperFxRam = 0x14,
    RomObc1Ram    = 0x24,
    RomSa1Ram     = 0x34,
    RomSdd1Ram    = 0x44,
    RomOtherRam   = 0xE4,
    RomCustomRam  = 0xF4,

    RomDspRamSram     = 0x05,
    RomSuperFxRamSram = 0x15,
    RomObc1RamSram    = 0x25,
    RomSa1RamSram     = 0x35,
    RomSdd1RamSram    = 0x45,
    RomOtherRamSram   = 0xE5,
    RomCustomRamSram  = 0xF5,

    RomDspSram     = 0x06,
    RomSuperFxSram = 0x16,
    RomObc1Sram    = 0x26,
    RomSa1Sram     = 0x36,
    RomSdd1Sram    = 0x46,
    RomOtherSram   = 0xE6,
    RomCustomSram  = 0xF6,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum DestinationCode {
    Japan        = 0x00,
    NorthAmerica = 0x01,
    Europe       = 0x02,
    Sweden       = 0x03,
    Finland      = 0x04,
    Denmark      = 0x05,
    France       = 0x06,
    Netherlands  = 0x07,
    Spain        = 0x08,
    Germany      = 0x09,
    Italy        = 0x0A,
    China        = 0x0B,
    Indonesia    = 0x0C,
    Korea        = 0x0D,
    Global       = 0x0E,
    Canada       = 0x0F,
    Brazil       = 0x10,
    Australia    = 0x11,
    Other1       = 0x12,
    Other2       = 0x13,
    Other3       = 0x14,
}

pub struct InternalHeader {
    pub maker_code: [u8; 2],
    pub game_code: [u8; 4],
    pub expansion_ram_size: u8, // actual size = 2^expansion_ram_size
    pub special_version: u8,
    pub cartridge_type: u8,
    pub internal_rom_name: [u8; 21],
    pub map_mode: MapMode,
    pub rom_type: RomType,
    pub rom_size: u8,  // actual size = 2^rom_size
    pub sram_size: u8, // actual size = 2^sram_size
    pub destination_code: DestinationCode,
    pub version_number: u8,
}

pub mod address_spaces {
    use crate::addr::AddressSpace;
    pub const HEADER_LOROM: AddressSpace = 0x007FC0..=0x008000;
    pub const HEADER_HIROM: AddressSpace = 0x00FFC0..=0x010000;
}

pub mod offset {
    pub const MAKER_CODE:         u32 = 0x00;
    pub const GAME_CODE:          u32 = 0x02;
    pub const EXPANSION_RAM_SIZE: u32 = 0x0D;
    pub const SPECIAL_VERSION:    u32 = 0x0E;
    pub const CARTRIDGE_TYPE:     u32 = 0x0F;
    pub const INTERNAL_ROM_NAME:  u32 = 0x10;
    pub const MAP_MODE:           u32 = 0x25;
    pub const ROM_TYPE:           u32 = 0x26;
    pub const ROM_SIZE:           u32 = 0x27;
    pub const SRAM_SIZE:          u32 = 0x28;
    pub const DESTINATION_CODE:   u32 = 0x29;
    pub const VERSION_NUMBER:     u32 = 0x2B;
    pub const COMPLEMENT_CHECK:   u32 = 0x2C;
    pub const CHECKSUM:           u32 = 0x2E;
}

impl InternalHeader {
    pub fn from_rom_data(data: &[u8], smc_header_offset: AddressPC) -> Result<Self, String> {
        let begin = InternalHeader::find(data, smc_header_offset)?;

        let maker_code = get_slice_at(data, begin + offset::MAKER_CODE, 2)?
            .try_into()
            .unwrap();

        let game_code = get_slice_at(data, begin + offset::GAME_CODE, 4)?
            .try_into()
            .unwrap();

        let expansion_ram_size= get_byte_at(data, begin + offset::EXPANSION_RAM_SIZE)?;
        let special_version = get_byte_at(data, begin + offset::SPECIAL_VERSION)?;
        let cartridge_type = get_byte_at(data, begin + offset::CARTRIDGE_TYPE)?;

        let internal_rom_name = get_slice_at(data, begin + offset::INTERNAL_ROM_NAME, 21)?
            .try_into()
            .unwrap();

        let map_mode = get_byte_at(data, begin + offset::MAP_MODE)?;
        let map_mode = MapMode::try_from(map_mode)
            .or_else(|_| Err(String::from("Invalid map mode.")))?;

        let rom_type = get_byte_at(data, begin + offset::ROM_TYPE)?;
        let rom_type = RomType::try_from(rom_type)
            .or_else(|_| Err(String::from("Invalid ROM type.")))?;

        let rom_size = get_byte_at(data, begin + offset::ROM_SIZE)?;
        let sram_size = get_byte_at(data, begin + offset::SRAM_SIZE)?;

        let destination_code = get_byte_at(data, begin + offset::DESTINATION_CODE)?;
        let destination_code = DestinationCode::try_from(destination_code)
            .or_else(|_| Err(String::from("Invalid destination code.")))?;

        let version_number = get_byte_at(data, begin + offset::VERSION_NUMBER)?;

        Ok(InternalHeader {
            maker_code,
            game_code,
            expansion_ram_size,
            special_version,
            cartridge_type,
            internal_rom_name,
            map_mode,
            rom_type,
            rom_size,
            sram_size,
            destination_code,
            version_number,
        })
    }

    fn find(data: &[u8], smc_header_offset: u32) -> Result<AddressPC, String> {
        let lorom_checksum_idx = smc_header_offset + *HEADER_LOROM.start() + offset::CHECKSUM;
        let lorom_complmnt_idx = smc_header_offset + *HEADER_LOROM.start() + offset::COMPLEMENT_CHECK;
        let hirom_checksum_idx = smc_header_offset + *HEADER_HIROM.start() + offset::CHECKSUM;
        let hirom_complmnt_idx = smc_header_offset + *HEADER_HIROM.start() + offset::COMPLEMENT_CHECK;

        let lorom_checksum = get_word_at(data, lorom_checksum_idx)?;
        let lorom_complmnt = get_word_at(data, lorom_complmnt_idx)?;
        let hirom_checksum = get_word_at(data, hirom_checksum_idx)?;
        let hirom_complmnt = get_word_at(data, hirom_complmnt_idx)?;

        if (lorom_checksum ^ lorom_complmnt) == 0xFFFF {
            Ok(*HEADER_LOROM.start() + smc_header_offset)
        } else if (hirom_checksum ^ hirom_complmnt) == 0xFFFF {
            Ok(*HEADER_HIROM.start() + smc_header_offset)
        } else {
            Err(String::from("Could not locate internal header: both checksums are invalid."))
        }
    }
}