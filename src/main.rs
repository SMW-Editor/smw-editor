mod frame_context;
mod ui;

use std::{cell::RefCell, env, sync::Arc};

use smwe_project::{Project, ProjectRef};
use smwe_rom::SmwRom;

use crate::ui::UiMainWindow;

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize log4rs");

    let project = dev_open_rom();
    let options = eframe::NativeOptions::default();
    if let Err(e) = eframe::run_native("SMW Editor v0.1.0", options, Box::new(|_| Box::new(UiMainWindow::new(project))))
    {
        log::error!("Application error: {e}");
    };
}

fn dev_open_rom() -> Option<ProjectRef> {
    let Ok(rom_path) = env::var("ROM_PATH") else {
        log::info!("No path defined in ROM_PATH");
        return None;
    };

    log::info!("Opening ROM from path defined in ROM_PATH");
    let project = Project {
        title:    String::from("Test Project"),
        rom_data: SmwRom::from_file(rom_path)
            .map_err(|e| {
                log::error!("Couldn't load ROM: {e}");
                e
            })
            .ok()?,
    };
    Some(Arc::new(RefCell::new(project)))
}
