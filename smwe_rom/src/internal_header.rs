use std::{clone::Clone, convert::TryFrom, fmt};

use nom::{
    combinator::{map, map_res},
    multi::{count, many1},
    number::complete::{le_u16, le_u8},
    sequence::pair,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    error::InternalHeaderParseError,
    snes_utils::{
        addr::{AddrPc, AddrSnes},
        rom::Rom,
        rom_slice::PcSlice,
    },
};

#[rustfmt::skip]
pub mod offsets {
    pub const COMPLEMENT_CHECK: usize = 0x1C;
    pub const CHECKSUM:         usize = 0x1E;
}

#[rustfmt::skip]
pub mod sizes {
    pub const INTERNAL_HEADER:   usize = 64;
    pub const INTERNAL_ROM_NAME: usize = 21;
}

// -------------------------------------------------------------------------------------------------

pub struct RomInternalHeader {
    pub internal_rom_name: String,
    pub map_mode:          MapMode,
    pub rom_type:          RomType,
    pub rom_size:          u8,
    pub sram_size:         u8,
    pub region_code:       RegionCode,
    pub developer_id:      u8,
    pub version_number:    u8,
    pub interrupt_vectors: Vec<AddrSnes>,
}

#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MapMode {
    SlowLoRom   = 0b100000,
    SlowHiRom   = 0b100001,
    SlowExLoRom = 0b100010,
    SlowExHiRom = 0b100100,
    FastLoRom   = 0b110000,
    FastHiRom   = 0b110001,
    FastExLoRom = 0b110010,
    FastExHiRom = 0b110100,
}

#[derive(Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum RomType {
    Rom               = 0x00,
    RomRam            = 0x01,
    RomRamSram        = 0x02,

    RomDsp            = 0x03,
    RomSuperFx        = 0x13,
    RomObc1           = 0x23,
    RomSa1            = 0x33,
    RomSdd1           = 0x43,
    RomSrtc           = 0x53,
    RomOther          = 0xE3,
    RomCustom         = 0xF3,

    RomDspRam         = 0x04,
    RomSuperFxRam     = 0x14,
    RomObc1Ram        = 0x24,
    RomSa1Ram         = 0x34,
    RomSdd1Ram        = 0x44,
    RomSRtcRam        = 0x54,
    RomOtherRam       = 0xE4,
    RomCustomRam      = 0xF4,

    RomDspRamSram     = 0x05,
    RomSuperFxRamSram = 0x15,
    RomObc1RamSram    = 0x25,
    RomSa1RamSram     = 0x35,
    RomSdd1RamSram    = 0x45,
    RomSRtcRamSram    = 0x55,
    RomOtherRamSram   = 0xE5,
    RomCustomRamSram  = 0xF5,

    RomDspSram        = 0x06,
    RomSuperFxSram    = 0x16,
    RomObc1Sram       = 0x26,
    RomSa1Sram        = 0x36,
    RomSdd1Sram       = 0x46,
    RomSRtcSram       = 0x56,
    RomOtherSram      = 0xE6,
    RomCustomSram     = 0xF6,
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

// -------------------------------------------------------------------------------------------------

impl RomInternalHeader {
    pub fn parse(rom: &Rom) -> Result<Self, InternalHeaderParseError> {
        let rih_slice = RomInternalHeader::find(rom)?;
        let name_slice = rih_slice.resize(sizes::INTERNAL_ROM_NAME);
        let byte_slice = name_slice.skip_forward(1).resize(1);

        Ok(Self {
            internal_rom_name: rom
                .with_error_mapper(InternalHeaderParseError::ReadRomName)
                .slice_pc(name_slice)
                .parse(map_res(many1(le_u8), |s| std::str::from_utf8(&s).map(String::from)))?,
            map_mode:          rom
                .with_error_mapper(InternalHeaderParseError::ReadMapMode)
                .slice_pc(byte_slice)
                .parse(map_res(le_u8, MapMode::try_from))?,
            rom_type:          rom
                .with_error_mapper(InternalHeaderParseError::ReadRomType)
                .slice_pc(byte_slice.skip_forward(1))
                .parse(map_res(le_u8, RomType::try_from))?,
            rom_size:          rom
                .with_error_mapper(InternalHeaderParseError::ReadRomSize)
                .slice_pc(byte_slice.skip_forward(2))
                .parse(le_u8)?,
            sram_size:         rom
                .with_error_mapper(InternalHeaderParseError::ReadSramSize)
                .slice_pc(byte_slice.skip_forward(3))
                .parse(le_u8)?,
            region_code:       rom
                .with_error_mapper(InternalHeaderParseError::ReadRegionCode)
                .slice_pc(byte_slice.skip_forward(4))
                .parse(map_res(le_u8, RegionCode::try_from))?,
            developer_id:      rom
                .with_error_mapper(InternalHeaderParseError::ReadDeveloperId)
                .slice_pc(byte_slice.skip_forward(5))
                .parse(le_u8)?,
            version_number:    rom
                .with_error_mapper(InternalHeaderParseError::ReadVersionNumber)
                .slice_pc(byte_slice.skip_forward(6))
                .parse(le_u8)?,
            interrupt_vectors: {
                let vectors_slice = byte_slice.skip_forward(15).resize(2 * 6);
                let mut parse_vectors = count(map(le_u16, |addr| AddrSnes(addr as usize)), 6);
                let native = rom
                    .with_error_mapper(InternalHeaderParseError::ReadNativeModeInterruptVectors)
                    .slice_pc(vectors_slice)
                    .parse(&mut parse_vectors)?;
                let emulation = rom
                    .with_error_mapper(InternalHeaderParseError::ReadEmulationModeInterruptVectors)
                    .slice_pc(vectors_slice.skip_forward(1).offset_forward(4))
                    .parse(&mut parse_vectors)?;
                native.into_iter().chain(emulation.into_iter()).collect()
            },
        })
    }

    fn find(rom: &Rom) -> Result<PcSlice, InternalHeaderParseError> {
        const HEADER_LOROM: PcSlice = PcSlice::new(AddrPc(0x007FC0), sizes::INTERNAL_HEADER);
        const HEADER_HIROM: PcSlice = PcSlice::new(AddrPc(0x00FFC0), sizes::INTERNAL_HEADER);

        let lo_cpl_csm = HEADER_LOROM.offset_forward(offsets::COMPLEMENT_CHECK).resize(4);
        let hi_cpl_csm = HEADER_HIROM.offset_forward(offsets::COMPLEMENT_CHECK).resize(4);

        let (lo_cpl, lo_csm) = rom
            .with_error_mapper(InternalHeaderParseError::ReadLoRomChecksum)
            .slice_pc(lo_cpl_csm)
            .parse(pair(le_u16, le_u16))?;
        let (hi_cpl, hi_csm) = rom
            .with_error_mapper(InternalHeaderParseError::ReadHiRomChecksum)
            .slice_pc(hi_cpl_csm)
            .parse(pair(le_u16, le_u16))?;

        if (lo_csm ^ lo_cpl) == 0xFFFF {
            log::info!("Internal ROM header found at LoROM location: {:#X}", HEADER_LOROM.begin);
            Ok(HEADER_LOROM)
        } else if (hi_csm ^ hi_cpl) == 0xFFFF {
            log::info!("Internal ROM header found at HiROM location: {:#X}", HEADER_HIROM.begin);
            Ok(HEADER_HIROM)
        } else {
            log::error!("Couldn't find internal ROM header due to invalid checksums");
            log::error!("(LoROM: {:X}^{:X}, HiROM: {:X}^{:X})", lo_cpl, lo_csm, hi_cpl, hi_csm);
            Err(InternalHeaderParseError::NotFound)
        }
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

impl fmt::Display for MapMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MapMode::*;
        write!(f, "{}", match self {
            SlowLoRom => "LoROM",
            SlowHiRom => "HiROM",
            SlowExLoRom => "ExLoROM",
            SlowExHiRom => "ExHiROM",
            FastLoRom => "Fast LoROM",
            FastHiRom => "Fast HiROM",
            FastExLoRom => "Fast ExLoROM",
            FastExHiRom => "Fast ExHiROM",
        })
    }
}

#[rustfmt::skip]
impl MapMode {
    pub fn as_u8(&self) -> u8 { (*self).into() }
    pub fn is_slow(&self)    -> bool { (self.as_u8() & 0b010000) == 0 }
    pub fn is_fast(&self)    -> bool { !self.is_slow() }
    pub fn is_lorom(&self)   -> bool { (self.as_u8() & 0b000001) == 0 }
    pub fn is_hirom(&self)   -> bool { (self.as_u8() & 0b000001) != 0 }
    pub fn is_exlorom(&self) -> bool { (self.as_u8() & 0b000010) != 0 }
    pub fn is_exhirom(&self) -> bool { (self.as_u8() & 0b000100) != 0 }
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
                    format!("{coprocessor} + {}", match memory {
                        0x4 => "RAM",
                        0x5 => "RAM + SRAM",
                        0x6 => "SRAM",
                        _ => "Unknown memory chip",
                    })
                }
            }),
        })
    }
}

impl fmt::Display for RegionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RegionCode::*;
        write!(f, "{}", match self {
            Japan => "Japan",
            NorthAmerica => "North America",
            Europe => "Europe",
            Sweden => "Sweden",
            Finland => "Finland",
            Denmark => "Denmark",
            France => "France",
            Netherlands => "Netherlands",
            Spain => "Spain",
            Germany => "Germany",
            Italy => "Italy",
            China => "China",
            Indonesia => "Indonesia",
            Korea => "Korea",
            Global => "Global",
            Canada => "Canada",
            Brazil => "Brazil",
            Australia => "Australia",
            Other1 => "Other (1)",
            Other2 => "Other (2)",
            Other3 => "Other (3)",
        })
    }
}
