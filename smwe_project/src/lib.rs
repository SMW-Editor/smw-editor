use std::{cell::RefCell, rc::Rc};

use smwe_rom::SmwRom;

pub struct Project {
    pub title:    String,
    pub rom_data: SmwRom,
}

pub type ProjectRef = Rc<RefCell<Project>>;
