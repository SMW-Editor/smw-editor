use crate::snes_utils::addr::{Addr, AddrPc, AddrSnes};

#[derive(Copy, Clone, Debug)]
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
    pub fn end(&self) -> A {
        self.begin + self.size
    }

    pub fn shift_forward(&self, offset: usize) -> Self {
        Self { begin: self.begin + offset, size: self.size }
    }

    pub fn shift_backward(&self, offset: usize) -> Self {
        Self { begin: self.begin - offset, size: self.size }
    }

    pub fn skip_forward(&self, times_size: usize) -> Self {
        Self { begin: self.begin + (self.size * times_size), size: self.size }
    }

    pub fn skip_backward(&self, times_size: usize) -> Self {
        Self { begin: self.begin - (self.size * times_size), size: self.size }
    }

    pub fn expand(&self, new_size: usize) -> Self {
        Self { begin: self.begin, size: self.size + new_size }
    }

    pub fn shrink(&self, new_size: usize) -> Self {
        Self { begin: self.begin, size: self.size - new_size }
    }
}
