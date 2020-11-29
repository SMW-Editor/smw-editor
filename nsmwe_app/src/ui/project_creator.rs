use crate::{
    OptProjectRef,
    Project,
    UiTool,
    color,
};

use imgui::{
    ImString,
    Window,
    Ui,
    im_str,
};

use inline_tweak::tweak;

use nsmwe_rom::Rom;

use std::{
    path::Path,
    rc::Rc,
};

pub struct UiProjectCreator {
    project_title: ImString,
    base_rom_path: ImString,

    err_project_title: ImString,
    err_base_rom_path: ImString,
    err_project_creation: ImString,

    project_ref: Rc<OptProjectRef>,
}

impl UiTool for UiProjectCreator {
    fn run(&mut self, ui: &Ui) -> bool {
        let mut opened = true;
        let mut created_or_cancelled = false;

        Window::new(im_str!("Create new project"))
            .always_auto_resize(true)
            .resizable(false)
            .collapsible(false)
            .opened(&mut opened)
            .build(ui, || {
                self.input_project_title(ui);
                self.input_rom_file_path(ui);
                self.create_or_cancel(ui, &mut created_or_cancelled);
                self.project_error_popup(ui);
            });

        opened && !created_or_cancelled
    }
}

impl UiProjectCreator {
    pub fn new(project_ref: Rc<OptProjectRef>) -> Self {
        let mut myself = UiProjectCreator {
            project_title: ImString::new("My SMW hack"),
            base_rom_path: ImString::new(""),

            err_project_title: ImString::new(""),
            err_base_rom_path: ImString::new(""),
            err_project_creation: ImString::new(""),

            project_ref,
        };
        myself.handle_rom_file_path();
        myself
    }

    fn input_project_title(&mut self, ui: &Ui) {
        ui.text(im_str!("Project title:"));
        if ui.input_text(im_str!("##project_title"), &mut self.project_title)
            .build()
        {
            self.handle_project_title();
        }

        if !self.err_project_title.is_empty() {
            ui.text_colored(color::TEXT_ERROR, &self.err_project_title);
        }
    }

    fn handle_project_title(&mut self) {
        if self.project_title.is_empty() {
            self.err_project_title = ImString::new("Project title cannot be empty.");
        } else {
            self.err_project_title.clear();
        }
    }

    fn input_rom_file_path(&mut self, ui: &Ui) {
        ui.text(im_str!("Base ROM file:"));
        if ui.input_text(im_str!("##rom_file"), &mut self.base_rom_path)
            .build()
        {
            self.handle_rom_file_path();
        }
        ui.same_line(0.0);
        if ui.small_button(im_str!("Browse...")) {
            self.open_file_selector();
        }

        if !self.err_base_rom_path.is_empty() {
            ui.text_colored(color::TEXT_ERROR, &self.err_base_rom_path);
        }
    }

    fn handle_rom_file_path(&mut self) {
        let file_path = Path::new(self.base_rom_path.to_str());
        if !file_path.exists() {
            self.err_base_rom_path = ImString::new(
                format!("File '{}' does not exist.", self.base_rom_path));
        } else if file_path.is_dir() {
            self.err_base_rom_path = ImString::new(
                format!("'{}' is not a file.", self.base_rom_path));
        } else {
            self.err_base_rom_path.clear();
        }
    }

    fn open_file_selector(&mut self) {
        use nfd2::Response;
        if let Response::Okay(path) = nfd2::open_file_dialog(Some("smc,sfc"), None)
            .unwrap_or_else(|e| panic!("Cannot open file selector: {}", e))
        {
            self.base_rom_path = ImString::new(path.to_str().unwrap());
            self.handle_rom_file_path();
        }
    }

    fn create_or_cancel(&mut self, ui: &Ui, created_or_cancelled: &mut bool) {
        if self.no_creation_errors() {
            if ui.small_button(im_str!("Create")) {
                self.handle_project_creation(ui, created_or_cancelled);
            }
        } else {
            ui.text_disabled(im_str!("Create"));
        }
        ui.same_line(0.0);
        if ui.small_button(im_str!("Cancel")) {
            *created_or_cancelled = true;
        }
    }

    fn handle_project_creation(&mut self, ui: &Ui, created_or_cancelled: &mut bool) {
        match Rom::from_file(self.base_rom_path.to_str()) {
            Ok(rom_data) => {
                let project = Project {
                    title: self.project_title.to_string(),
                    rom_data,
                };
                *self.project_ref.borrow_mut() = Some(project);
                *created_or_cancelled = true;
                self.err_project_creation.clear();
            }
            Err(err) => {
                self.err_project_creation = ImString::from(err.to_string());
                ui.open_popup(im_str!("Error!##project_error"));
            }
        }
    }

    fn project_error_popup(&self, ui: &Ui) {
        ui.popup_modal(im_str!("Error!##project_error"))
            .always_auto_resize(true)
            .resizable(false)
            .collapsible(false)
            .build(|| {
                ui.text_wrapped(&self.err_project_creation);
                if ui.button(im_str!("OK"), [tweak!(300.0), tweak!(20.0)]) {
                    ui.close_current_popup();
                }
            });
    }

    fn no_creation_errors(&self) -> bool {
        vec![
            &self.err_base_rom_path,
            &self.err_project_title,
        ].iter().all(|s| s.is_empty())
    }
}