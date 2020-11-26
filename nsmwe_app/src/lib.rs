pub mod ui;

mod backend;
mod project;

pub use crate::{
    project::{
        OptProjectRef,
        Project,
    },
    ui::*,
};

use self::backend::Backend;
use self::ui::{
    UiMainWindow,
    UiTool,
};

use nsmwe_rom::Rom;

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
            Some(Project {
                title: String::from("Test Project"),
                rom_data: Rom::from_file(rom_path).expect("Couldn't load ROM."),
            })
        } else {
            None
        };
        
        App {
            backend: Backend::new(width, height, title),
            project_data: Rc::new(RefCell::new(project)),
        }
    }

    pub fn run(self) {
        let mut main_window = UiMainWindow::new(Rc::clone(&self.project_data));
        self.backend.run(move |ui| main_window.run(&ui));
    }
}