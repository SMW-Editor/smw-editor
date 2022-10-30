use std::{fmt, fmt::Formatter};

use crate::snes_utils::addr::{Addr, AddrPc, AddrSnes};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RomSlice<A: Addr> {
    pub begin: A,
    pub size:  usize,
}

pub type PcSlice = RomSlice<AddrPc>;
pub type SnesSlice = RomSlice<AddrSnes>;

impl PcSlice {
    pub const fn new(begin: AddrPc, size: usize) -> Self {
        Self { begin, size }
    }
}

impl SnesSlice {
    pub const fn new(begin: AddrSnes, size: usize) -> Self {
        Self { begin, size }
    }
}

impl<A: Addr> RomSlice<A> {
    pub fn end(&self) -> Option<A> {
        if self.is_infinite() {
            None
        } else {
            Some(self.begin + self.size)
        }
    }

    #[must_use]
    pub fn offset_forward(self, offset: usize) -> Self {
        Self { begin: self.begin + offset, ..self }
    }

    #[must_use]
    pub fn offset_backward(self, offset: usize) -> Self {
        Self { begin: self.begin - offset, ..self }
    }

    #[must_use]
    pub fn skip_forward(self, lengths: usize) -> Self {
        if self.is_infinite() {
            self
        } else {
            Self { begin: self.begin + (self.size * lengths), ..self }
        }
    }

    #[must_use]
    pub fn skip_backward(self, lengths: usize) -> Self {
        if self.is_infinite() {
            self
        } else {
            Self { begin: self.begin - (self.size * lengths), ..self }
        }
    }

    #[must_use]
    pub fn move_to(self, new_address: A) -> Self {
        Self { begin: new_address, ..self }
    }

    #[must_use]
    pub fn expand(self, diff: usize) -> Self {
        if self.is_infinite() {
            self
        } else {
            Self { size: self.size + diff, ..self }
        }
    }

    #[must_use]
    pub fn shrink(self, diff: usize) -> Self {
        Self { size: self.size - diff, ..self }
    }

    #[must_use]
    pub fn resize(self, new_size: usize) -> Self {
        Self { size: new_size, ..self }
    }

    #[must_use]
    pub fn infinite(self) -> Self {
        Self { size: usize::MAX, ..self }
    }

    pub fn is_infinite(&self) -> bool {
        self.size == usize::MAX
    }

    pub fn contains(&self, addr: A) -> bool {
        if let Some(end) = self.end() {
            (self.begin..end).contains(&addr)
        } else {
            false
        }
    }
}

impl<A: Addr> fmt::Display for RomSlice<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "RomSlice {{ begin: {:X}, size: {} }}", self.begin, self.size)
    }
}
