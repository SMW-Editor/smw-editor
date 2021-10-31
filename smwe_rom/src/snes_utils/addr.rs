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
        ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub},
    };

    use crate::error::AddressError;

    pub trait Addr:
        Sized
        + Copy
        + Clone
        + PartialOrd
        + fmt::LowerHex
        + fmt::UpperHex
        + Add<usize, Output = Self>
        + Add<Self, Output = Self>
        + BitAnd<usize, Output = Self>
        + BitAnd<Self, Output = Self>
        + BitOr<usize, Output = Self>
        + BitOr<Self, Output = Self>
        + BitXor<usize, Output = Self>
        + BitXor<Self, Output = Self>
        + Div<usize, Output = Self>
        + Div<Self, Output = Self>
        + Mul<usize, Output = Self>
        + Mul<Self, Output = Self>
        + Rem<usize, Output = Self>
        + Rem<Self, Output = Self>
        + Shl<usize, Output = Self>
        + Shl<Self, Output = Self>
        + Shr<usize, Output = Self>
        + Shr<Self, Output = Self>
        + Sub<usize, Output = Self>
        + Sub<Self, Output = Self>
    {
        type OppositeAddr: Addr;
        fn try_from_lorom(addr: Self::OppositeAddr) -> Result<Self, AddressError>;
        fn try_from_hirom(addr: Self::OppositeAddr) -> Result<Self, AddressError>;
        fn is_valid_lorom(&self) -> bool;
        fn is_valid_hirom(&self) -> bool;
    }

    pub type Mask = usize;

    macro_rules! gen_address_type {
        ($name:ident) => {
            #[derive(Copy, Clone, Debug)]
            pub struct $name(pub usize);

            impl From<usize> for $name {
                fn from(addr: usize) -> Self { Self(addr) }
            }

            impl From<u32> for $name {
                fn from(addr: u32) -> Self { Self(addr as usize) }
            }

            impl From<$name> for usize {
                fn from(this: $name) -> usize { this.0 }
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
                    impl $op_name<usize> for $name {
                        type Output = Self;
                        fn $op_fn_name(self, rhs: usize) -> Self::Output { Self(self.0 $op rhs) }
                    }
                    impl $op_name<$name> for usize {
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

    gen_address_type!(AddrPc);
    gen_address_type!(AddrSnes);

    impl Addr for AddrPc {
        type OppositeAddr = AddrSnes;

        fn try_from_lorom(addr: AddrSnes) -> Result<Self, AddressError> {
            if addr.is_valid_lorom() {
                Ok(Self(((addr.0 & 0x7F0000) >> 1) | (addr.0 & 0x7FFF)))
            } else {
                Err(AddressError::InvalidSnesLoRom(addr))
            }
        }

        fn try_from_hirom(addr: AddrSnes) -> Result<Self, AddressError> {
            if addr.is_valid_hirom() {
                Ok(Self(addr.0 & 0x3FFFFF))
            } else {
                Err(AddressError::InvalidSnesHiRom(addr))
            }
        }

        fn is_valid_lorom(&self) -> bool {
            self.0 < 0x400000
        }

        fn is_valid_hirom(&self) -> bool {
            self.0 < 0x400000
        }
    }

    impl TryFrom<AddrSnes> for AddrPc {
        type Error = AddressError;

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

    impl Addr for AddrSnes {
        type OppositeAddr = AddrPc;

        fn try_from_lorom(addr: AddrPc) -> Result<Self, AddressError> {
            if addr.is_valid_lorom() {
                Ok(Self(((addr.0 << 1) & 0x7F0000) | (addr.0 & 0x7FFF) | 0x8000))
            } else {
                Err(AddressError::InvalidPcLoRom(addr))
            }
        }

        fn try_from_hirom(addr: AddrPc) -> Result<Self, AddressError> {
            if addr.is_valid_hirom() {
                Ok(Self(addr.0 | 0xC00000))
            } else {
                Err(AddressError::InvalidPcHiRom(addr))
            }
        }

        fn is_valid_lorom(&self) -> bool {
            let wram = (self.0 & 0xFE0000) == 0x7E0000;
            let junk = (self.0 & 0x408000) == 0x000000;
            let sram = (self.0 & 0x708000) == 0x700000;
            !wram && !junk && !sram
        }

        fn is_valid_hirom(&self) -> bool {
            let wram = (self.0 & 0xFE0000) == 0x7E0000;
            let junk = (self.0 & 0x408000) == 0x000000;
            !wram && !junk
        }
    }

    impl TryFrom<AddrPc> for AddrSnes {
        type Error = AddressError;

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
