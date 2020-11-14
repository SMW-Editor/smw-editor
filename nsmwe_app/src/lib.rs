pub mod ui;

mod backend;
mod project;

pub use self::project::{
    OptProjectRef,
    Project,
};

use self::backend::Backend;
use self::ui::{
    UiMainWindow,
    UiTool,
};

use std::{
    cell::RefCell,
    rc::Rc,
};

pub struct App {
    backend: Backend,
    project_data: Rc<OptProjectRef>,
}

impl App {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        App {
            backend: Backend::new(width, height, title),
            project_data: Rc::new(RefCell::new(None)),
        }
    }

    pub fn run(self) {
        let mut main_window = UiMainWindow::new(Rc::clone(&self.project_data));
        self.backend.run(move |ui| main_window.run(&ui));
    }
}