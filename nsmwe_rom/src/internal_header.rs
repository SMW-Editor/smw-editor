pub use self::address_spaces::*;

use crate::{
    addr::AddressPC,
    get_byte_at,
    get_word_at,
    get_slice_at,
};

use num_enum::{
    IntoPrimitive,
    TryFromPrimitive,
};

use std::{
    clone::Clone,
    convert::TryFrom,
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

impl fmt::Display for DestinationCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DestinationCode::*;
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
    pub const HEADER_LOROM: AddressSpace = 0x007FB0..=0x008000;
    pub const HEADER_HIROM: AddressSpace = 0x00FFB0..=0x010000;
}

pub mod offset {
    pub const INTERNAL_ROM_NAME:  u32 = 0x10;
    pub const MAP_MODE:           u32 = 0x25;
    pub const ROM_TYPE:           u32 = 0x26;
    pub const ROM_SIZE:           u32 = 0x27;
    pub const SRAM_SIZE:          u32 = 0x28;
    pub const DESTINATION_CODE:   u32 = 0x29;
    pub const DEVELOPER_ID:       u32 = 0x2A;
    pub const VERSION_NUMBER:     u32 = 0x2B;
    pub const COMPLEMENT_CHECK:   u32 = 0x2C;
    pub const CHECKSUM:           u32 = 0x2E;
}

pub struct RomInternalHeader {
    pub internal_rom_name: String,
    pub map_mode: MapMode,
    pub rom_type: RomType,
    pub rom_size: u8,
    pub sram_size: u8,
    pub destination_code: DestinationCode,
    pub developer_id: u8,
    pub version_number: u8,
}

impl RomInternalHeader {
    pub fn from_rom_data(data: &[u8], smc_header_offset: AddressPC) -> Result<Self, String> {
        let begin = RomInternalHeader::find(data, smc_header_offset)?;
        Ok(RomInternalHeader {
            internal_rom_name: {
                let slice = get_slice_at(data, begin + offset::INTERNAL_ROM_NAME, 21)?;
                let slice = Vec::from(slice);
                String::from_utf8(slice).unwrap_or(String::from("error"))
            },
            map_mode: {
                let mm = get_byte_at(data, begin + offset::MAP_MODE)?;
                MapMode::try_from(mm)
                    .or_else(|_| Err(String::from("Invalid map mode.")))?
            },
            rom_type: {
                let rt = get_byte_at(data, begin + offset::ROM_TYPE)?;
                RomType::try_from(rt)
                    .or_else(|_| Err(String::from("Invalid ROM type.")))?
            },
            rom_size:
                get_byte_at(data, begin + offset::ROM_SIZE)?,
            sram_size:
                get_byte_at(data, begin + offset::SRAM_SIZE)?,
            destination_code: {
                let dc = get_byte_at(data, begin + offset::DESTINATION_CODE)?;
                DestinationCode::try_from(dc)
                    .or_else(|_| Err(String::from("Invalid destination code.")))?
            },
            developer_id:
                get_byte_at(data, begin + offset::DEVELOPER_ID)?,
            version_number:
                get_byte_at(data, begin + offset::VERSION_NUMBER)?,
        })
    }

    fn find(data: &[u8], smc_header_offset: u32) -> Result<AddressPC, String> {
        let lorom_header_start = smc_header_offset + *HEADER_LOROM.start();
        let hirom_header_start = smc_header_offset + *HEADER_HIROM.start();

        let lorom_checksum_idx = lorom_header_start + offset::CHECKSUM;
        let lorom_complmnt_idx = lorom_header_start + offset::COMPLEMENT_CHECK;
        let hirom_checksum_idx = hirom_header_start + offset::CHECKSUM;
        let hirom_complmnt_idx = hirom_header_start + offset::COMPLEMENT_CHECK;

        let lorom_checksum = get_word_at(data, lorom_checksum_idx)?;
        let lorom_complmnt = get_word_at(data, lorom_complmnt_idx)?;
        let hirom_checksum = get_word_at(data, hirom_checksum_idx)?;
        let hirom_complmnt = get_word_at(data, hirom_complmnt_idx)?;

        if (lorom_checksum ^ lorom_complmnt) == 0xFFFF {
            Ok(lorom_header_start)
        } else if (hirom_checksum ^ hirom_complmnt) == 0xFFFF {
            Ok(hirom_header_start)
        } else {
            Err(String::from("Could not locate internal header: both checksums are invalid."))
        }
    }

    pub fn rom_size_in_kb(&self) -> u32 {
        let exponent = self.rom_size as u32;
        2u32.pow(exponent)
    }

    pub fn sram_size_in_kb(&self) -> u32 {
        let exponent = self.sram_size as u32;
        let non_zero = (exponent > 0) as u32;
        non_zero * 2u32.pow(exponent)
    }
}