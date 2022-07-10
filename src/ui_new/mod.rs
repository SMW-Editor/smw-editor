mod color;
mod dev_utils;
mod project_creator;
mod tool;

use std::sync::Arc;
use eframe::{
    egui::{self, Context, Style},
    Frame,
};
use eframe::egui::Ui;
use smwe_project::ProjectRef;
use crate::{
    frame_context::EFrameContext,
    ui_new::{
        tool::UiTool,
        dev_utils::address_converter::UiAddressConverter,
        project_creator::UiProjectCreator,
    },
};
use crate::ui_new::dev_utils::rom_info::UiRomInfo;

pub struct UiMainWindow {
    project: Option<ProjectRef>,
    style: Arc<Style>,

    tools: Vec<Box<dyn UiTool>>,
    last_open_tool_idx: usize,
}

impl eframe::App for UiMainWindow {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.set_style(self.style.clone());
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.dark_mode = true;
            self.main_menu_bar(ctx, frame);
            self.update_tools(ctx, frame, ui);
        });
    }
}

impl UiMainWindow {
    pub fn new(project: Option<ProjectRef>) -> Self {
        let mut style = Style::default();
        style.visuals.dark_mode = true;
        Self {
            project,
            style: Arc::new(style),
            tools: vec![],
            last_open_tool_idx: 0,
        }
    }

    fn open_tool<ToolType: 'static + UiTool>(&mut self, tool: ToolType) {
        if self.last_open_tool_idx < usize::MAX {
            self.tools.push(Box::new(tool));
            self.last_open_tool_idx += 1;
        }
    }

    fn update_tools(&mut self, ctx: &Context, frame: &mut Frame, ui: &mut Ui) {
        let mut frame_ctx = EFrameContext {
            project_ref: &mut self.project,
            ctx,
            frame,
        };
        let mut tools_to_close = vec![];
        for (i, tool) in self.tools.iter_mut().enumerate() {
            if !tool.update(ui, &mut frame_ctx) {
                tools_to_close.push(i);
            }
        }
        for i in tools_to_close.into_iter().rev() {
            self.tools.swap_remove(i);
        }
    }

    fn main_menu_bar(&mut self, ctx: &Context, frame: &mut Frame) {
        use egui::Button;
        let is_project_loaded = self.project.is_some();

        egui::TopBottomPanel::top("main_top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New project").clicked() {
                        self.open_tool(UiProjectCreator::default());
                    }
                    if ui.add_enabled(is_project_loaded, Button::new("Save ROM dump")).clicked() {
                        use nfd2::Response;
                        if let Response::Okay(path) =
                            nfd2::open_save_dialog(Some("txt"), None) //
                                .unwrap_or_else(|e| panic!("Cannot open file selector: {}", e))
                        {
                            use std::fmt::Write;
                            let mut dump = String::with_capacity(4096);
                            write!(&mut dump, "{:?}", self.project.as_ref().unwrap().borrow().rom_data.disassembly).unwrap();
                            std::fs::write(path, dump).unwrap();
                        }
                    }
                    if ui.button("Exit").clicked() {
                        frame.quit();
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if ui.button("Address converter").clicked() {
                        self.open_tool(UiAddressConverter::default());
                    }
                    if ui.add_enabled(is_project_loaded, Button::new("Internal ROM Header")).clicked() {
                        let rom_info = UiRomInfo::new(&self.project.as_ref().unwrap().borrow().rom_data.internal_header);
                        self.open_tool(rom_info);
                    }
                    if ui.add_enabled(is_project_loaded, Button::new("Color palettes")).clicked() {

                    }
                    if ui.add_enabled(is_project_loaded, Button::new("GFX files")).clicked() {

                    }
                    if ui.add_enabled(is_project_loaded, Button::new("Disassembly")).clicked() {

                    }
                });
            });
        });
    }
}
