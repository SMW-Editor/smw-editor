pub use address_spaces::*;
pub use aliases::*;
pub use conversions::*;
pub use helpers::*;
pub use masks::*;

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
    pub const RAM_MIRROR_BB:   AddressSpace = 0x000000..=0x3F0000;
    pub const RAM_MIRROR_HHDD: AddressSpace = 0x000000..=0x001FFF;
    pub const RAM:             AddressSpace = 0x7E0000..=0x7FFFFF;
    pub const SRAM_BB:         AddressSpace = 0x700000..=0x730000;
    pub const SRAM_HHDD:       AddressSpace = 0x000000..=0x007FFF;
    pub const STACK:           AddressSpace = 0x7E0000..=0x7E1FFF;
}

pub mod aliases {
    pub type AddressPC = u32;
    pub type AddressSNES = u32;
    pub type AddressSpace = std::ops::RangeInclusive<super::AddressSNES>;
    pub type Mask = u32;
}

pub mod helpers {
    use crate::addr::{
        address_spaces::*,
        aliases::*,
        masks::*,
    };

    pub fn get_bb_hhdd(addr: AddressSNES) -> (u32, u32) {
        (addr & BB, addr & HHDD)
    }

    pub fn is_in_ram(addr: AddressSNES) -> bool {
        RAM.contains(&addr)
    }

    pub fn is_in_stack(addr: AddressSNES) -> bool {
        STACK.contains(&addr)
    }

    pub fn is_valid_lorom_address(addr: AddressSNES) -> bool {
        let (bb, hhdd) = get_bb_hhdd(addr);
        LOROM_BB.contains(&bb) && LOROM_HHDD.contains(&hhdd)
    }

    pub fn is_valid_hirom_address(addr: AddressSNES) -> bool {
        let (bb, hhdd) = get_bb_hhdd(addr);
        HIROM_BB.contains(&bb) && HIROM_HHDD.contains(&hhdd)
    }

    pub fn is_in_sram(addr: AddressSNES) -> bool {
        let (bb, hhdd) = get_bb_hhdd(addr);
        SRAM_BB.contains(&bb) && SRAM_HHDD.contains(&hhdd)
    }

    pub fn is_stack_mirror(addr: AddressSNES) -> bool {
        let (bb, hhdd) = get_bb_hhdd(addr);
        RAM_MIRROR_BB.contains(&bb) && RAM_MIRROR_HHDD.contains(&hhdd)
    }

    pub fn mirror_to_stack(addr: AddressSNES) -> Option<AddressSNES> {
        if is_stack_mirror(addr) {
            let hhdd = addr & HHDD;
            Some(*STACK.start() | hhdd)
        } else {
            None
        }
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

        pub fn hirom(addr: AddressPC) -> AddressSNES {
            let (bb, hhdd) = get_bb_hhdd(addr);
            ((bb + *HIROM_BB.start()) | hhdd) & BBHHDD
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
                Ok((((bb & 0x7F0000) | hhdd) - 0x7E00) & BBHHDD)
            } else {
                Err(format!("Invalid LoROM address: {:#x}.", addr))
            }
        }

        pub fn hirom(addr: AddressSNES) -> Result<AddressPC, String> {
            if is_valid_hirom_address(addr) {
                let (bb, hhdd) = get_bb_hhdd(addr);
                Ok(((bb - *HIROM_BB.start()) | hhdd) & BBHHDD)
            } else {
                Err(format!("Invalid HiROM address: {:#x}.", addr))
            }
        }
    }
}
