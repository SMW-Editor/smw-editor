use std::fmt;

use AddressingMode::*;
use Mnemonic::*;

// -------------------------------------------------------------------------------------------------

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
    ADC, // Add with carry
    AND, // AND Accumulator
    ASL, // Left-shift Accumulator
    BCC, // Branch if carry clear
    BCS, // Branch if carry set
    BEQ, // Branch if equal
    BIT, // Bit test
    BMI, // Branch if minus
    BNE, // Branch if not equal
    BPL, // Branch if plus
    BRA, // Branch always
    BRK, // Break to instruction
    BRL, // Branch relative long
    BVC, // Branch if overflow clear
    BVS, // Branch if overflow set
    CLC, // Clear carry flag
    CLD, // Clear decimal flag
    CLI, // Clear interrupt flag
    CLV, // Clear overflow flag
    CMP, // Compare Accumulator with memory
    CPX, // Compare X with memory
    CPY, // Compare Y with memory
    COP, // Coprocessor Empowerment
    DEC, // Decrement Accumulator
    DEX, // Decrement X
    DEY, // Decrement Y
    EOR, // Exclusive-OR Accumulator
    INC, // Increment Accumulator
    INX, // Increment X
    INY, // Increment Y
    JMP, // Jump to location
    JML, // Jump long
    JSR, // Jump subroutine
    JSL, // Jump subroutine long
    LDA, // Load Accumulator with memory
    LDX, // Load X with memory
    LDY, // Load Y with memory
    LSR, // Right-shift Accumulator or memory
    MVN, // Block move negative
    MVP, // Block move positive
    NOP, // No operation
    ORA, // OR Accumulator with memory
    PEA, // Push effective address
    PEI, // Push effective indirect address
    PER, // Push program counter relative
    PHA, // Push Accumulator
    PHB, // Push Data Bank Register
    PHD, // Push Direct Page Register
    PHK, // Push Program Bank
    PHP, // Push Processor Status
    PHX, // Push X
    PHY, // Push Y
    PLA, // Pull Accumulator
    PLB, // Pull Data Bank Register
    PLD, // Pull Direct Page Register
    PLP, // Pull flags
    PLX, // Pull X
    PLY, // Pull Y
    REP, // Reset flag
    ROL, // Rotate bit left
    ROR, // Rotate bit right
    RTI, // Return from interrupt
    RTS, // Return from subroutine
    RTL, // Return from subroutine long
    SBC, // Subtract with carry
    SEC, // Set carry flag
    SED, // Set decimal flag
    SEI, // Set interrupt flag
    SEP, // Set flag
    STA, // Store Accumulator to memory
    STX, // Store X to memory
    STY, // Store Y to memory
    STP, // Stop the clock
    STZ, // Store zero to memory
    TAX, // Transfer Accumulator to X
    TAY, // Transfer Accumulator to Y
    TCD, // Transfer Accumulator to Direct Page
    TCS, // Transfer Accumulator to Stack
    TDC, // Transfer Direct Page to Accumulator
    TSC, // Transfer Stack to Accumulator
    TSX, // Transfer Stack to X
    TXA, // Transfer X to Accumulator
    TXS, // Transfer X to Stack
    TXY, // Transfer X to Y
    TYA, // Transfer Y to Accumulator
    TYX, // Transfer Y to X
    TRB, // Test and reset bit
    TSB, // Test and set bit
    WAI, // Wait for interrupt
    WDM, // (Reserved for future expansion)
    XBA, // Exchange B with A (bytes in Accumulator)
    XCE, // Exchange Carry with Emulation
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Opcode {
    pub mnemonic: Mnemonic,
    pub mode:     AddressingMode,
}

// -------------------------------------------------------------------------------------------------

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl AddressingMode {
    #[inline]
    pub fn operands_size(self) -> usize {
        match self {
            Accumulator | Implied | ImmediateXFlagDependent | ImmediateMFlagDependent => 0,
            Long | LongXIndex => 3,
            Immediate16 | Relative16 | BlockMove => 2,
            m if (Address..=AddressXIndexIndirect).contains(&m) => 2,
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

// -------------------------------------------------------------------------------------------------

/// Ordered by hex values of opcodes
pub static SNES_OPCODES: [Opcode; 0x100] = [
    /* 00 */ Opcode::new(BRK, Constant8),
    /* 01 */ Opcode::new(ORA, DirectPageXIndexIndirect),
    /* 02 */ Opcode::new(COP, Constant8),
    /* 03 */ Opcode::new(ORA, DirectPageSIndex),
    /* 04 */ Opcode::new(TSB, DirectPage),
    /* 05 */ Opcode::new(ORA, DirectPage),
    /* 06 */ Opcode::new(ASL, DirectPage),
    /* 07 */ Opcode::new(ORA, DirectPageLongIndirect),
    /* 08 */ Opcode::new(PHP, Implied),
    /* 09 */ Opcode::new(ORA, ImmediateMFlagDependent),
    /* 0A */ Opcode::new(ASL, Accumulator),
    /* 0B */ Opcode::new(PHD, Implied),
    /* 0C */ Opcode::new(TSB, Address),
    /* 0D */ Opcode::new(ORA, Address),
    /* 0E */ Opcode::new(ASL, Address),
    /* 0F */ Opcode::new(ORA, Long),
    /* 10 */ Opcode::new(BPL, Relative8),
    /* 11 */ Opcode::new(ORA, DirectPageIndirectYIndex),
    /* 12 */ Opcode::new(ORA, DirectPageIndirect),
    /* 13 */ Opcode::new(ORA, DirectPageSIndexIndirectYIndex),
    /* 14 */ Opcode::new(TRB, DirectPage),
    /* 15 */ Opcode::new(ORA, DirectPageXIndex),
    /* 16 */ Opcode::new(ASL, DirectPageXIndex),
    /* 17 */ Opcode::new(ORA, DirectPageLongIndirectYIndex),
    /* 18 */ Opcode::new(CLC, Implied),
    /* 19 */ Opcode::new(ORA, AddressYIndex),
    /* 1A */ Opcode::new(INC, Accumulator),
    /* 1B */ Opcode::new(TCS, Implied),
    /* 1C */ Opcode::new(TRB, Address),
    /* 1D */ Opcode::new(ORA, AddressXIndex),
    /* 1E */ Opcode::new(ASL, AddressXIndex),
    /* 1F */ Opcode::new(ORA, LongXIndex),
    /* 20 */ Opcode::new(JSR, Address),
    /* 21 */ Opcode::new(AND, DirectPageXIndexIndirect),
    /* 22 */ Opcode::new(JSL, Long),
    /* 23 */ Opcode::new(AND, DirectPageSIndex),
    /* 24 */ Opcode::new(BIT, DirectPage),
    /* 25 */ Opcode::new(AND, DirectPage),
    /* 26 */ Opcode::new(ROL, DirectPage),
    /* 27 */ Opcode::new(AND, DirectPageLongIndirect),
    /* 28 */ Opcode::new(PLP, Implied),
    /* 29 */ Opcode::new(AND, ImmediateMFlagDependent),
    /* 2A */ Opcode::new(ROL, Accumulator),
    /* 2B */ Opcode::new(PLD, Implied),
    /* 2C */ Opcode::new(BIT, Address),
    /* 2D */ Opcode::new(AND, Address),
    /* 2E */ Opcode::new(ROL, Address),
    /* 2F */ Opcode::new(AND, Long),
    /* 30 */ Opcode::new(BMI, Relative8),
    /* 31 */ Opcode::new(AND, DirectPageIndirectYIndex),
    /* 32 */ Opcode::new(AND, DirectPageIndirect),
    /* 33 */ Opcode::new(AND, DirectPageSIndexIndirectYIndex),
    /* 34 */ Opcode::new(BIT, DirectPageXIndex),
    /* 35 */ Opcode::new(AND, DirectPageXIndex),
    /* 36 */ Opcode::new(ROL, DirectPageXIndex),
    /* 37 */ Opcode::new(AND, DirectPageLongIndirectYIndex),
    /* 38 */ Opcode::new(SEC, Implied),
    /* 39 */ Opcode::new(AND, AddressYIndex),
    /* 3A */ Opcode::new(DEC, Accumulator),
    /* 3B */ Opcode::new(TSC, Implied),
    /* 3C */ Opcode::new(BIT, AddressXIndex),
    /* 3D */ Opcode::new(AND, AddressXIndex),
    /* 3E */ Opcode::new(ROL, AddressXIndex),
    /* 3F */ Opcode::new(AND, LongXIndex),
    /* 40 */ Opcode::new(RTI, Implied),
    /* 41 */ Opcode::new(EOR, DirectPageXIndexIndirect),
    /* 42 */ Opcode::new(WDM, Constant8),
    /* 43 */ Opcode::new(EOR, DirectPageSIndex),
    /* 44 */ Opcode::new(MVP, BlockMove),
    /* 45 */ Opcode::new(EOR, DirectPage),
    /* 46 */ Opcode::new(LSR, DirectPage),
    /* 47 */ Opcode::new(EOR, DirectPageLongIndirect),
    /* 48 */ Opcode::new(PHA, Implied),
    /* 49 */ Opcode::new(EOR, ImmediateMFlagDependent),
    /* 4A */ Opcode::new(LSR, Accumulator),
    /* 4B */ Opcode::new(PHK, Implied),
    /* 4C */ Opcode::new(JMP, Address),
    /* 4D */ Opcode::new(EOR, Address),
    /* 4E */ Opcode::new(LSR, Address),
    /* 4F */ Opcode::new(EOR, Long),
    /* 50 */ Opcode::new(BVC, Relative8),
    /* 51 */ Opcode::new(EOR, DirectPageIndirectYIndex),
    /* 52 */ Opcode::new(EOR, DirectPageIndirect),
    /* 53 */ Opcode::new(EOR, DirectPageSIndexIndirectYIndex),
    /* 54 */ Opcode::new(MVN, BlockMove),
    /* 55 */ Opcode::new(EOR, DirectPageXIndex),
    /* 56 */ Opcode::new(LSR, DirectPageXIndex),
    /* 57 */ Opcode::new(EOR, DirectPageLongIndirectYIndex),
    /* 58 */ Opcode::new(CLI, Implied),
    /* 59 */ Opcode::new(EOR, AddressYIndex),
    /* 5A */ Opcode::new(PHY, Implied),
    /* 5B */ Opcode::new(TCD, Implied),
    /* 5C */ Opcode::new(JML, Long),
    /* 5D */ Opcode::new(EOR, AddressXIndex),
    /* 5E */ Opcode::new(LSR, AddressXIndex),
    /* 5F */ Opcode::new(EOR, LongXIndex),
    /* 60 */ Opcode::new(RTS, Implied),
    /* 61 */ Opcode::new(ADC, DirectPageXIndexIndirect),
    /* 62 */ Opcode::new(PER, Relative16),
    /* 63 */ Opcode::new(ADC, DirectPageSIndex),
    /* 64 */ Opcode::new(STZ, DirectPage),
    /* 65 */ Opcode::new(ADC, DirectPage),
    /* 66 */ Opcode::new(ROR, DirectPage),
    /* 67 */ Opcode::new(ADC, DirectPageLongIndirect),
    /* 68 */ Opcode::new(PLA, Implied),
    /* 69 */ Opcode::new(ADC, ImmediateMFlagDependent),
    /* 6A */ Opcode::new(ROR, Accumulator),
    /* 6B */ Opcode::new(RTL, Implied),
    /* 6C */ Opcode::new(JMP, AddressIndirect),
    /* 6D */ Opcode::new(ADC, Address),
    /* 6E */ Opcode::new(ROR, Address),
    /* 6F */ Opcode::new(ADC, Long),
    /* 70 */ Opcode::new(BVS, Relative8),
    /* 71 */ Opcode::new(ADC, DirectPageIndirectYIndex),
    /* 72 */ Opcode::new(ADC, DirectPageIndirect),
    /* 73 */ Opcode::new(ADC, DirectPageSIndexIndirectYIndex),
    /* 74 */ Opcode::new(STZ, DirectPageXIndex),
    /* 75 */ Opcode::new(ADC, DirectPageXIndex),
    /* 76 */ Opcode::new(ROR, DirectPageXIndex),
    /* 77 */ Opcode::new(ADC, DirectPageLongIndirectYIndex),
    /* 78 */ Opcode::new(SEI, Implied),
    /* 79 */ Opcode::new(ADC, AddressYIndex),
    /* 7A */ Opcode::new(PLY, Implied),
    /* 7B */ Opcode::new(TDC, Implied),
    /* 7C */ Opcode::new(JMP, AddressXIndexIndirect),
    /* 7D */ Opcode::new(ADC, AddressXIndex),
    /* 7E */ Opcode::new(ROR, AddressXIndex),
    /* 7F */ Opcode::new(ADC, LongXIndex),
    /* 80 */ Opcode::new(BRA, Relative8),
    /* 81 */ Opcode::new(STA, DirectPageXIndexIndirect),
    /* 82 */ Opcode::new(BRL, Relative16),
    /* 83 */ Opcode::new(STA, DirectPageSIndex),
    /* 84 */ Opcode::new(STY, DirectPage),
    /* 85 */ Opcode::new(STA, DirectPage),
    /* 86 */ Opcode::new(STX, DirectPage),
    /* 87 */ Opcode::new(STA, DirectPageLongIndirect),
    /* 88 */ Opcode::new(DEY, Implied),
    /* 89 */ Opcode::new(BIT, ImmediateMFlagDependent),
    /* 8A */ Opcode::new(TXA, Implied),
    /* 8B */ Opcode::new(PHB, Implied),
    /* 8C */ Opcode::new(STY, Address),
    /* 8D */ Opcode::new(STA, Address),
    /* 8E */ Opcode::new(STX, Address),
    /* 8F */ Opcode::new(STA, Long),
    /* 90 */ Opcode::new(BCC, Relative8),
    /* 91 */ Opcode::new(STA, DirectPageIndirectYIndex),
    /* 92 */ Opcode::new(STA, DirectPageIndirect),
    /* 93 */ Opcode::new(STA, DirectPageSIndexIndirectYIndex),
    /* 94 */ Opcode::new(STY, DirectPageXIndex),
    /* 95 */ Opcode::new(STA, DirectPageXIndex),
    /* 96 */ Opcode::new(STX, DirectPageYIndex),
    /* 97 */ Opcode::new(STA, DirectPageLongIndirectYIndex),
    /* 98 */ Opcode::new(TYA, Implied),
    /* 99 */ Opcode::new(STA, AddressYIndex),
    /* 9A */ Opcode::new(TXS, Implied),
    /* 9B */ Opcode::new(TXY, Implied),
    /* 9C */ Opcode::new(STZ, Address),
    /* 9D */ Opcode::new(STA, AddressXIndex),
    /* 9E */ Opcode::new(STZ, AddressXIndex),
    /* 9F */ Opcode::new(STA, LongXIndex),
    /* A0 */ Opcode::new(LDY, ImmediateXFlagDependent),
    /* A1 */ Opcode::new(LDA, DirectPageXIndexIndirect),
    /* A2 */ Opcode::new(LDX, ImmediateXFlagDependent),
    /* A3 */ Opcode::new(LDA, DirectPageSIndex),
    /* A4 */ Opcode::new(LDY, DirectPage),
    /* A5 */ Opcode::new(LDA, DirectPage),
    /* A6 */ Opcode::new(LDX, DirectPage),
    /* A7 */ Opcode::new(LDA, DirectPageLongIndirect),
    /* A8 */ Opcode::new(TAY, Implied),
    /* A9 */ Opcode::new(LDA, ImmediateMFlagDependent),
    /* AA */ Opcode::new(TAX, Implied),
    /* AB */ Opcode::new(PLB, Implied),
    /* AC */ Opcode::new(LDY, Address),
    /* AD */ Opcode::new(LDA, Address),
    /* AE */ Opcode::new(LDX, Address),
    /* AF */ Opcode::new(LDA, Long),
    /* B0 */ Opcode::new(BCS, Relative8),
    /* B1 */ Opcode::new(LDA, DirectPageIndirectYIndex),
    /* B2 */ Opcode::new(LDA, DirectPageIndirect),
    /* B3 */ Opcode::new(LDA, DirectPageSIndexIndirectYIndex),
    /* B4 */ Opcode::new(LDY, DirectPageXIndex),
    /* B5 */ Opcode::new(LDA, DirectPageXIndex),
    /* B6 */ Opcode::new(LDX, DirectPageYIndex),
    /* B7 */ Opcode::new(LDA, DirectPageLongIndirectYIndex),
    /* B8 */ Opcode::new(CLV, Implied),
    /* B9 */ Opcode::new(LDA, AddressYIndex),
    /* BA */ Opcode::new(TSX, Implied),
    /* BB */ Opcode::new(TYX, Implied),
    /* BC */ Opcode::new(LDY, AddressXIndex),
    /* BD */ Opcode::new(LDA, AddressXIndex),
    /* BE */ Opcode::new(LDX, AddressYIndex),
    /* BF */ Opcode::new(LDA, LongXIndex),
    /* C0 */ Opcode::new(CPY, ImmediateXFlagDependent),
    /* C1 */ Opcode::new(CMP, DirectPageXIndexIndirect),
    /* C2 */ Opcode::new(REP, Constant8),
    /* C3 */ Opcode::new(CMP, DirectPageSIndex),
    /* C4 */ Opcode::new(CPY, DirectPage),
    /* C5 */ Opcode::new(CMP, DirectPage),
    /* C6 */ Opcode::new(DEC, DirectPage),
    /* C7 */ Opcode::new(CMP, DirectPageLongIndirect),
    /* C8 */ Opcode::new(INY, Implied),
    /* C9 */ Opcode::new(CMP, ImmediateMFlagDependent),
    /* CA */ Opcode::new(DEX, Implied),
    /* CB */ Opcode::new(WAI, Implied),
    /* CC */ Opcode::new(CPY, Address),
    /* CD */ Opcode::new(CMP, Address),
    /* CE */ Opcode::new(DEC, Address),
    /* CF */ Opcode::new(CMP, Long),
    /* D0 */ Opcode::new(BNE, Relative8),
    /* D1 */ Opcode::new(CMP, DirectPageIndirectYIndex),
    /* D2 */ Opcode::new(CMP, DirectPageIndirect),
    /* D3 */ Opcode::new(CMP, DirectPageSIndexIndirectYIndex),
    /* D4 */ Opcode::new(PEI, DirectPageIndirect),
    /* D5 */ Opcode::new(CMP, DirectPageXIndex),
    /* D6 */ Opcode::new(DEC, DirectPageXIndex),
    /* D7 */ Opcode::new(CMP, DirectPageLongIndirectYIndex),
    /* D8 */ Opcode::new(CLD, Implied),
    /* D9 */ Opcode::new(CMP, AddressYIndex),
    /* DA */ Opcode::new(PHX, Implied),
    /* DB */ Opcode::new(STP, Implied),
    /* DC */ Opcode::new(JML, AddressLongIndirect),
    /* DD */ Opcode::new(CMP, AddressXIndex),
    /* DE */ Opcode::new(DEC, AddressXIndex),
    /* DF */ Opcode::new(CMP, LongXIndex),
    /* E0 */ Opcode::new(CPX, ImmediateXFlagDependent),
    /* E1 */ Opcode::new(SBC, DirectPageXIndexIndirect),
    /* E2 */ Opcode::new(SEP, Constant8),
    /* E3 */ Opcode::new(SBC, DirectPageSIndex),
    /* E4 */ Opcode::new(CPX, DirectPage),
    /* E5 */ Opcode::new(SBC, DirectPage),
    /* E6 */ Opcode::new(INC, DirectPage),
    /* E7 */ Opcode::new(SBC, DirectPageLongIndirect),
    /* E8 */ Opcode::new(INX, Implied),
    /* E9 */ Opcode::new(SBC, ImmediateMFlagDependent),
    /* EA */ Opcode::new(NOP, Implied),
    /* EB */ Opcode::new(XBA, Implied),
    /* EC */ Opcode::new(CPX, Address),
    /* ED */ Opcode::new(SBC, Address),
    /* EE */ Opcode::new(INC, Address),
    /* EF */ Opcode::new(SBC, Long),
    /* F0 */ Opcode::new(BEQ, Relative8),
    /* F1 */ Opcode::new(SBC, DirectPageIndirectYIndex),
    /* F2 */ Opcode::new(SBC, DirectPageIndirect),
    /* F3 */ Opcode::new(SBC, DirectPageSIndexIndirectYIndex),
    /* F4 */ Opcode::new(PEA, Address),
    /* F5 */ Opcode::new(SBC, DirectPageXIndex),
    /* F6 */ Opcode::new(INC, DirectPageXIndex),
    /* F7 */ Opcode::new(SBC, DirectPageLongIndirectYIndex),
    /* F8 */ Opcode::new(SED, Implied),
    /* F9 */ Opcode::new(SBC, AddressYIndex),
    /* FA */ Opcode::new(PLX, Implied),
    /* FB */ Opcode::new(XCE, Implied),
    /* FC */ Opcode::new(JSR, AddressXIndexIndirect),
    /* FD */ Opcode::new(SBC, AddressXIndex),
    /* FE */ Opcode::new(INC, AddressXIndex),
    /* FF */ Opcode::new(SBC, LongXIndex),
];