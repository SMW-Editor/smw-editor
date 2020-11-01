use crate::addr::AddressPC;

pub use address_spaces::*;
pub use addresses::*;

pub enum MapMode {
    FastRom = 0b110000,
    HiRom   = 0b1,
    LoRom   = 0b0,
}

pub enum RomType {
    Rom          = 0x00,
    RomRam       = 0x01,
    RomRamSram   = 0x02,

    RomDsp     = 0x03,
    RomSuperFx = 0x13,
    RomObc1    = 0x23,
    RomSa1     = 0x33,
    RomOther   = 0xE3,
    RomCustom  = 0xF3,

    RomDspRam     = 0x04,
    RomSuperFxRam = 0x14,
    RomObc1Ram    = 0x24,
    RomSa1Ram     = 0x34,
    RomOtherRam   = 0xE4,
    RomCustomRam  = 0xF4,

    RomDspRamSram     = 0x05,
    RomSuperFxRamSram = 0x15,
    RomObc1RamSram    = 0x25,
    RomSa1RamSram     = 0x35,
    RomOtherRamSram   = 0xE5,
    RomCustomRamSram  = 0xF5,

    RomDspSram     = 0x06,
    RomSuperFxSram = 0x16,
    RomObc1Sram    = 0x26,
    RomSa1Sram     = 0x36,
    RomOtherSram   = 0xE6,
    RomCustomSram  = 0xF6,
}

pub enum DestinationCode {
    Japan        = 0x00,
    NorthAmerica = 0x01,
    Europe       = 0x02,
    Scandinavia  = 0x03,
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
    maker_code: [u8; 2],
    game_code: [u8; 4],
    expansion_ram_size: u8,
    special_version: u8,
    cartridge_type: u8,

    internal_rom_name: [u8; 21],
    map_mode: MapMode,
    rom_type: RomType,
    rom_size: u8,  // actual size = 2^rom_size
    sram_size: u8, // actual size = 2^sram_size
    destination_code: DestinationCode,
    version_number: u8,
}

pub mod address_spaces {
    use crate::addr::AddressSpace;
    pub const HEADER_LOROM: AddressSpace = 0x007FC0..=0x008000;
    pub const HEADER_HIROM: AddressSpace = 0x00FFC0..=0x010000;
}

pub mod addresses {
    use crate::addr::AddressPC;
    pub const HEADER_LOROM_COMPLIMENT_CHECK: AddressPC = 0x007FDC;
    pub const HEADER_LOROM_CHECKSUM:         AddressPC = 0x007FDE;

    pub const HEADER_HIROM_COMPLIMENT_CHECK: AddressPC = 0x00FFDC;
    pub const HEADER_HIROM_CHECKSUM:         AddressPC = 0x00FFDE;
}

impl InternalHeader {
    pub fn from_rom_data(data: &[u8], smc_header_offset: AddressPC) -> Self {
        let _internal_header_offset = InternalHeader::find(data, smc_header_offset);

        let maker_code = [0x0; 2];
        let game_code = [0x0; 4];
        let expansion_ram_size= 0x0;
        let special_version = 0x0;
        let cartridge_type = 0x0;

        let internal_rom_name = [0x0; 21];
        let map_mode = MapMode::LoRom;
        let rom_type = RomType::Rom;
        let rom_size = 0x0;
        let sram_size = 0x0;
        let destination_code = DestinationCode::NorthAmerica;
        let version_number = 0x0;

        InternalHeader {
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
        }
    }

    fn find(data: &[u8], smc_header_offset: u32) -> Result<AddressPC, String> {
        let lorom_checksum_idx = smc_header_offset + HEADER_LOROM_CHECKSUM;
        let lorom_complmnt_idx = smc_header_offset + HEADER_LOROM_COMPLIMENT_CHECK;
        let hirom_checksum_idx = smc_header_offset + HEADER_HIROM_CHECKSUM;
        let hirom_complmnt_idx = smc_header_offset + HEADER_HIROM_COMPLIMENT_CHECK;

        use crate::get_word_at;
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