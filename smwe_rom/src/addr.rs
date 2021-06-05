pub use self::{masks::*, types::*};

#[rustfmt::skip]
pub mod masks {
    use super::types::Mask;
    pub const BB:     Mask = 0xFF0000; // Bank
    pub const HH:     Mask = 0x00FF00; // High byte
    pub const DD:     Mask = 0x0000FF; // Low byte
    pub const HHDD:   Mask = HH | DD;  // Absolute address
    pub const BBHHDD: Mask = BB | HH | DD; // Long address
}

pub mod types {
    use std::{
        cmp::{Ordering, PartialEq, PartialOrd},
        convert::TryFrom,
        fmt,
        ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, RangeInclusive, Rem, Shl, Shr, Sub},
    };

    use crate::{error::AddressConversionError, internal_header::MapMode};

    pub type AddrSpaceSnes = RangeInclusive<AddrSnes>;
    pub type AddrSpacePc = RangeInclusive<AddrPc>;
    pub type Mask = usize;

    macro_rules! gen_address_type {
        ($name:ident, $prim_type:ty) => {
            #[derive(Copy, Clone, Debug)]
            pub struct $name(pub $prim_type);

            impl From<$prim_type> for $name {
                fn from(addr: $prim_type) -> Self { Self(addr) }
            }

            impl From<u32> for $name {
                fn from(addr: u32) -> Self { Self(addr as $prim_type) }
            }

            impl From<$name> for $prim_type {
                fn from(this: $name) -> $prim_type { this.0 }
            }

            impl PartialEq for $name {
                fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
            }

            impl PartialOrd for $name {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.0.cmp(&other.0)) }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
            }

            macro_rules! gen_address_bin_op {
                ($op_name: ident, $op_fn_name: ident, $op: tt) => {
                    impl $op_name<$name> for $name {
                        type Output = Self;
                        fn $op_fn_name(self, rhs: Self) -> Self::Output { Self(self.0 $op rhs.0) }
                    }
                    impl $op_name<$prim_type> for $name {
                        type Output = Self;
                        fn $op_fn_name(self, rhs: $prim_type) -> Self::Output { Self(self.0 $op rhs) }
                    }
                    impl $op_name<$name> for $prim_type {
                        type Output = $name;
                        fn $op_fn_name(self, rhs: $name) -> Self::Output { $name(self $op rhs.0) }
                    }
                }
            }

            gen_address_bin_op!(Add,    add,    +);
            gen_address_bin_op!(Sub,    sub,    -);
            gen_address_bin_op!(Mul,    mul,    *);
            gen_address_bin_op!(Div,    div,    /);
            gen_address_bin_op!(Rem,    rem,    %);
            gen_address_bin_op!(BitAnd, bitand, &);
            gen_address_bin_op!(BitOr,  bitor,  |);
            gen_address_bin_op!(BitXor, bitxor, ^);
            gen_address_bin_op!(Shl,    shl,    <<);
            gen_address_bin_op!(Shr,    shr,    >>);
        }
    }

    gen_address_type!(AddrPc, usize);
    gen_address_type!(AddrSnes, usize);

    impl AddrPc {
        pub fn try_from_lorom(addr: AddrSnes) -> Result<Self, AddressConversionError> {
            if addr.is_valid_lorom() {
                Ok(Self(((addr.0 & 0x7F0000) >> 1) | (addr.0 & 0x7FFF)))
            } else {
                Err(AddressConversionError::SnesToPc(addr, MapMode::SlowLoRom))
            }
        }

        pub fn try_from_hirom(addr: AddrSnes) -> Result<Self, AddressConversionError> {
            if addr.is_valid_hirom() {
                Ok(Self(addr.0 & 0x3FFFFF))
            } else {
                Err(AddressConversionError::SnesToPc(addr, MapMode::SlowHiRom))
            }
        }

        pub fn is_valid_lorom(self) -> bool {
            self.0 < 0x400000
        }

        pub fn is_valid_hirom(self) -> bool {
            self.0 < 0x400000
        }
    }

    impl TryFrom<AddrSnes> for AddrPc {
        type Error = AddressConversionError;

        fn try_from(value: AddrSnes) -> Result<Self, Self::Error> {
            Self::try_from_lorom(value)
        }
    }

    impl fmt::LowerHex for AddrPc {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "PC {:#x}", self.0)
        }
    }

    impl fmt::UpperHex for AddrPc {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "PC {:#X}", self.0)
        }
    }

    impl AddrSnes {
        pub fn try_from_lorom(addr: AddrPc) -> Result<AddrSnes, AddressConversionError> {
            if addr.is_valid_lorom() {
                Ok(AddrSnes(((addr.0 << 1) & 0x7F0000) | (addr.0 & 0x7FFF) | 0x8000))
            } else {
                Err(AddressConversionError::PcToSnes(addr))
            }
        }

        pub fn try_from_hirom(addr: AddrPc) -> Result<AddrSnes, AddressConversionError> {
            if addr.is_valid_hirom() {
                Ok(AddrSnes(addr.0 | 0xC00000))
            } else {
                Err(AddressConversionError::PcToSnes(addr))
            }
        }

        pub fn is_valid_lorom(self) -> bool {
            let wram = (self.0 & 0xFE0000) == 0x7E0000;
            let junk = (self.0 & 0x408000) == 0x000000;
            let sram = (self.0 & 0x708000) == 0x700000;
            !wram && !junk && !sram
        }

        pub fn is_valid_hirom(self) -> bool {
            let wram = (self.0 & 0xFE0000) == 0x7E0000;
            let junk = (self.0 & 0x408000) == 0x000000;
            !wram && !junk
        }
    }

    impl TryFrom<AddrPc> for AddrSnes {
        type Error = AddressConversionError;

        fn try_from(value: AddrPc) -> Result<Self, Self::Error> {
            Self::try_from_lorom(value)
        }
    }

    impl fmt::LowerHex for AddrSnes {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "SNES ${:x}", self.0)
        }
    }

    impl fmt::UpperHex for AddrSnes {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "SNES ${:X}", self.0)
        }
    }
}
