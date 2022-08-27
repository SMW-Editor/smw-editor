use std::{cell::RefCell, sync::Arc};

use smwe_rom::SmwRom;

pub struct Project<'r> {
    pub title:    String,
    pub rom_data: SmwRom<'r>,
}

pub type ProjectRef<'r> = Arc<RefCell<Project<'r>>>;
