//! The CPU's status register

use std::fmt;

const NEG_FLAG: u8 = 0x80;
const OVERFLOW_FLAG: u8 = 0x40;
/// 1 = Accumulator is 8-bit (native mode only)
const SMALL_ACC_FLAG: u8 = 0x20;
/// 1 = Index registers X/Y are 8-bit (native mode only)
const SMALL_INDEX_FLAG: u8 = 0x10;
/// Emulation mode only (same bit as `SMALL_INDEX_FLAG`)
#[allow(dead_code)] // FIXME Implement or scrap this
const BREAK_FLAG: u8 = 0x10;
const DEC_FLAG: u8 = 0x08;
/// 1 = IRQs disabled
const IRQ_FLAG: u8 = 0x04;
const ZERO_FLAG: u8 = 0x02;
const CARRY_FLAG: u8 = 0x01;

#[derive(Clone)]
pub struct StatusReg(pub u8);

impl StatusReg {
    pub fn new() -> StatusReg {
        // Acc and index regs start in 8-bit mode, IRQs disabled
        StatusReg(SMALL_ACC_FLAG | SMALL_INDEX_FLAG | IRQ_FLAG)
    }

    pub fn negative(&self) -> bool    { self.0 & NEG_FLAG != 0 }
    pub fn overflow(&self) -> bool    { self.0 & OVERFLOW_FLAG != 0 }
    pub fn small_acc(&self) -> bool   { self.0 & SMALL_ACC_FLAG != 0 }
    pub fn small_index(&self) -> bool { self.0 & SMALL_INDEX_FLAG != 0 }
    pub fn decimal(&self) -> bool     { self.0 & DEC_FLAG != 0 }
    pub fn irq_disable(&self) -> bool { self.0 & IRQ_FLAG != 0 }
    pub fn zero(&self) -> bool        { self.0 & ZERO_FLAG != 0}
    pub fn carry(&self) -> bool       { self.0 & CARRY_FLAG != 0 }

    fn set(&mut self, flag: u8, value: bool) {
        if value {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }

    pub fn set_negative(&mut self, value: bool)    { self.set(NEG_FLAG, value) }
    pub fn set_overflow(&mut self, value: bool)    { self.set(OVERFLOW_FLAG, value) }
    pub fn set_small_acc(&mut self, value: bool)   { self.set(SMALL_ACC_FLAG, value) }
    pub fn set_small_index(&mut self, value: bool) { self.set(SMALL_INDEX_FLAG, value) }
    pub fn set_decimal(&mut self, value: bool)     { self.set(DEC_FLAG, value) }
    pub fn set_irq_disable(&mut self, value: bool) { self.set(IRQ_FLAG, value) }
    pub fn set_zero(&mut self, value: bool)        { self.set(ZERO_FLAG, value) }
    pub fn set_carry(&mut self, value: bool)       { self.set(CARRY_FLAG, value) }

    pub fn set_nz(&mut self, val: u16) -> u16 {
        self.set_zero(val == 0);
        self.set_negative(val & 0x8000 != 0);
        val
    }

    pub fn set_nz_8(&mut self, val: u8) -> u8 {
        self.set_zero(val == 0);
        self.set_negative(val & 0x80 != 0);
        val
    }
}

impl fmt::Display for StatusReg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(if self.negative() { "N" } else { "-" })?;
        f.write_str(if self.overflow() { "V" } else { "-" })?;
        f.write_str(if self.small_acc() { "M" } else { "-" })?;
        f.write_str(if self.small_index() { "X" } else { "-" })?;
        f.write_str(if self.decimal() { "D" } else { "-" })?;
        f.write_str(if self.irq_disable() { "I" } else { "-" })?;
        f.write_str(if self.zero() { "Z" } else { "-" })?;
        f.write_str(if self.carry() { "C" } else { "-" })?;

        Ok(())
    }
}
