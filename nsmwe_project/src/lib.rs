extern crate nsmwe_rom;

use nsmwe_rom::Rom;

use std::cell::RefCell;

pub struct Project {
    pub title: String,
    pub rom_data: Rom,
}

pub type OptProjectRef = RefCell<Option<Project>>;
