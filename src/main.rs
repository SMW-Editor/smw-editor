mod backend;
mod frame_context;
mod ui;
mod ui_new;

use std::{cell::RefCell, env, sync::Arc};

use smwe_project::{Project, ProjectRef};
use smwe_rom::SmwRom;

use crate::ui_new::UiMainWindow;

// use crate::{backend::Backend, ui::UiMainWindow};

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize log4rs");

    let project: Option<ProjectRef> = match env::var("ROM_PATH") {
        Ok(rom_path) => {
            log::info!("Opening ROM from path defined in ROM_PATH");
            Some(Arc::new(RefCell::new(Project {
                title:    String::from("Test Project"),
                rom_data: SmwRom::from_file(rom_path)
                    .map_err(|e| {
                        log::error!("{e}");
                        e
                    })
                    .expect("Couldn't load ROM"),
            })))
        }
        Err(_) => {
            log::info!("No path defined in ROM_PATH");
            None
        }
    };

    // let backend = Backend::new(800, 600, "NSMWE v0.1.0");
    // let mut main_window = UiMainWindow::new();
    // backend.run(move |ctx| main_window.tick(ctx), project);

    let options = eframe::NativeOptions::default();
    eframe::run_native("SMW Editor v0.1.0", options, Box::new(|_| Box::new(UiMainWindow::new(project))));
}
