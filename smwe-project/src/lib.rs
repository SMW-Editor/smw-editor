use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use smwe_emu::rom::Rom;
use smwe_rom::SmwRom;

#[derive(Debug)]
pub struct Project {
    pub title:        String,
    pub old_rom_data: SmwRom,
    pub rom:          Rc<Rom>,
}

pub type ProjectRef = Arc<RefCell<Project>>;

impl Project {
    pub fn new(rom_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let old_rom_data = SmwRom::from_file(&rom_path)?;
        // TODO: check smc/sfc
        let mut rom = Rom::new(std::fs::read(&rom_path)?[0x200..].to_vec());
        rom.load_symbols(include_str!("../../symbols/SMW_U.sym"));

        Ok(Self { title: String::from("Test Project"), old_rom_data, rom: Rc::new(rom) })
    }
}
