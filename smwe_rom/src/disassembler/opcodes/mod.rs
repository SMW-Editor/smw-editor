mod data;

use std::fmt;

pub use data::SNES_OPCODES;
use AddressingMode::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum AddressingMode {
    Accumulator,
    Address,
    AddressIndirect,
    AddressLongIndirect,
    AddressXIndex,
    AddressYIndex,
    AddressXIndexIndirect,
    BlockMove,
    Constant8,
    DirectPage,
    DirectPageIndirect,
    DirectPageIndirectYIndex,
    DirectPageLongIndirect,
    DirectPageLongIndirectYIndex,
    DirectPageXIndex,
    DirectPageXIndexIndirect,
    DirectPageYIndex,
    DirectPageSIndex,
    DirectPageSIndexIndirectYIndex,
    Implied,
    Immediate8,
    Immediate16,
    ImmediateXFlagDependent,
    ImmediateMFlagDependent,
    Long,
    LongXIndex,
    Relative8,
    Relative16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mnemonic {
    /// Add with carry
    ADC,
    /// AND Accumulator
    AND,
    /// Left-shift Accumulator
    ASL,
    /// Branch if carry clear
    BCC,
    /// Branch if carry set
    BCS,
    /// Branch if equal
    BEQ,
    /// Bit test
    BIT,
    /// Branch if minus
    BMI,
    /// Branch if not equal
    BNE,
    /// Branch if plus
    BPL,
    /// Branch always
    BRA,
    /// Break to instruction
    BRK,
    /// Branch relative long
    BRL,
    /// Branch if overflow clear
    BVC,
    /// Branch if overflow set
    BVS,
    /// Clear carry flag
    CLC,
    /// Clear decimal flag
    CLD,
    /// Clear interrupt flag
    CLI,
    /// Clear overflow flag
    CLV,
    /// Compare Accumulator with memory
    CMP,
    /// Compare X with memory
    CPX,
    /// Compare Y with memory
    CPY,
    /// Coprocessor Empowerment
    COP,
    /// Decrement Accumulator
    DEC,
    /// Decrement X
    DEX,
    /// Decrement Y
    DEY,
    /// Exclusive-OR Accumulator
    EOR,
    /// Increment Accumulator
    INC,
    /// Increment X
    INX,
    /// Increment Y
    INY,
    /// Jump to location
    JMP,
    /// Jump long
    JML,
    /// Jump subroutine
    JSR,
    /// Jump subroutine long
    JSL,
    /// Load Accumulator with memory
    LDA,
    /// Load X with memory
    LDX,
    /// Load Y with memory
    LDY,
    /// Right-shift Accumulator or memory
    LSR,
    /// Block move negative
    MVN,
    /// Block move positive
    MVP,
    /// No operation
    NOP,
    /// OR Accumulator with memory
    ORA,
    /// Push effective address
    PEA,
    /// Push effective indirect address
    PEI,
    /// Push program counter relative
    PER,
    /// Push Accumulator
    PHA,
    /// Push Data Bank Register
    PHB,
    /// Push Direct Page Register
    PHD,
    /// Push Program Bank
    PHK,
    /// Push Processor Status
    PHP,
    /// Push X
    PHX,
    /// Push Y
    PHY,
    /// Pull Accumulator
    PLA,
    /// Pull Data Bank Register
    PLB,
    /// Pull Direct Page Register
    PLD,
    /// Pull flags
    PLP,
    /// Pull X
    PLX,
    /// Pull Y
    PLY,
    /// Reset flag
    REP,
    /// Rotate bit left
    ROL,
    /// Rotate bit right
    ROR,
    /// Return from interrupt
    RTI,
    /// Return from subroutine
    RTS,
    /// Return from subroutine long
    RTL,
    /// Subtract with carry
    SBC,
    /// Set carry flag
    SEC,
    /// Set decimal flag
    SED,
    /// Set interrupt flag
    SEI,
    /// Set flag
    SEP,
    /// Store Accumulator to memory
    STA,
    /// Store X to memory
    STX,
    /// Store Y to memory
    STY,
    /// Stop the clock
    STP,
    /// Store zero to memory
    STZ,
    /// Transfer Accumulator to X
    TAX,
    /// Transfer Accumulator to Y
    TAY,
    /// Transfer Accumulator to Direct Page
    TCD,
    /// Transfer Accumulator to Stack
    TCS,
    /// Transfer Direct Page to Accumulator
    TDC,
    /// Transfer Stack to Accumulator
    TSC,
    /// Transfer Stack to X
    TSX,
    /// Transfer X to Accumulator
    TXA,
    /// Transfer X to Stack
    TXS,
    /// Transfer X to Y
    TXY,
    /// Transfer Y to Accumulator
    TYA,
    /// Transfer Y to X
    TYX,
    /// Test and reset bit
    TRB,
    /// Test and set bit
    TSB,
    /// Wait for interrupt
    WAI,
    /// (Reserved for future expansion)
    WDM,
    /// Exchange B with A (bytes in Accumulator)
    XBA,
    /// Exchange Carry with Emulation
    XCE,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Opcode {
    pub mnemonic: Mnemonic,
    pub mode:     AddressingMode,
}

// -------------------------------------------------------------------------------------------------

impl Mnemonic {
    pub fn can_branch(self) -> bool {
        use Mnemonic::*;
        [BCC, BCS, BEQ, BMI, BNE, BRK, BPL, BRA, BRL, BVC, BVS, COP, JMP, JML, JSR, JSL, RTI, RTS, RTL].contains(&self)
    }

    pub fn is_subroutine_call(self) -> bool {
        use Mnemonic::*;
        [JSR, JSL].contains(&self)
    }

    pub fn is_subroutine_return(self) -> bool {
        use Mnemonic::*;
        [RTS, RTL].contains(&self)
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl AddressingMode {
    #[inline]
    pub fn operands_size(self) -> usize {
        match self {
            Accumulator | Implied => 0,
            Long | LongXIndex => 3,
            Immediate16 | Relative16 | BlockMove => 2,
            m if (Address..=AddressXIndexIndirect).contains(&m) => 2,
            ImmediateXFlagDependent | ImmediateMFlagDependent => {
                // These two modes must be replaced with Immediate8 or Immediate16,
                // depending on the X and M flags.
                unreachable!()
            }
            _ => 1,
        }
    }
}

impl Opcode {
    pub const fn new(mnemonic: Mnemonic, mode: AddressingMode) -> Self {
        Self { mnemonic, mode }
    }

    #[inline]
    pub fn instruction_size(self) -> usize {
        1 + self.mode.operands_size()
    }
}
