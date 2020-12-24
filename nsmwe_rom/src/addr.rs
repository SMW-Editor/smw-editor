pub use self::address_spaces::*;
pub use self::helpers::*;
pub use self::masks::*;
pub use self::types::*;

pub mod masks {
    use super::types::Mask;
    pub const BB:     Mask = 0xFF0000; // Bank
    pub const HH:     Mask = 0x00FF00; // High byte
    pub const DD:     Mask = 0x0000FF; // Low byte
    pub const HHDD:   Mask = HH | DD;  // Absolute address
    pub const BBHHDD: Mask = BB | HH | DD; // Long address
}

pub mod address_spaces {
    use super::types::{AddrSnes, AddrSpaceSnes};
    pub const LOROM_BB:   AddrSpaceSnes = AddrSnes(0x000000)..=AddrSnes(0x6F0000);
    pub const LOROM_HHDD: AddrSpaceSnes = AddrSnes(0x008000)..=AddrSnes(0x00FFFF);
    pub const HIROM_BB:   AddrSpaceSnes = AddrSnes(0xC00000)..=AddrSnes(0xFF0000);
    pub const HIROM_HHDD: AddrSpaceSnes = AddrSnes(0x000000)..=AddrSnes(0x00FFFF);
}

pub mod types {
    use crate::{
        addr::{address_spaces::*, helpers::*, masks::*},
        error::AddressConversionError,
        internal_header::MapMode,
    };

    use std::{
        cmp::{Ordering, PartialEq, PartialOrd},
        convert::TryFrom,
        fmt,
        ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub, RangeInclusive},
    };

    pub type AddrSpaceSnes = RangeInclusive<AddrSnes>;
    pub type AddrSpacePc = RangeInclusive<AddrPc>;
    pub type Mask = usize;

    macro_rules! gen_address_type {
        ($name:ident, $prim_type:ty) => {
            #[derive(Copy, Clone, Debug)]
            pub struct $name(pub $prim_type);

            impl From<usize> for $name {
                fn from(addr: $prim_type) -> Self { Self(addr) }
            }

            impl Into<$prim_type> for $name {
                fn into(self) -> $prim_type { self.0 }
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

            impl fmt::LowerHex for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:x}", self.0) }
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
            if is_valid_lorom_address(addr) {
                let (bb, hhdd) = get_bb_hhdd(addr.into());
                Ok(Self(((((bb & 0x7F0000) | hhdd) - 0x8000) & BBHHDD).into()))
            } else {
                Err(AddressConversionError::Snes(addr, MapMode::SlowLoRom))
            }
        }

        pub fn try_from_hirom(addr: AddrSnes) -> Result<Self, AddressConversionError> {
            if is_valid_hirom_address(addr) {
                let (bb, hhdd) = get_bb_hhdd(addr.into());
                Ok(Self((((bb - *HIROM_BB.start()) | hhdd) & BBHHDD).into()))
            } else {
                Err(AddressConversionError::Snes(addr, MapMode::SlowHiRom))
            }
        }
    }

    impl TryFrom<AddrSnes> for AddrPc {
        type Error = AddressConversionError;
        fn try_from(value: AddrSnes) -> Result<Self, Self::Error> {
            Self::try_from_lorom(value)
        }
    }

    impl AddrSnes {
        pub fn try_from_lorom(addr: AddrPc) -> Result<AddrSnes, AddressConversionError> {
            if addr < AddrPc(0x400000) {
                let bb = (addr & BB) | if addr >= AddrPc(0x380000) { 0x800000 } else { 0 };
                let hh = (addr & 0x7F00) + 0x8000;
                let dd = addr & DD;
                Ok(AddrSnes((bb | hh | dd).into()))
            } else {
                Err(AddressConversionError::Pc(addr))
            }
        }

        pub fn try_from_hirom(addr: AddrPc) -> Result<AddrSnes, AddressConversionError> {
            let (bb, hhdd) = get_bb_hhdd(addr.into());
            Ok(((bb + *HIROM_BB.start()) | hhdd) & BBHHDD)
        }
    }

    impl TryFrom<AddrPc> for AddrSnes {
        type Error = AddressConversionError;
        fn try_from(value: AddrPc) -> Result<Self, Self::Error> {
            Self::try_from_lorom(value)
        }
    }
}

pub mod helpers {
    use crate::addr::{
        address_spaces::*,
        types::AddrSnes,
        masks::*,
    };

    pub fn get_bb_hhdd(addr: usize) -> (usize, usize) {
        (addr & BB, addr & HHDD)
    }

    pub fn is_valid_lorom_address(addr: AddrSnes) -> bool {
        let (bb, hhdd) = get_bb_hhdd(addr.into());
        LOROM_BB.contains(&bb.into()) && LOROM_HHDD.contains(&hhdd.into())
    }

    pub fn is_valid_hirom_address(addr: AddrSnes) -> bool {
        let (bb, hhdd) = get_bb_hhdd(addr.into());
        HIROM_BB.contains(&bb.into()) && HIROM_HHDD.contains(&hhdd.into())
    }
}
