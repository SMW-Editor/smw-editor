mod backend;
mod frame_context;
mod ui;

use std::{cell::RefCell, env, rc::Rc};

use smwe_project::{Project, ProjectRef};
use smwe_rom::SmwRom;

use crate::{backend::Backend, ui::UiMainWindow};

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize log4rs");

    let project: Option<ProjectRef> = if let Ok(rom_path) = env::var("ROM_PATH") {
        log::info!("Opening ROM from path defined in ROM_PATH");
        Some(Rc::new(RefCell::new(Project {
            title:    String::from("Test Project"),
            rom_data: SmwRom::from_file(rom_path)
                .map_err(|e| {
                    log::error!("{}", e);
                    e
                })
                .expect("Couldn't load ROM"),
        })))
    } else {
        log::info!("No path defined in ROM_PATH");
        None
    };

    let backend = Backend::new(800, 600, "NSMWE v0.1.0");
    let mut main_window = UiMainWindow::new();
    backend.run(move |ctx| main_window.tick(ctx), project);
}
