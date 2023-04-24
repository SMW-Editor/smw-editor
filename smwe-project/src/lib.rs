use std::{cell::RefCell, sync::Arc};

use smwe_emu::rom::Rom;
use smwe_rom::SmwRom;

pub struct Project {
    pub title:        String,
    pub old_rom_data: SmwRom,
    pub rom:          Rom,
}

pub type ProjectRef = Arc<RefCell<Project>>;
