pub use self::address_spaces::*;

use crate::addr::AddressPc;

use nom::{
    do_parse,
    IResult,
    map,
    map_res,
    named,
    number::complete::{
        le_u8,
        le_u16,
    },
    sequence::tuple,
    take_str,
};

use num_enum::{
    IntoPrimitive,
    TryFromPrimitive,
};

use std::{
    clone::Clone,
    convert::{
        From,
        TryFrom,
    },
    fmt,
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

impl fmt::Display for MapMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MapMode::*;
        write!(f, "{}", match self {
            LoRom       => "LoROM",
            HiRom       => "HiROM",
            ExLoRom     => "ExLoROM",
            ExHiRom     => "ExHiROM",
            FastLoRom   => "Fast LoROM",
            FastHiRom   => "Fast HiROM",
            FastExLoRom => "Fast ExLoROM",
            FastExHiRom => "Fast ExHiROM",
        })
    }
}

#[derive(Copy, Clone, IntoPrimitive, TryFromPrimitive)]
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
    RomSrtc    = 0x53,
    RomOther   = 0xE3,
    RomCustom  = 0xF3,

    RomDspRam     = 0x04,
    RomSuperFxRam = 0x14,
    RomObc1Ram    = 0x24,
    RomSa1Ram     = 0x34,
    RomSdd1Ram    = 0x44,
    RomSRtcRam    = 0x54,
    RomOtherRam   = 0xE4,
    RomCustomRam  = 0xF4,

    RomDspRamSram     = 0x05,
    RomSuperFxRamSram = 0x15,
    RomObc1RamSram    = 0x25,
    RomSa1RamSram     = 0x35,
    RomSdd1RamSram    = 0x45,
    RomSRtcRamSram    = 0x55,
    RomOtherRamSram   = 0xE5,
    RomCustomRamSram  = 0xF5,

    RomDspSram     = 0x06,
    RomSuperFxSram = 0x16,
    RomObc1Sram    = 0x26,
    RomSa1Sram     = 0x36,
    RomSdd1Sram    = 0x46,
    RomSRtcSram    = 0x56,
    RomOtherSram   = 0xE6,
    RomCustomSram  = 0xF6,
}

impl fmt::Display for RomType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RomType::*;
        let self_as_byte: u8 = (*self).into();
        write!(f, "{}", match self {
            Rom => String::from("ROM"),
            RomRam => String::from("ROM + RAM"),
            RomRamSram => String::from("ROM + RAM + SRAM"),
            _ => format!("ROM + {}", {
                let coprocessor = match self_as_byte & 0xF0 {
                    0x00 => "DSP",
                    0x10 => "SuperFX",
                    0x20 => "OBC-1",
                    0x30 => "SA-1",
                    0x40 => "SDD-1",
                    0x50 => "S-RTC",
                    0xE0 => "Other expansion chip",
                    0xF0 => "Custom expansion chip",
                    _ => "Unknown expansion chip",
                };
                let memory = self_as_byte & 0xF;
                if memory == 0x3 {
                    coprocessor.to_string()
                } else {
                    format!("{} + {}", coprocessor, match memory {
                        0x4 => "RAM",
                        0x5 => "RAM + SRAM",
                        0x6 => "SRAM",
                        _ => "Unknown memory chip",
                    })
                }
            })
        })
    }
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum RegionCode {
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

impl fmt::Display for RegionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RegionCode::*;
        write!(f, "{}", match self {
            Japan        => "Japan",
            NorthAmerica => "North America",
            Europe       => "Europe",
            Sweden       => "Sweden",
            Finland      => "Finland",
            Denmark      => "Denmark",
            France       => "France",
            Netherlands  => "Netherlands",
            Spain        => "Spain",
            Germany      => "Germany",
            Italy        => "Italy",
            China        => "China",
            Indonesia    => "Indonesia",
            Korea        => "Korea",
            Global       => "Global",
            Canada       => "Canada",
            Brazil       => "Brazil",
            Australia    => "Australia",
            Other1       => "Other (1)",
            Other2       => "Other (2)",
            Other3       => "Other (3)",
        })
    }
}

pub mod address_spaces {
    use crate::addr::AddressSpace;
    pub const HEADER_LOROM: AddressSpace = 0x007FC0..=0x008000;
    pub const HEADER_HIROM: AddressSpace = 0x00FFC0..=0x010000;
}

pub mod offsets {
    pub const COMPLEMENT_CHECK: usize = 0x1C;
    pub const CHECKSUM:         usize = 0x1E;
}

pub mod sizes {
    pub const INTERNAL_HEADER:   usize = 32;
    pub const INTERNAL_ROM_NAME: usize = 21;
}

pub struct RomInternalHeader {
    pub internal_rom_name: String,
    pub map_mode: MapMode,
    pub rom_type: RomType,
    pub rom_size: u8,
    pub sram_size: u8,
    pub region_code: RegionCode,
    pub developer_id: u8,
    pub version_number: u8,
}

impl RomInternalHeader {
    pub fn from_rom_data(rom_data: &[u8], smc_header_offset: AddressPc) -> IResult<&[u8], Self> {
        use nom::{ Err as NomErr, error::{ Error as NomError, ErrorKind } };
        match RomInternalHeader::find(rom_data, smc_header_offset)?.1 {
            Some(begin) => {
                let end = begin + sizes::INTERNAL_HEADER;
                RomInternalHeader::parse(&rom_data[begin..end])
            }
            None => Err(NomErr::Error(NomError::new(rom_data, ErrorKind::Satisfy))),
        }
    }

    fn find(rom_data: &[u8], smc_header_offset: AddressPc) -> IResult<&[u8], Option<AddressPc>> {
        let lorom_header_start = smc_header_offset + *HEADER_LOROM.start();
        let hirom_header_start = smc_header_offset + *HEADER_HIROM.start();

        let lorom_complmnt_idx = lorom_header_start + offsets::COMPLEMENT_CHECK;
        let lorom_checksum_idx = lorom_header_start + offsets::CHECKSUM;
        let hirom_complmnt_idx = hirom_header_start + offsets::COMPLEMENT_CHECK;
        let hirom_checksum_idx = hirom_header_start + offsets::CHECKSUM;

        let lorom_input = &rom_data[lorom_complmnt_idx..=lorom_checksum_idx + 2];
        let hirom_input = &rom_data[hirom_complmnt_idx..=hirom_checksum_idx + 2];

        let (_, (lorom_complement, lorom_checksum)) = tuple((le_u16, le_u16))(lorom_input)?;
        let (_, (hirom_complement, hirom_checksum)) = tuple((le_u16, le_u16))(hirom_input)?;

        if (lorom_checksum ^ lorom_complement) == 0xFFFF {
            Ok((rom_data, Some(lorom_header_start)))
        } else if (hirom_checksum ^ hirom_complement) == 0xFFFF {
            Ok((rom_data, Some(hirom_header_start)))
        } else {
            Ok((rom_data, None))
        }
    }

    fn parse(input: &[u8]) -> IResult<&[u8], RomInternalHeader> {
        named!(take_internal_rom_name<&[u8], &str>, take_str!(sizes::INTERNAL_ROM_NAME));
        named!(take_map_mode<&[u8], MapMode>, map_res!(le_u8, MapMode::try_from));
        named!(take_rom_type<&[u8], RomType>, map_res!(le_u8, RomType::try_from));
        named!(take_region_code<&[u8], RegionCode>, map_res!(le_u8, RegionCode::try_from));
        named!(do_parse_header<&[u8], RomInternalHeader>, do_parse!(
            internal_rom_name: map!(take_internal_rom_name, String::from) >>
            map_mode:          take_map_mode                              >>
            rom_type:          take_rom_type                              >>
            rom_size:          le_u8                                      >>
            sram_size:         le_u8                                      >>
            region_code:       take_region_code                           >>
            developer_id:      le_u8                                      >>
            version_number:    le_u8                                      >>
            (RomInternalHeader {
                internal_rom_name,
                map_mode,
                rom_type,
                rom_size,
                sram_size,
                region_code,
                developer_id,
                version_number,
            })
        ));
        Ok(do_parse_header(input)?)
    }

    pub fn rom_size_in_kb(&self) -> u32 {
        let exponent = self.rom_size as u32;
        2u32.pow(exponent)
    }

    pub fn sram_size_in_kb(&self) -> u32 {
        match self.sram_size as u32 {
            0 => 0,
            exponent => 2u32.pow(exponent),
        }
    }
}