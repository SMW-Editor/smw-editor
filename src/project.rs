use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use egui::Id;
use smwe_emu::rom::Rom;

#[derive(Debug)]
pub struct Project {
    pub title: String,
    pub rom:   Arc<Rom>,
}

pub type ProjectRef = Rc<RefCell<Project>>;

impl Project {
    pub fn new(rom_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        // TODO: check smc/sfc
        let mut rom = Rom::new(std::fs::read(&rom_path)?[0x200..].to_vec());
        rom.load_symbols(include_str!("../symbols/SMW_U.sym"));

        Ok(Self { title: String::from("Test Project"), rom: Arc::new(rom) })
    }

    pub fn rom_id() -> Id {
        Id::new("rom")
    }

    pub fn project_title_id() -> Id {
        Id::new("project_title")
    }
}
