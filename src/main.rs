mod ui;

use std::{cell::RefCell, env, sync::Arc};

use eframe::NativeOptions;
use smwe_project::{Project, ProjectRef};

use crate::ui::UiMainWindow;

fn main() -> eframe::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize log4rs");

    let project = dev_open_rom();
    let native_options = NativeOptions::default();
    eframe::run_native("SMW Editor v0.1.0", native_options, Box::new(|_| Box::new(UiMainWindow::new(project))))
}

fn dev_open_rom() -> Option<ProjectRef> {
    let Ok(rom_path) = env::var("ROM_PATH") else {
        log::info!("No path defined in ROM_PATH");
        return None;
    };

    log::info!("Opening ROM from path defined in ROM_PATH");
    let project = Project::new(rom_path)
        .map_err(|e| {
            log::info!("Cannot create project: {e}");
            e
        })
        .ok()?;

    Some(Arc::new(RefCell::new(project)))
}
