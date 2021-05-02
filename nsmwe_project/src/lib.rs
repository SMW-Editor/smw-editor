use std::{cell::RefCell, rc::Rc};

use nsmwe_rom::Rom;

pub struct Project {
    pub title:    String,
    pub rom_data: Rom,
}

pub type ProjectRef = Rc<RefCell<Project>>;
