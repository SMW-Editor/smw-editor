// This module will probably not define every register of the 65816,
// only those that are needed for disassembly.

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PRegister(pub u8);

impl PRegister {
    /// Negative
    pub fn n_flag(&self) -> bool {
        (self.0 & 0b10000000) != 0
    }

    /// Overflow
    pub fn v_flag(&self) -> bool {
        (self.0 & 0b01000000) != 0
    }

    /// Accumulator register size (0 = 16 bit, 1 = 8 bit)
    pub fn m_flag(&self) -> bool {
        (self.0 & 0b00100000) != 0
    }

    /// Index register size (0 = 16 bit, 1 = 8 bit)
    pub fn x_flag(&self) -> bool {
        (self.0 & 0b00010000) != 0
    }

    /// Decimal
    pub fn d_flag(&self) -> bool {
        (self.0 & 0b00001000) != 0
    }

    /// IRQ disable
    pub fn i_flag(&self) -> bool {
        (self.0 & 0b00000100) != 0
    }

    /// Zero
    pub fn z_flag(&self) -> bool {
        (self.0 & 0b00000010) != 0
    }

    /// Carry
    pub fn c_flag(&self) -> bool {
        (self.0 & 0b00000001) != 0
    }
}
