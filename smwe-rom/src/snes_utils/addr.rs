use std::{convert::TryFrom, fmt, num::ParseIntError, ops::*};

use duplicate::*;
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

/// Bank
pub const MASK_BB: u32 = 0xFF0000;
/// High byte
pub const MASK_HH: u32 = 0x00FF00;
/// Low byte
pub const MASK_DD: u32 = 0x0000FF;
/// Absolute address
pub const MASK_HHDD: u32 = MASK_HH | MASK_DD;
/// Long address
pub const MASK_BBHHDD: u32 = MASK_BB | MASK_HH | MASK_DD;

// -------------------------------------------------------------------------------------------------

pub trait Addr: Clone + NumOps<usize, Self> + PrimInt + fmt::LowerHex + fmt::UpperHex {
    const MIN: Self;
}

// -------------------------------------------------------------------------------------------------

duplicate! {
    [
        addr_type   inner   fmt_lower_hex   fmt_upper_hex;
        [AddrPc]    [u32]   ["PC {:#x}"]    ["PC {:#X}"];
        [AddrSnes]  [u32]   ["SNES ${:x}"]  ["SNES ${:X}"];
        [AddrVram]  [u16]   ["VRAM ${:x}"]  ["VRAM ${:X}"];
    ]

    #[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct addr_type(pub inner);

    impl addr_type {
        #[inline]
        pub fn as_index(self) -> usize {
            self.0 as usize
        }
    }

    impl Default for addr_type {
        #[inline]
        fn default() -> Self {
            Self::MIN
        }
    }

    impl fmt::LowerHex for addr_type {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, fmt_lower_hex, self.0)
        }
    }

    impl fmt::UpperHex for addr_type {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, fmt_upper_hex, self.0)
        }
    }

    impl Zero for addr_type {
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

    impl One for addr_type {
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

    impl Num for addr_type {
        type FromStrRadixErr = ParseIntError;

        #[inline]
        fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
            Ok(Self(inner::from_str_radix(str, radix)?))
        }
    }

    impl NumCast for addr_type {
        #[inline]
        fn from<T: ToPrimitive>(n: T) -> Option<Self> {
            Some(Self(n.to_u32()? as _))
        }
    }

    impl ToPrimitive for addr_type {
        #[inline]
        fn to_i64(&self) -> Option<i64> {
            cast::<inner, i64>(self.0)
        }

        #[inline]
        fn to_u64(&self) -> Option<u64> {
            cast::<inner, u64>(self.0)
        }
    }

    impl PrimInt for addr_type {
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
            Self(inner::from_be(x.0))
        }

        #[inline]
        fn from_le(x: Self) -> Self {
            Self(inner::from_le(x.0))
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

    impl Bounded for addr_type {
        #[inline]
        fn min_value() -> Self {
            Self::MIN
        }

        #[inline]
        fn max_value() -> Self {
            Self(inner::MAX)
        }
    }

    impl Saturating for addr_type {
        #[inline]
        fn saturating_add(self, v: Self) -> Self {
            Self(self.0.saturating_add(v.0))
        }

        #[inline]
        fn saturating_sub(self, v: Self) -> Self {
            Self(self.0.saturating_sub(v.0))
        }
    }

    impl Not for addr_type {
        type Output = Self;

        #[inline]
        fn not(self) -> Self::Output {
            Self(!self.0)
        }
    }

    #[duplicate_item(
        op_name     op;
        [Add]       [+];
        [Sub]       [-];
        [Mul]       [*];
        [Div]       [/];
        [Rem]       [%];
        [BitAnd]    [&];
        [BitOr]     [|];
        [BitXor]    [^];
        [Shl]       [<<];
        [Shr]       [>>];
    )]
    paste! {
        impl<I: PrimInt> op_name<I> for addr_type {
            type Output = Self;
            fn [<op_name:lower>](self, rhs: I) -> Self::Output {
                Self(self.0 op cast::<I, inner>(rhs).unwrap())
            }
        }
        impl<I: PrimInt> [<op_name Assign>]<I> for addr_type {
            fn [<op_name:lower _assign>](&mut self, rhs: I) {
                self.0 = self.0 op cast::<I, inner>(rhs).unwrap();
            }
        }
    }

    #[duplicate_item(
        op_name            op;
        [CheckedAdd]       [+];
        [CheckedSub]       [-];
        [CheckedMul]       [*];
        [CheckedDiv]       [/];
        [CheckedRem]       [%];
    )]
    impl op_name for addr_type {
        paste! {
            fn [<op_name:snake>](&self, v: &Self) -> Option<Self> {
                Some(Self(self.0.[<op_name:snake>](v.0)?))
            }
        }
    }
}

duplicate! {
    [
        addr_type   opposite_type;
        [AddrPc]    [AddrSnes];
        [AddrSnes]  [AddrPc];
    ]
    impl fmt::Debug for addr_type {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match opposite_type::try_from(*self) {
                Ok(opposite) => write!(f, "{}({:#06x}) [-> {opposite:X}]", stringify!(addr_type), self.0),
                Err(_) => write!(f, "{}({:#06x})", stringify!(addr_type), self.0),
            }
        }
    }

    impl fmt::Display for addr_type {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match opposite_type::try_from(*self) {
                Ok(opposite) => write!(f, "{:#06x} [-> {opposite:X}]", self.0),
                Err(_) => write!(f, "{:#06x}", self.0),
            }
        }
    }

    impl TryFrom<opposite_type> for addr_type {
        type Error = AddressError;

        fn try_from(value: opposite_type) -> Result<Self, Self::Error> {
            Self::try_from_lorom(value)
        }
    }
}

// -------------------------------------------------------------------------------------------------

impl Addr for AddrPc {
    const MIN: Self = AddrPc(0);
}

impl AddrPc {
    pub fn try_from_lorom(addr: AddrSnes) -> Result<Self, AddressError> {
        if addr.is_valid_lorom() {
            Ok(Self(((addr.0 & 0x7F0000) >> 1) | (addr.0 & 0x7FFF)))
        } else {
            Err(AddressError::InvalidSnesLoRom(addr))
        }
    }

    pub fn try_from_hirom(addr: AddrSnes) -> Result<Self, AddressError> {
        if addr.is_valid_hirom() {
            Ok(Self(addr.0 & 0x3FFFFF))
        } else {
            Err(AddressError::InvalidSnesHiRom(addr))
        }
    }

    pub fn is_valid_lorom(&self) -> bool {
        self.0 < 0x400000
    }

    pub fn is_valid_hirom(&self) -> bool {
        self.0 < 0x400000
    }
}

impl Addr for AddrSnes {
    const MIN: Self = AddrSnes(0x8000);
}

impl AddrSnes {
    pub fn try_from_lorom(addr: AddrPc) -> Result<Self, AddressError> {
        if addr.is_valid_lorom() {
            Ok(Self(((addr.0 << 1) & 0x7F0000) | (addr.0 & 0x7FFF) | 0x8000))
        } else {
            Err(AddressError::InvalidPcLoRom(addr))
        }
    }

    pub fn try_from_hirom(addr: AddrPc) -> Result<Self, AddressError> {
        if addr.is_valid_hirom() {
            Ok(Self(addr.0 | 0xC00000))
        } else {
            Err(AddressError::InvalidPcHiRom(addr))
        }
    }

    pub fn is_valid_lorom(&self) -> bool {
        let wram = (self.0 & 0xFE0000) == 0x7E0000;
        let junk = (self.0 & 0x408000) == 0x000000;
        let sram = (self.0 & 0x708000) == 0x700000;
        !wram && !junk && !sram
    }

    pub fn is_valid_hirom(&self) -> bool {
        let wram = (self.0 & 0xFE0000) == 0x7E0000;
        let junk = (self.0 & 0x408000) == 0x000000;
        !wram && !junk
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
        Self((self.0 & 0x00FFFF) | ((bank as u32) << 16))
    }

    #[must_use]
    pub fn with_high(self, high: u8) -> Self {
        Self((self.0 & 0xFF00FF) | ((high as u32) << 8))
    }

    #[must_use]
    pub fn with_low(self, low: u8) -> Self {
        Self((self.0 & 0xFFFF00) | (low as u32))
    }

    #[must_use]
    pub fn with_absolute(self, absolute: u16) -> Self {
        Self((self.0 & 0xFF0000) | (absolute as u32))
    }
}

impl Addr for AddrVram {
    const MIN: Self = Self(0);
}
