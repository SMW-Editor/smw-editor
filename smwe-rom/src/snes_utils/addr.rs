use std::{convert::TryFrom, fmt, num::ParseIntError, ops::*};

use num_traits::{cast::cast, *};
use paste::*;
use thiserror::Error;

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum AddressError {
    #[error("Invalid PC LoROM address {0:#x}")]
    InvalidPcLoRom(AddrPc),
    #[error("Invalid PC LoROM address {0:#x}")]
    InvalidPcHiRom(AddrPc),
    #[error("Invalid SNES LoROM address {0:#x}")]
    InvalidSnesLoRom(AddrSnes),
    #[error("Invalid SNES LoROM address {0:#x}")]
    InvalidSnesHiRom(AddrSnes),
}

// -------------------------------------------------------------------------------------------------

pub type AddrInner = u32;
pub type AddrInnerSigned = i32;

// -------------------------------------------------------------------------------------------------

/// Bank
pub const MASK_BB: AddrInner = 0xFF0000;
/// High byte
pub const MASK_HH: AddrInner = 0x00FF00;
/// Low byte
pub const MASK_DD: AddrInner = 0x0000FF;
/// Absolute address
pub const MASK_HHDD: AddrInner = MASK_HH | MASK_DD;
/// Long address
pub const MASK_BBHHDD: AddrInner = MASK_BB | MASK_HH | MASK_DD;

// -------------------------------------------------------------------------------------------------

pub trait Addr: Clone + NumOps<usize, Self> + PrimInt + fmt::LowerHex + fmt::UpperHex {
    type OppositeAddr: Addr;
    const MIN: Self;
    fn try_from_lorom(addr: Self::OppositeAddr) -> Result<Self, AddressError>;
    fn try_from_hirom(addr: Self::OppositeAddr) -> Result<Self, AddressError>;
    fn is_valid_lorom(&self) -> bool;
    fn is_valid_hirom(&self) -> bool;
}

// -------------------------------------------------------------------------------------------------

macro_rules! gen_address_type {
    ($T:ident) => {
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        pub struct $T(pub AddrInner);

        impl $T {
            #[inline]
            pub fn as_index(self) -> usize {
                self.0 as usize
            }
        }

        impl Default for $T {
            #[inline]
            fn default() -> Self {
                Self::MIN
            }
        }

        impl Zero for $T {
            #[inline]
            fn zero() -> Self {
                Self::MIN
            }

            #[inline]
            fn set_zero(&mut self) {
                self.0 = Self::zero().0
            }

            #[inline]
            fn is_zero(&self) -> bool {
                *self == Self::zero()
            }
        }

        impl One for $T {
            #[inline]
            fn one() -> Self {
                Self::MIN + 1
            }

            fn set_one(&mut self) {
                self.0 = Self::one().0
            }

            #[inline]
            fn is_one(&self) -> bool {
                *self == Self::one()
            }
        }

        impl Num for $T {
            type FromStrRadixErr = ParseIntError;

            #[inline]
            fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                Ok(Self(AddrInner::from_str_radix(str, radix)?))
            }
        }

        impl NumCast for $T {
            #[inline]
            fn from<T: ToPrimitive>(n: T) -> Option<Self> {
                Some(Self(n.to_u32()? as AddrInner))
            }
        }

        impl ToPrimitive for $T {
            #[inline]
            fn to_i64(&self) -> Option<i64> {
                cast::<AddrInner, i64>(self.0)
            }

            #[inline]
            fn to_u64(&self) -> Option<u64> {
                cast::<AddrInner, u64>(self.0)
            }
        }

        impl PrimInt for $T {
            #[inline]
            fn count_ones(self) -> u32 {
                self.0.count_ones()
            }

            #[inline]
            fn count_zeros(self) -> u32 {
                self.0.count_zeros()
            }

            #[cfg(has_leading_trailing_ones)]
            #[inline]
            fn leading_ones(self) -> u32 {
                self.0.leading_ones()
            }

            #[inline]
            fn leading_zeros(self) -> u32 {
                self.0.leading_zeros()
            }

            #[cfg(has_leading_trailing_ones)]
            #[inline]
            fn trailing_ones(self) -> u32 {
                self.0.trailing_ones()
            }

            #[inline]
            fn trailing_zeros(self) -> u32 {
                self.0.trailing_zeros()
            }

            #[inline]
            fn rotate_left(self, n: u32) -> Self {
                Self(self.0.rotate_left(n))
            }

            #[inline]
            fn rotate_right(self, n: u32) -> Self {
                Self(self.0.rotate_right(n))
            }

            #[inline]
            fn signed_shl(self, n: u32) -> Self {
                Self(self.0 << n)
            }

            #[inline]
            fn signed_shr(self, n: u32) -> Self {
                Self(self.0 >> n)
            }

            #[inline]
            fn unsigned_shl(self, n: u32) -> Self {
                Self(self.0 << n)
            }

            #[inline]
            fn unsigned_shr(self, n: u32) -> Self {
                Self(self.0 >> n)
            }

            #[inline]
            fn swap_bytes(self) -> Self {
                Self(self.0.swap_bytes())
            }

            #[cfg(has_reverse_bits)]
            #[inline]
            fn reverse_bits(self) -> Self {
                Self(self.0.reverse_bits())
            }

            #[inline]
            fn from_be(x: Self) -> Self {
                Self(AddrInner::from_be(x.0))
            }

            #[inline]
            fn from_le(x: Self) -> Self {
                Self(AddrInner::from_le(x.0))
            }

            #[inline]
            fn to_be(self) -> Self {
                Self(self.0.to_be())
            }

            #[inline]
            fn to_le(self) -> Self {
                Self(self.0.to_le())
            }

            #[inline]
            fn pow(self, exp: u32) -> Self {
                Self(self.0.pow(exp))
            }
        }

        impl Bounded for $T {
            #[inline]
            fn min_value() -> Self {
                Self::MIN
            }

            #[inline]
            fn max_value() -> Self {
                Self(AddrInner::MAX)
            }
        }

        impl Saturating for $T {
            #[inline]
            fn saturating_add(self, v: Self) -> Self {
                Self(self.0.saturating_add(v.0))
            }

            #[inline]
            fn saturating_sub(self, v: Self) -> Self {
                Self(self.0.saturating_sub(v.0))
            }
        }

        impl Not for $T {
            type Output = Self;

            #[inline]
            fn not(self) -> Self::Output {
                Self(!self.0)
            }
        }

        macro_rules! gen_address_bin_op {
            ($op_name: ident, $op: tt) => {
                paste! {
                    impl<I: PrimInt> $op_name<I> for $T {
                        type Output = Self;
                        fn [<$op_name:lower>](self, rhs: I) -> Self::Output {
                            Self(self.0 $op cast::<I, AddrInner>(rhs).unwrap())
                        }
                    }
                    impl<I: PrimInt> [<$op_name Assign>]<I> for $T {
                        fn [<$op_name:lower _assign>](&mut self, rhs: I) {
                            self.0 = self.0 $op cast::<I, AddrInner>(rhs).unwrap();
                        }
                    }
                }
            };
            ($op_name: ident, $op: tt, checked) => {
                gen_address_bin_op!($op_name, $op);
                paste! {
                    impl [<Checked $op_name>] for $T {
                        fn [<checked_ $op_name:lower>](&self, v: &Self) -> Option<Self> {
                            Some(Self(self.0.[<checked_ $op_name:lower>](v.0)?))
                        }
                    }
                }
            };
        }

        gen_address_bin_op!(Add,    +, checked);
        gen_address_bin_op!(Sub,    -, checked);
        gen_address_bin_op!(Mul,    *, checked);
        gen_address_bin_op!(Div,    /, checked);
        gen_address_bin_op!(Rem,    %, checked);
        gen_address_bin_op!(BitAnd, &);
        gen_address_bin_op!(BitOr,  |);
        gen_address_bin_op!(BitXor, ^);
        gen_address_bin_op!(Shl,    <<);
        gen_address_bin_op!(Shr,    >>);
    }
}

// -------------------------------------------------------------------------------------------------

gen_address_type!(AddrPc);
gen_address_type!(AddrSnes);

impl Addr for AddrPc {
    type OppositeAddr = AddrSnes;

    const MIN: Self = AddrPc(0);

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

impl fmt::Display for AddrPc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match AddrSnes::try_from(*self) {
            Ok(snes) => write!(f, "0x{:06x} (SNES: {snes})", self.0),
            Err(_) => write!(f, "0x{:06x}", self.0),
        }
    }
}

impl fmt::Debug for AddrPc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match AddrSnes::try_from(*self) {
            Ok(snes) => write!(f, "AddrPc(0x{:06x} (SNES: {snes}))", self.0),
            Err(_) => write!(f, "AddrPc(0x{:06x})", self.0),
        }
    }
}

impl AddrSnes {
    #[must_use]
    pub fn bank(self) -> u8 {
        (self.0 >> 16) as u8
    }

    #[must_use]
    pub fn high(self) -> u8 {
        ((self.0 & MASK_HH) >> 8) as u8
    }

    #[must_use]
    pub fn low(self) -> u8 {
        (self.0 & MASK_DD) as u8
    }

    #[must_use]
    pub fn absolute(self) -> u16 {
        (self.0 & MASK_HHDD) as u16
    }

    #[must_use]
    pub fn with_bank(self, bank: u8) -> Self {
        Self((self.0 & 0x00FFFF) | ((bank as AddrInner) << 16))
    }

    #[must_use]
    pub fn with_high(self, high: u8) -> Self {
        Self((self.0 & 0xFF00FF) | ((high as AddrInner) << 8))
    }

    #[must_use]
    pub fn with_low(self, low: u8) -> Self {
        Self((self.0 & 0xFFFF00) | (low as AddrInner))
    }

    #[must_use]
    pub fn with_absolute(self, absolute: u16) -> Self {
        Self((self.0 & 0xFF0000) | (absolute as AddrInner))
    }
}

impl Addr for AddrSnes {
    type OppositeAddr = AddrPc;

    const MIN: Self = AddrSnes(0x8000);

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

impl fmt::Display for AddrSnes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:06x}", self.0)
    }
}

impl fmt::Debug for AddrSnes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AddrSnes(0x{:06x})", self.0)
    }
}
