//! Contains addressing mode definitions

use std::fmt;

use super::{Cpu, Mem};

/// As a safety measure, the load and store methods take the mode by value and consume it. Using
/// the same object twice requires an explicit `.clone()` (`Copy` isn't implemented).
#[derive(Clone)]
pub enum AddressingMode {
    Immediate(u16),
    Immediate8(u8),

    /// "Program Counter Relative-r"
    /// Used for jumps
    /// (PBR, PC + <val>)  [PC+<val> wraps inside the bank]
    Rel(i8),
    /// "PC Relative Long-r"
    /// Used for `PER` (Push Effective Relative Address)
    /// (<unused>, PC + <val>)
    RelLong(i16),

    /// "Direct-d"
    /// <val> + direct page register in bank 0
    /// (0, D + <val>)
    Direct(u8),
    /// "Direct Indexed with X-d,x"
    /// (0, D + <val> + X)
    DirectIndexedX(u8),
    /// "Direct Indexed with Y-d,y"
    /// (0, D + <val> + Y)
    DirectIndexedY(u8),

    /// "Direct Indexed Indirect-(d,x)" - Indirect-X
    /// addr := load2(0, D + <val> + X)
    /// (DBR, addr)
    DirectIndexedIndirect(u8),

    /// "Direct Indirect-(d)"
    /// addr := load2(0, D + <val>)
    /// (DBR, addr)
    DirectIndirect(u8),

    // "Direct Indirect Indexed-(d),y" - Indirect-Y
    // addr := load2(D + <val>)
    // (DBR, addr + Y)  (NOTE: Wraps across data bank!)
    DirectIndirectIndexed(u8),

    /// "Direct Indirect Long-[d]"
    /// (bank, addr) := load3(0, D + <val>)
    /// (bank, addr)
    DirectIndirectLong(u8),

    /// "Direct Indirect Long Indexed-[d],y" (or "Direct Indirect Indexed Long")
    /// (bank, addr) := load3(0, D + <val>)
    /// (bank, addr + Y)
    DirectIndirectLongIdx(u8),

    /// "Absolute-a"
    /// Access absolute offset in the current data bank
    /// (DBR, <val>)
    Absolute(u16),

    /// "Absolute Indexed with X-a,x"
    /// (DBR, <val> + X)
    AbsIndexedX(u16),

    /// "Absolute Indexed with Y-a,y"
    /// (DBR, <val> + Y)
    AbsIndexedY(u16),

    /// "Absolute Indexed Indirect-(a,x)"
    /// addr := load2(PBR, <val> + X)
    /// (PBR, addr)
    AbsIndexedIndirect(u16),

    /// "Absolute Long Indexed With X-al,x" - Absolute Long + X
    /// (<val0>, <val1> + X)
    AbsLongIndexedX(u8, u16),

    /// "Absolute Long-al"
    /// Access absolute offset in the specified data bank (DBR is not changed)
    /// (<val0>, <val1>)
    AbsoluteLong(u8, u16),

    /// "Absolute Indirect-(a)"
    /// Used only by `jmp`.
    /// addr := load2(0, <val>)
    /// (PBR, addr)
    AbsoluteIndirect(u16),

    /// "Absolute Indirect Long-[a]"
    /// Used only by `jml`.
    /// (bank, addr) := load3(0, <val>)
    /// (bank, addr)
    AbsoluteIndirectLong(u16),

    /// "Stack Relative-d,s"
    /// (0, SP + <val>)
    StackRel(u8),
}

impl AddressingMode {
    /// Loads a byte from where this AM points to (or returns the immediate value)
    pub fn loadb<M: Mem>(self, cpu: &mut Cpu<M>) -> u8 {
        match self {
            AddressingMode::Immediate(_) => panic!("loadb on 16-bit immediate"),
            AddressingMode::Immediate8(val) => val,
            _ => {
                let (bank, addr) = self.address(cpu);
                cpu.loadb(bank, addr)
            }
        }
    }

    pub fn loadw<M: Mem>(self, cpu: &mut Cpu<M>) -> u16 {
        match self {
            AddressingMode::Immediate(val) => val,
            AddressingMode::Immediate8(_) => panic!("loadw on 8-bit immediate"),
            _ => {
                let (bank, addr) = self.address(cpu);
                cpu.loadw(bank, addr)
            }
        }
    }

    pub fn storeb<M: Mem>(self, cpu: &mut Cpu<M>, value: u8) {
        let (bank, addr) = self.address(cpu);
        cpu.storeb(bank, addr, value);
    }

    pub fn storew<M: Mem>(self, cpu: &mut Cpu<M>, value: u16) {
        let (bank, addr) = self.address(cpu);
        cpu.storew(bank, addr, value);
    }

    /// Computes the effective address as a bank-address-tuple. Panics if the addressing mode is
    /// immediate. For jumps, the effective address is the jump target.
    pub fn address<M: Mem>(&self, cpu: &mut Cpu<M>) -> (u8, u16) {
        use self::AddressingMode::*;

        // FIXME is something here dependant on register sizes?
        // -> Yes, the cycle count. This causes bad timing, fix it!
        // FIXME Use next bank on some address overflows

        match *self {
            Absolute(addr) => (cpu.dbr, addr),
            AbsoluteLong(bank, addr) => (bank, addr),
            AbsLongIndexedX(bank, addr) => {
                if !cpu.p.small_index() {
                    cpu.cy += 1
                }
                let a = ((bank as u32) << 16) | addr as u32;
                let mut eff_addr = a + cpu.x as u32;
                //assert!(eff_addr & 0xff000000 == 0, "address overflow");
                eff_addr &= 0xFFFFFF;
                let bank = eff_addr >> 16;
                let addr = eff_addr as u16;
                (bank as u8, addr)
            }
            AbsIndexedX(offset) => {
                if !cpu.p.small_index() {
                    cpu.cy += 1
                }
                (cpu.dbr, offset + cpu.x)
            }
            AbsIndexedY(offset) => {
                if !cpu.p.small_index() {
                    cpu.cy += 1
                }
                (cpu.dbr, offset + cpu.y)
            }
            AbsIndexedIndirect(addr_ptr) => {
                let (x, pbr) = (cpu.x, cpu.pbr);
                let addr = cpu.loadw(pbr, addr_ptr + x);
                (pbr, addr)
            }
            AbsoluteIndirect(addr_ptr) => {
                let addr = cpu.loadw(0, addr_ptr);
                (cpu.pbr, addr)
            }
            AbsoluteIndirectLong(addr_ptr) => {
                let addr = cpu.loadw(0, addr_ptr);
                let bank = cpu.loadb(0, addr_ptr + 2);
                (bank, addr)
            }
            Rel(rel) => (cpu.pbr, (cpu.pc as i16).wrapping_add(rel as i16) as u16),
            RelLong(rel_long) => (cpu.pbr, (cpu.pc as i16).wrapping_add(rel_long) as u16),
            Direct(offset) => {
                if cpu.d & 0xff != 0 {
                    cpu.cy += 1
                }
                (0, cpu.d.wrapping_add(offset as u16))
            }
            DirectIndexedX(offset) => {
                if cpu.d & 0xff != 0 {
                    cpu.cy += 1
                }
                if !cpu.p.small_index() {
                    cpu.cy += 1
                }
                (0, cpu.d.wrapping_add(offset as u16).wrapping_add(cpu.x))
            }
            DirectIndexedY(offset) => {
                if cpu.d & 0xff != 0 {
                    cpu.cy += 1
                }
                if !cpu.p.small_index() {
                    cpu.cy += 1
                }
                (0, cpu.d.wrapping_add(offset as u16).wrapping_add(cpu.y))
            }
            DirectIndexedIndirect(offset) => {
                if cpu.d & 0xff != 0 {
                    cpu.cy += 1
                }
                let addr_ptr = cpu.d.wrapping_add(offset as u16).wrapping_add(cpu.x);
                let lo = cpu.loadb(0, addr_ptr) as u16;
                let hi = cpu.loadb(0, addr_ptr + 1) as u16;
                (cpu.dbr, (hi << 8) | lo)
            }
            DirectIndirectIndexed(offset) => {
                if cpu.d & 0xff != 0 {
                    cpu.cy += 1
                }
                if !cpu.p.small_index() {
                    cpu.cy += 1
                }

                let addr_ptr = cpu.d.wrapping_add(offset as u16);
                let lo = cpu.loadb(0, addr_ptr) as u32;
                let hi = cpu.loadb(0, addr_ptr + 1) as u32;
                let base_address = ((cpu.dbr as u32) << 16) | (hi << 8) | lo;
                let eff_addr = base_address + cpu.y as u32;
                assert!(eff_addr & 0xff000000 == 0, "address overflow");

                let bank = (eff_addr >> 16) as u8;
                let addr = eff_addr as u16;
                (bank, addr)
            }
            DirectIndirect(offset) => {
                if cpu.d & 0xff != 0 {
                    cpu.cy += 1
                }
                let addr_ptr = cpu.d.wrapping_add(offset as u16);
                let lo = cpu.loadb(0, addr_ptr) as u16;
                let hi = cpu.loadb(0, addr_ptr + 1) as u16;
                (cpu.dbr, (hi << 8) | lo)
            }
            DirectIndirectLong(offset) => {
                if cpu.d & 0xff != 0 {
                    cpu.cy += 1
                }
                let addr_ptr = cpu.d.wrapping_add(offset as u16);
                let lo = cpu.loadb(0, addr_ptr) as u16;
                let hi = cpu.loadb(0, addr_ptr + 1) as u16;
                let bank = cpu.loadb(0, addr_ptr + 2);
                (bank, (hi << 8) | lo)
            }
            DirectIndirectLongIdx(offset) => {
                // "The 24-bit base address is pointed to by the sum of the second byte of the
                // instruction and the Direct Register. The effective address is this 24-bit base
                // address plus the Y Index Register."
                if cpu.d & 0xff != 0 {
                    cpu.cy += 1
                }
                if !cpu.p.small_index() {
                    cpu.cy += 1
                }

                let addr_ptr = cpu.d.wrapping_add(offset as u16);
                let lo = cpu.loadb(0, addr_ptr) as u32;
                let hi = cpu.loadb(0, addr_ptr + 1) as u32;
                let bank = cpu.loadb(0, addr_ptr + 2) as u32;
                let base_address = (bank << 16) | (hi << 8) | lo;
                let eff_addr = base_address + cpu.y as u32;
                assert!(eff_addr & 0xff000000 == 0, "address overflow");

                let bank = (eff_addr >> 16) as u8;
                let addr = eff_addr as u16;
                (bank, addr)
            }
            StackRel(offset) => {
                let addr = cpu.s + offset as u16;
                (0, addr)
            }
            Immediate(_) | Immediate8(_) => panic!(
                "attempted to take the address of an immediate value (attempted store to \
                    immediate?)"
            ),
        }
    }
}

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AddressingMode::*;

        match *self {
            Immediate(val) => write!(f, "#${:04X}", val),
            Immediate8(val) => write!(f, "#${:02X}", val),
            Absolute(addr) => write!(f, "${:04X}", addr),
            AbsoluteLong(bank, addr) => write!(f, "${:02X}:{:04X}", bank, addr),
            AbsLongIndexedX(bank, addr) => write!(f, "${:02X}:{:04X},x", bank, addr),
            AbsIndexedX(offset) => write!(f, "${:04X},x", offset),
            AbsIndexedY(offset) => write!(f, "${:04X},y", offset),
            AbsIndexedIndirect(addr) => write!(f, "(${:04X},x)", addr),
            AbsoluteIndirect(addr) => write!(f, "(${:04X})", addr),
            AbsoluteIndirectLong(addr) => write!(f, "[${:04X}]", addr),
            Rel(rel) => write!(f, "{:+}", rel),
            RelLong(rel_long) => write!(f, "{:+}", rel_long),
            Direct(offset) => write!(f, "${:02X}", offset),
            DirectIndexedX(offset) => write!(f, "${:02X},x", offset),
            DirectIndexedY(offset) => write!(f, "${:02X},y", offset),
            DirectIndexedIndirect(offset) => write!(f, "(${:02X},x)", offset),
            DirectIndirectIndexed(offset) => write!(f, "(${:02X}),y", offset),
            DirectIndirect(offset) => write!(f, "(${:02X})", offset),
            DirectIndirectLong(offset) => write!(f, "[${:02X}]", offset),
            DirectIndirectLongIdx(offset) => write!(f, "[${:02X}],y", offset),
            StackRel(offset) => write!(f, "${:02X},s", offset),
        }
    }
}
