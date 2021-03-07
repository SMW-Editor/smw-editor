use nsmwe_rom::Rom;

use std::{
    cell::RefCell,
    rc::Rc,
};

pub struct Project {
    pub title: String,
    pub rom_data: Rom,
}

pub type ProjectRef = Rc<RefCell<Project>>;
