use std::{cell::RefCell, path::Path, rc::Rc};

use imgui::{PopupModal, Ui, Window};
use inline_tweak::tweak;
use smwe_project::Project;
use smwe_rom::SmwRom;

use crate::{
    frame_context::FrameContext,
    ui::{color, title_with_id, UiTool, WindowId},
};

pub struct UiProjectCreator {
    title: String,

    project_title: String,
    base_rom_path: String,

    err_project_title:    String,
    err_base_rom_path:    String,
    err_project_creation: String,
}

impl UiTool for UiProjectCreator {
    fn tick(&mut self, ctx: &mut FrameContext) -> bool {
        let mut opened = true;
        let mut created_or_cancelled = false;

        let title = std::mem::take(&mut self.title);
        Window::new(&title) //
            .always_auto_resize(true)
            .resizable(false)
            .collapsible(false)
            .opened(&mut opened)
            .build(ctx.ui, || {
                self.input_project_title(ctx.ui);
                self.input_rom_file_path(ctx.ui);
                self.create_or_cancel(ctx, &mut created_or_cancelled);
                self.project_error_popup(ctx.ui);
            });
        self.title = title;

        let running = opened && !created_or_cancelled;
        if !running {
            log::info!("Closed Project Creator");
        }
        running
    }
}

impl UiProjectCreator {
    pub fn new(id: WindowId) -> Self {
        log::info!("Opened Project Creator");
        let mut myself = UiProjectCreator {
            title: title_with_id("Create new project", id),

            project_title: String::from("My SMW hack"),
            base_rom_path: String::new(),

            err_project_title:    String::new(),
            err_base_rom_path:    String::new(),
            err_project_creation: String::new(),
        };
        myself.handle_rom_file_path();
        myself
    }

    fn input_project_title(&mut self, ui: &Ui) {
        ui.text("Project title:");
        if ui.input_text("##project_title", &mut self.project_title).build() {
            self.handle_project_title();
        }

        if !self.err_project_title.is_empty() {
            ui.text_colored(color::TEXT_ERROR, &self.err_project_title);
        }
    }

    fn handle_project_title(&mut self) {
        if self.project_title.is_empty() {
            self.err_project_title = String::from("Project title cannot be empty.");
        } else {
            self.err_project_title.clear();
        }
    }

    fn input_rom_file_path(&mut self, ui: &Ui) {
        ui.text("Base ROM file:");
        if ui.input_text("##rom_file", &mut self.base_rom_path).build() {
            self.handle_rom_file_path();
        }
        ui.same_line();
        if ui.small_button("Browse...") {
            self.open_file_selector();
        }

        if !self.err_base_rom_path.is_empty() {
            ui.text_colored(color::TEXT_ERROR, &self.err_base_rom_path);
        }
    }

    fn handle_rom_file_path(&mut self) {
        let file_path = Path::new(&self.base_rom_path);
        if !file_path.exists() {
            self.err_base_rom_path = format!("File '{}' does not exist.", self.base_rom_path);
        } else if file_path.is_dir() {
            self.err_base_rom_path = format!("'{}' is not a file.", self.base_rom_path);
        } else {
            self.err_base_rom_path.clear();
        }
    }

    fn open_file_selector(&mut self) {
        log::info!("Opened File Selector");
        use nfd2::Response;
        if let Response::Okay(path) = nfd2::open_file_dialog(Some("smc,sfc"), None) //
            .unwrap_or_else(|e| panic!("Cannot open file selector: {}", e))
        {
            self.base_rom_path = String::from(path.to_str().unwrap());
            self.handle_rom_file_path();
        }
    }

    fn create_or_cancel(&mut self, ctx: &mut FrameContext, created_or_cancelled: &mut bool) {
        if self.no_creation_errors() {
            if ctx.ui.small_button("Create") {
                log::info!("Attempting to create a new project");
                self.handle_project_creation(ctx, created_or_cancelled);
            }
        } else {
            ctx.ui.text_disabled("Create");
        }
        ctx.ui.same_line();
        if ctx.ui.small_button("Cancel") {
            log::info!("Cancelled project creation");
            *created_or_cancelled = true;
        }
    }

    fn handle_project_creation(&mut self, ctx: &mut FrameContext, created_or_cancelled: &mut bool) {
        match SmwRom::from_file(self.base_rom_path) {
            Ok(rom_data) => {
                log::info!("Success creating a new project");
                let project = Project { title: self.project_title.to_string(), rom_data };
                *ctx.project_ref = Some(Rc::new(RefCell::new(project)));
                *created_or_cancelled = true;
                self.err_project_creation.clear();
            }
            Err(err) => {
                log::info!("Failed to create a new project: {}", err);
                self.err_project_creation = err.to_string();
                ctx.ui.open_popup("Error!##project_error");
            }
        }
    }

    fn project_error_popup(&self, ui: &Ui) {
        PopupModal::new("Error!##project_error").always_auto_resize(true).resizable(false).collapsible(false).build(
            ui,
            || {
                ui.text_wrapped(&self.err_project_creation);
                if ui.button_with_size("OK", [tweak!(300.0), tweak!(20.0)]) {
                    ui.close_current_popup();
                }
            },
        );
    }

    fn no_creation_errors(&self) -> bool {
        vec![&self.err_base_rom_path, &self.err_project_title].iter().all(|s| s.is_empty())
    }
}
