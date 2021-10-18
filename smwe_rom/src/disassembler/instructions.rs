// A lot of this code is "borrowed" from DiztinGUIsh, an SNES ROM disassembler and debugger written in C#.
// https://github.com/Dotsarecool/DiztinGUIsh

use std::fmt;

use AddressingMode::*;
use Mnemonic::*;

use crate::snes_utils::addr::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

pub struct Instruction {
    pub mnemonic: Mnemonic,
    pub mode:     AddressingMode,
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Instruction {
    pub const fn new(mnemonic: Mnemonic, mode: AddressingMode) -> Self {
        Self { mnemonic, mode }
    }

    pub fn size(&self) -> usize {
        match self.mode {
            Accumulator | Implied | ImmediateXFlagDependent | ImmediateMFlagDependent => 1,
            Long | LongXIndex => 4,
            Immediate16 | Relative16 | BlockMove => 3,
            m if (Address..=AddressXIndexIndirect).contains(&m) => 3,
            _ => 2,
        }
    }

    fn get_intermediate_address(&self, offset: usize, op_bytes: &[u8], resolve: bool) -> u32 {
        match self.mode {
            m if (DirectPage..=DirectPageYIndex).contains(&m) => {
                if resolve {
                    let operand = op_bytes[0] as u32;
                    let direct_page: u32 = todo!();
                    ((direct_page + operand) & 0xFFFF) as u32
                } else {
                    op_bytes[0] as u32
                }
            }
            DirectPageSIndex | DirectPageSIndexIndirectYIndex => op_bytes[0] as u32,
            Address | AddressXIndex | AddressYIndex | AddressXIndexIndirect => {
                let bank = if self.mnemonic == JSR || self.mnemonic == JMP {
                    let offset_pc = AddrPc(offset);
                    (AddrSnes::try_from_lorom(offset_pc).unwrap().0 >> 16) as u32
                } else {
                    todo!()
                };
                let operand = u16::from_le_bytes([op_bytes[0], op_bytes[1]]);
                (bank << 16) | (operand as u32)
            }
            AddressIndirect | AddressLongIndirect => u16::from_le_bytes([op_bytes[0], op_bytes[1]]) as u32,
            Long | LongXIndex => u32::from_le_bytes([op_bytes[0], op_bytes[1], op_bytes[3], 0]),
            Relative8 | Relative16 => {
                let op_size = if self.mode == Relative8 { 1 } else { 2 };

                let program_counter = {
                    let offset_pc = AddrPc(offset + 1 + op_size);
                    AddrSnes::try_from_lorom(offset_pc).unwrap().0 as u32
                };
                let bank = program_counter >> 16;
                let address = if self.mode == Relative8 {
                    op_bytes[0] as u32
                } else {
                    u16::from_le_bytes([op_bytes[0], op_bytes[1]]) as u32
                };

                (bank << 16) | ((program_counter + address) & 0xFFFF)
            }
            _ => 0,
        }
    }

    pub fn asm_instruction(&self, offset: usize, op_bytes: &[u8]) -> String {
        let address = self.get_intermediate_address(offset, op_bytes, false);
        format!("CODE_{:06X}: {}{}", offset, self.mnemonic, match self.mode {
            Implied => {
                "".to_string()
            }
            Accumulator => {
                " A".to_string()
            }
            Constant8 | Immediate8 => {
                format!(" #${:02X}", op_bytes[0])
            }
            Immediate16 => {
                format!(" #${:04X}", u16::from_le_bytes([op_bytes[0], op_bytes[1]]))
            }
            ImmediateXFlagDependent => {
                let x_flag: bool = todo!();
                if x_flag {
                    format!(" #${:02X}", op_bytes[0])
                } else {
                    format!(" #${:04X}", u16::from_le_bytes([op_bytes[0], op_bytes[1]]))
                }
            }
            ImmediateMFlagDependent => {
                let m_flag: bool = todo!();
                if m_flag {
                    format!(" #${:02X}", op_bytes[0])
                } else {
                    format!(" #${:04X}", u16::from_le_bytes([op_bytes[0], op_bytes[1]]))
                }
            }
            DirectPage => {
                format!(" ${:02X}", address)
            }
            Relative8 => {
                let address = op_bytes[0] as u32;
                let address = address & !(-1 << 8) as u32;
                format!(" ${:02X}", address)
            }
            Relative16 => {
                let address = u16::from_le_bytes([op_bytes[0], op_bytes[1]]) as u32;
                let address = address & !(-1 << 16) as u32;
                format!(" ${:04X}", address)
            }
            Address => {
                format!(" ${:04X}", address)
            }
            Long => {
                format!(" ${:06X}", address)
            }
            DirectPageXIndex | AddressXIndex | LongXIndex => {
                format!(" ${:02X}, X", address)
            }
            DirectPageYIndex | AddressYIndex => {
                format!(" ${:02X}, Y", address)
            }
            DirectPageSIndex => {
                format!(" ${:02X}, S", address)
            }
            DirectPageIndirect => {
                format!(" (${:02X})", address)
            }
            AddressIndirect => {
                format!(" (${:04X})", address)
            }
            DirectPageXIndexIndirect => {
                format!(" (${:02X}, X)", address)
            }
            AddressXIndexIndirect => {
                format!(" (${:04X}, X)", address)
            }
            DirectPageIndirectYIndex => {
                format!(" (${:02X}), Y", address)
            }
            DirectPageSIndexIndirectYIndex => {
                format!(" (${:02X}, S), Y", address)
            }
            DirectPageLongIndirect => {
                format!(" [${:02X}]", address)
            }
            AddressLongIndirect => {
                format!(" [${:04X}]", address)
            }
            DirectPageLongIndirectYIndex => {
                format!(" [${:02X}], Y", address)
            }
            BlockMove => {
                format!(" ${:02X}, ${:02X}", op_bytes[0], op_bytes[1])
            }
        })
    }
}

// Ordered by hex values of opcodes
pub static SNES_INSTRUCTIONS: [Instruction; 0x100] = [
    /* 00 */ Instruction::new(BRK, Constant8),
    /* 01 */ Instruction::new(ORA, DirectPageXIndexIndirect),
    /* 02 */ Instruction::new(COP, Constant8),
    /* 03 */ Instruction::new(ORA, DirectPageSIndex),
    /* 04 */ Instruction::new(TSB, DirectPage),
    /* 05 */ Instruction::new(ORA, DirectPage),
    /* 06 */ Instruction::new(ASL, DirectPage),
    /* 07 */ Instruction::new(ORA, DirectPageLongIndirect),
    /* 08 */ Instruction::new(PHP, Implied),
    /* 09 */ Instruction::new(ORA, ImmediateMFlagDependent),
    /* 0A */ Instruction::new(ASL, Accumulator),
    /* 0B */ Instruction::new(PHD, Implied),
    /* 0C */ Instruction::new(TSB, Address),
    /* 0D */ Instruction::new(ORA, Address),
    /* 0E */ Instruction::new(ASL, Address),
    /* 0F */ Instruction::new(ORA, Long),
    /* 10 */ Instruction::new(BPL, Relative8),
    /* 11 */ Instruction::new(ORA, DirectPageIndirectYIndex),
    /* 12 */ Instruction::new(ORA, DirectPageIndirect),
    /* 13 */ Instruction::new(ORA, DirectPageSIndexIndirectYIndex),
    /* 14 */ Instruction::new(TRB, DirectPage),
    /* 15 */ Instruction::new(ORA, DirectPageXIndex),
    /* 16 */ Instruction::new(ASL, DirectPageXIndex),
    /* 17 */ Instruction::new(ORA, DirectPageLongIndirectYIndex),
    /* 18 */ Instruction::new(CLC, Implied),
    /* 19 */ Instruction::new(ORA, AddressYIndex),
    /* 1A */ Instruction::new(INC, Accumulator),
    /* 1B */ Instruction::new(TCS, Implied),
    /* 1C */ Instruction::new(TRB, Address),
    /* 1D */ Instruction::new(ORA, AddressXIndex),
    /* 1E */ Instruction::new(ASL, AddressXIndex),
    /* 1F */ Instruction::new(ORA, LongXIndex),
    /* 20 */ Instruction::new(JSR, Address),
    /* 21 */ Instruction::new(AND, DirectPageXIndexIndirect),
    /* 22 */ Instruction::new(JSL, Long),
    /* 23 */ Instruction::new(AND, DirectPageSIndex),
    /* 24 */ Instruction::new(BIT, DirectPage),
    /* 25 */ Instruction::new(AND, DirectPage),
    /* 26 */ Instruction::new(ROL, DirectPage),
    /* 27 */ Instruction::new(AND, DirectPageLongIndirect),
    /* 28 */ Instruction::new(PLP, Implied),
    /* 29 */ Instruction::new(AND, ImmediateMFlagDependent),
    /* 2A */ Instruction::new(ROL, Accumulator),
    /* 2B */ Instruction::new(PLD, Implied),
    /* 2C */ Instruction::new(BIT, Address),
    /* 2D */ Instruction::new(AND, Address),
    /* 2E */ Instruction::new(ROL, Address),
    /* 2F */ Instruction::new(AND, Long),
    /* 30 */ Instruction::new(BMI, Relative8),
    /* 31 */ Instruction::new(AND, DirectPageIndirectYIndex),
    /* 32 */ Instruction::new(AND, DirectPageIndirect),
    /* 33 */ Instruction::new(AND, DirectPageSIndexIndirectYIndex),
    /* 34 */ Instruction::new(BIT, DirectPageXIndex),
    /* 35 */ Instruction::new(AND, DirectPageXIndex),
    /* 36 */ Instruction::new(ROL, DirectPageXIndex),
    /* 37 */ Instruction::new(AND, DirectPageLongIndirectYIndex),
    /* 38 */ Instruction::new(SEC, Implied),
    /* 39 */ Instruction::new(AND, AddressYIndex),
    /* 3A */ Instruction::new(DEC, Accumulator),
    /* 3B */ Instruction::new(TSC, Implied),
    /* 3C */ Instruction::new(BIT, AddressXIndex),
    /* 3D */ Instruction::new(AND, AddressXIndex),
    /* 3E */ Instruction::new(ROL, AddressXIndex),
    /* 3F */ Instruction::new(AND, LongXIndex),
    /* 40 */ Instruction::new(RTI, Implied),
    /* 41 */ Instruction::new(EOR, DirectPageXIndexIndirect),
    /* 42 */ Instruction::new(WDM, Constant8),
    /* 43 */ Instruction::new(EOR, DirectPageSIndex),
    /* 44 */ Instruction::new(MVP, BlockMove),
    /* 45 */ Instruction::new(EOR, DirectPage),
    /* 46 */ Instruction::new(LSR, DirectPage),
    /* 47 */ Instruction::new(EOR, DirectPageLongIndirect),
    /* 48 */ Instruction::new(PHA, Implied),
    /* 49 */ Instruction::new(EOR, ImmediateMFlagDependent),
    /* 4A */ Instruction::new(LSR, Accumulator),
    /* 4B */ Instruction::new(PHK, Implied),
    /* 4C */ Instruction::new(JMP, Address),
    /* 4D */ Instruction::new(EOR, Address),
    /* 4E */ Instruction::new(LSR, Address),
    /* 4F */ Instruction::new(EOR, Long),
    /* 50 */ Instruction::new(BVC, Relative8),
    /* 51 */ Instruction::new(EOR, DirectPageIndirectYIndex),
    /* 52 */ Instruction::new(EOR, DirectPageIndirect),
    /* 53 */ Instruction::new(EOR, DirectPageSIndexIndirectYIndex),
    /* 54 */ Instruction::new(MVN, BlockMove),
    /* 55 */ Instruction::new(EOR, DirectPageXIndex),
    /* 56 */ Instruction::new(LSR, DirectPageXIndex),
    /* 57 */ Instruction::new(EOR, DirectPageLongIndirectYIndex),
    /* 58 */ Instruction::new(CLI, Implied),
    /* 59 */ Instruction::new(EOR, AddressYIndex),
    /* 5A */ Instruction::new(PHY, Implied),
    /* 5B */ Instruction::new(TCD, Implied),
    /* 5C */ Instruction::new(JML, Long),
    /* 5D */ Instruction::new(EOR, AddressXIndex),
    /* 5E */ Instruction::new(LSR, AddressXIndex),
    /* 5F */ Instruction::new(EOR, LongXIndex),
    /* 60 */ Instruction::new(RTS, Implied),
    /* 61 */ Instruction::new(ADC, DirectPageXIndexIndirect),
    /* 62 */ Instruction::new(PER, Relative16),
    /* 63 */ Instruction::new(ADC, DirectPageSIndex),
    /* 64 */ Instruction::new(STZ, DirectPage),
    /* 65 */ Instruction::new(ADC, DirectPage),
    /* 66 */ Instruction::new(ROR, DirectPage),
    /* 67 */ Instruction::new(ADC, DirectPageLongIndirect),
    /* 68 */ Instruction::new(PLA, Implied),
    /* 69 */ Instruction::new(ADC, ImmediateMFlagDependent),
    /* 6A */ Instruction::new(ROR, Accumulator),
    /* 6B */ Instruction::new(RTL, Implied),
    /* 6C */ Instruction::new(JMP, AddressIndirect),
    /* 6D */ Instruction::new(ADC, Address),
    /* 6E */ Instruction::new(ROR, Address),
    /* 6F */ Instruction::new(ADC, Long),
    /* 70 */ Instruction::new(BVS, Relative8),
    /* 71 */ Instruction::new(ADC, DirectPageIndirectYIndex),
    /* 72 */ Instruction::new(ADC, DirectPageIndirect),
    /* 73 */ Instruction::new(ADC, DirectPageSIndexIndirectYIndex),
    /* 74 */ Instruction::new(STZ, DirectPageXIndex),
    /* 75 */ Instruction::new(ADC, DirectPageXIndex),
    /* 76 */ Instruction::new(ROR, DirectPageXIndex),
    /* 77 */ Instruction::new(ADC, DirectPageLongIndirectYIndex),
    /* 78 */ Instruction::new(SEI, Implied),
    /* 79 */ Instruction::new(ADC, AddressYIndex),
    /* 7A */ Instruction::new(PLY, Implied),
    /* 7B */ Instruction::new(TDC, Implied),
    /* 7C */ Instruction::new(JMP, AddressXIndexIndirect),
    /* 7D */ Instruction::new(ADC, AddressXIndex),
    /* 7E */ Instruction::new(ROR, AddressXIndex),
    /* 7F */ Instruction::new(ADC, LongXIndex),
    /* 80 */ Instruction::new(BRA, Relative8),
    /* 81 */ Instruction::new(STA, DirectPageXIndexIndirect),
    /* 82 */ Instruction::new(BRL, Relative16),
    /* 83 */ Instruction::new(STA, DirectPageSIndex),
    /* 84 */ Instruction::new(STY, DirectPage),
    /* 85 */ Instruction::new(STA, DirectPage),
    /* 86 */ Instruction::new(STX, DirectPage),
    /* 87 */ Instruction::new(STA, DirectPageLongIndirect),
    /* 88 */ Instruction::new(DEY, Implied),
    /* 89 */ Instruction::new(BIT, ImmediateMFlagDependent),
    /* 8A */ Instruction::new(TXA, Implied),
    /* 8B */ Instruction::new(PHB, Implied),
    /* 8C */ Instruction::new(STY, Address),
    /* 8D */ Instruction::new(STA, Address),
    /* 8E */ Instruction::new(STX, Address),
    /* 8F */ Instruction::new(STA, Long),
    /* 90 */ Instruction::new(BCC, Relative8),
    /* 91 */ Instruction::new(STA, DirectPageIndirectYIndex),
    /* 92 */ Instruction::new(STA, DirectPageIndirect),
    /* 93 */ Instruction::new(STA, DirectPageSIndexIndirectYIndex),
    /* 94 */ Instruction::new(STY, DirectPageXIndex),
    /* 95 */ Instruction::new(STA, DirectPageXIndex),
    /* 96 */ Instruction::new(STX, DirectPageYIndex),
    /* 97 */ Instruction::new(STA, DirectPageLongIndirectYIndex),
    /* 98 */ Instruction::new(TYA, Implied),
    /* 99 */ Instruction::new(STA, AddressYIndex),
    /* 9A */ Instruction::new(TXS, Implied),
    /* 9B */ Instruction::new(TXY, Implied),
    /* 9C */ Instruction::new(STZ, Address),
    /* 9D */ Instruction::new(STA, AddressXIndex),
    /* 9E */ Instruction::new(STZ, AddressXIndex),
    /* 9F */ Instruction::new(STA, LongXIndex),
    /* A0 */ Instruction::new(LDY, ImmediateXFlagDependent),
    /* A1 */ Instruction::new(LDA, DirectPageXIndexIndirect),
    /* A2 */ Instruction::new(LDX, ImmediateXFlagDependent),
    /* A3 */ Instruction::new(LDA, DirectPageSIndex),
    /* A4 */ Instruction::new(LDY, DirectPage),
    /* A5 */ Instruction::new(LDA, DirectPage),
    /* A6 */ Instruction::new(LDX, DirectPage),
    /* A7 */ Instruction::new(LDA, DirectPageLongIndirect),
    /* A8 */ Instruction::new(TAY, Implied),
    /* A9 */ Instruction::new(LDA, ImmediateMFlagDependent),
    /* AA */ Instruction::new(TAX, Implied),
    /* AB */ Instruction::new(PLB, Implied),
    /* AC */ Instruction::new(LDY, Address),
    /* AD */ Instruction::new(LDA, Address),
    /* AE */ Instruction::new(LDX, Address),
    /* AF */ Instruction::new(LDA, Long),
    /* B0 */ Instruction::new(BCS, Relative8),
    /* B1 */ Instruction::new(LDA, DirectPageIndirectYIndex),
    /* B2 */ Instruction::new(LDA, DirectPageIndirect),
    /* B3 */ Instruction::new(LDA, DirectPageSIndexIndirectYIndex),
    /* B4 */ Instruction::new(LDY, DirectPageXIndex),
    /* B5 */ Instruction::new(LDA, DirectPageXIndex),
    /* B6 */ Instruction::new(LDX, DirectPageYIndex),
    /* B7 */ Instruction::new(LDA, DirectPageLongIndirectYIndex),
    /* B8 */ Instruction::new(CLV, Implied),
    /* B9 */ Instruction::new(LDA, AddressYIndex),
    /* BA */ Instruction::new(TSX, Implied),
    /* BB */ Instruction::new(TYX, Implied),
    /* BC */ Instruction::new(LDY, AddressXIndex),
    /* BD */ Instruction::new(LDA, AddressXIndex),
    /* BE */ Instruction::new(LDX, AddressYIndex),
    /* BF */ Instruction::new(LDA, LongXIndex),
    /* C0 */ Instruction::new(CPY, ImmediateXFlagDependent),
    /* C1 */ Instruction::new(CMP, DirectPageXIndexIndirect),
    /* C2 */ Instruction::new(REP, Constant8),
    /* C3 */ Instruction::new(CMP, DirectPageSIndex),
    /* C4 */ Instruction::new(CPY, DirectPage),
    /* C5 */ Instruction::new(CMP, DirectPage),
    /* C6 */ Instruction::new(DEC, DirectPage),
    /* C7 */ Instruction::new(CMP, DirectPageLongIndirect),
    /* C8 */ Instruction::new(INY, Implied),
    /* C9 */ Instruction::new(CMP, ImmediateMFlagDependent),
    /* CA */ Instruction::new(DEX, Implied),
    /* CB */ Instruction::new(WAI, Implied),
    /* CC */ Instruction::new(CPY, Address),
    /* CD */ Instruction::new(CMP, Address),
    /* CE */ Instruction::new(DEC, Address),
    /* CF */ Instruction::new(CMP, Long),
    /* D0 */ Instruction::new(BNE, Relative8),
    /* D1 */ Instruction::new(CMP, DirectPageIndirectYIndex),
    /* D2 */ Instruction::new(CMP, DirectPageIndirect),
    /* D3 */ Instruction::new(CMP, DirectPageSIndexIndirectYIndex),
    /* D4 */ Instruction::new(PEI, DirectPageIndirect),
    /* D5 */ Instruction::new(CMP, DirectPageXIndex),
    /* D6 */ Instruction::new(DEC, DirectPageXIndex),
    /* D7 */ Instruction::new(CMP, DirectPageLongIndirectYIndex),
    /* D8 */ Instruction::new(CLD, Implied),
    /* D9 */ Instruction::new(CMP, AddressYIndex),
    /* DA */ Instruction::new(PHX, Implied),
    /* DB */ Instruction::new(STP, Implied),
    /* DC */ Instruction::new(JML, AddressLongIndirect),
    /* DD */ Instruction::new(CMP, AddressXIndex),
    /* DE */ Instruction::new(DEC, AddressXIndex),
    /* DF */ Instruction::new(CMP, LongXIndex),
    /* E0 */ Instruction::new(CPX, ImmediateXFlagDependent),
    /* E1 */ Instruction::new(SBC, DirectPageXIndexIndirect),
    /* E2 */ Instruction::new(SEP, Constant8),
    /* E3 */ Instruction::new(SBC, DirectPageSIndex),
    /* E4 */ Instruction::new(CPX, DirectPage),
    /* E5 */ Instruction::new(SBC, DirectPage),
    /* E6 */ Instruction::new(INC, DirectPage),
    /* E7 */ Instruction::new(SBC, DirectPageLongIndirect),
    /* E8 */ Instruction::new(INX, Implied),
    /* E9 */ Instruction::new(SBC, ImmediateMFlagDependent),
    /* EA */ Instruction::new(NOP, Implied),
    /* EB */ Instruction::new(XBA, Implied),
    /* EC */ Instruction::new(CPX, Address),
    /* ED */ Instruction::new(SBC, Address),
    /* EE */ Instruction::new(INC, Address),
    /* EF */ Instruction::new(SBC, Long),
    /* F0 */ Instruction::new(BEQ, Relative8),
    /* F1 */ Instruction::new(SBC, DirectPageIndirectYIndex),
    /* F2 */ Instruction::new(SBC, DirectPageIndirect),
    /* F3 */ Instruction::new(SBC, DirectPageSIndexIndirectYIndex),
    /* F4 */ Instruction::new(PEA, Address),
    /* F5 */ Instruction::new(SBC, DirectPageXIndex),
    /* F6 */ Instruction::new(INC, DirectPageXIndex),
    /* F7 */ Instruction::new(SBC, DirectPageLongIndirectYIndex),
    /* F8 */ Instruction::new(SED, Implied),
    /* F9 */ Instruction::new(SBC, AddressYIndex),
    /* FA */ Instruction::new(PLX, Implied),
    /* FB */ Instruction::new(XCE, Implied),
    /* FC */ Instruction::new(JSR, AddressXIndexIndirect),
    /* FD */ Instruction::new(SBC, AddressXIndex),
    /* FE */ Instruction::new(INC, AddressXIndex),
    /* FF */ Instruction::new(SBC, LongXIndex),
];
