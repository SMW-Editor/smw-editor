pub use self::address_spaces::*;
pub use self::aliases::*;
pub use self::conversions::*;
pub use self::helpers::*;
pub use self::masks::*;

pub mod masks {
    use super::aliases::Mask;
    pub const BB:     Mask = 0xFF0000; // Bank
    pub const HH:     Mask = 0x00FF00; // High byte
    pub const DD:     Mask = 0x0000FF; // Low byte
    pub const HHDD:   Mask = HH | DD;  // Absolute address
    pub const BBHHDD: Mask = BB | HH | DD; // Long address
}

pub mod address_spaces {
    use super::aliases::AddressSpace;
    pub const LOROM_BB:        AddressSpace = 0x000000..=0x6F0000;
    pub const LOROM_HHDD:      AddressSpace = 0x008000..=0x00FFFF;
    pub const HIROM_BB:        AddressSpace = 0xC00000..=0xFF0000;
    pub const HIROM_HHDD:      AddressSpace = 0x000000..=0x00FFFF;
}

pub mod aliases {
    pub type AddressPC = u32;
    pub type AddressSNES = u32;
    pub type AddressSpace = std::ops::RangeInclusive<AddressSNES>;
    pub type Mask = u32;
}

pub mod helpers {
    use crate::addr::{
        address_spaces::*,
        aliases::AddressSNES,
        masks::*,
    };

    pub fn get_bb_hhdd(addr: AddressSNES) -> (u32, u32) {
        (addr & BB, addr & HHDD)
    }

    pub fn is_valid_lorom_address(addr: AddressSNES) -> bool {
        let (bb, hhdd) = get_bb_hhdd(addr);
        LOROM_BB.contains(&bb) && LOROM_HHDD.contains(&hhdd)
    }

    pub fn is_valid_hirom_address(addr: AddressSNES) -> bool {
        let (bb, hhdd) = get_bb_hhdd(addr);
        HIROM_BB.contains(&bb) && HIROM_HHDD.contains(&hhdd)
    }
}

pub mod conversions {
    pub mod pc_to_snes {
        use crate::addr::{
            address_spaces::*,
            aliases::*,
            helpers::*,
            masks::*,
        };

        pub fn lorom(addr: AddressPC) -> Result<AddressSNES, String> {
            if addr < 0x400000 {
                let bb = (addr & BB) | if addr >= 0x380000 { 0x800000 } else { 0 };
                let hh = (addr & 0x7F00) + 0x8000;
                let dd = addr & DD;
                Ok(bb | hh | dd)
            } else {
                Err(format!("PC address {:#x} is too big for LoROM.", addr))
            }
        }

        pub fn hirom(addr: AddressPC) -> Result<AddressSNES, String> {
            let (bb, hhdd) = get_bb_hhdd(addr);
            Ok(((bb + *HIROM_BB.start()) | hhdd) & BBHHDD)
        }
    }

    pub mod snes_to_pc {
        use crate::addr::{
            address_spaces::*,
            aliases::*,
            helpers::*,
            masks::*,
        };

        pub fn lorom(addr: AddressSNES) -> Result<AddressPC, String> {
            if is_valid_lorom_address(addr) {
                let (bb, hhdd) = get_bb_hhdd(addr);
                Ok((((bb & 0x7F0000) | hhdd) - 0x8000) & BBHHDD)
            } else {
                Err(format!("Invalid LoROM address: ${:x}.", addr))
            }
        }

        pub fn hirom(addr: AddressSNES) -> Result<AddressPC, String> {
            if is_valid_hirom_address(addr) {
                let (bb, hhdd) = get_bb_hhdd(addr);
                Ok(((bb - *HIROM_BB.start()) | hhdd) & BBHHDD)
            } else {
                Err(format!("Invalid HiROM address: ${:x}.", addr))
            }
        }
    }
}
