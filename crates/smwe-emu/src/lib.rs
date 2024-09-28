//! Game emulation, used to run various aspects of the game.

pub mod emu;
pub mod rom;

pub type Cpu = wdc65816::Cpu<emu::CheckedMem>;
