use std::{
    cell::RefCell,
    sync::Arc,
};

use smwe_rom::SmwRom;

pub struct Project {
    pub title:    String,
    pub rom_data: SmwRom,
}

pub type ProjectRef = Arc<RefCell<Project>>;
