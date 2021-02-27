use crate::backend::Backend;

use nsmwe_project::{OptProjectRef, Project};
use nsmwe_rom::Rom;
use nsmwe_ui::UiMainWindow;

use std::{
    cell::RefCell,
    env,
    rc::Rc,
};

pub struct App {
    backend: Backend,
    project_data: Rc<OptProjectRef>,
}

impl App {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        let project = if let Ok(rom_path) = env::var("ROM_PATH") {
            log::info!("Opening ROM from path defined in ROM_PATH");
            Some(Project {
                title: String::from("Test Project"),
                rom_data: Rom::from_file(rom_path).expect("Couldn't load ROM"),
            })
        } else {
            log::info!("No path defined in ROM_PATH");
            None
        };

        App {
            backend: Backend::new(width, height, title),
            project_data: Rc::new(RefCell::new(project)),
        }
    }

    pub fn run(self) {
        log::info!("Starting up");
        let mut main_window = UiMainWindow::new(Rc::clone(&self.project_data));
        self.backend.run(move |ui| main_window.run(&ui));
    }
}
