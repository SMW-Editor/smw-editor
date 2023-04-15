mod color;
mod dev_utils;
mod editor_prototypes;
mod project_creator;
mod tab_viewer;
mod tool;

use std::sync::Arc;

use eframe::Frame;
use egui::*;
use egui_dock::{DockArea, StyleBuilder, Tree};
use rfd::FileDialog;
use smwe_project::ProjectRef;

use crate::ui::{
    dev_utils::{
        address_converter::UiAddressConverter,
        disassembler::UiDisassembler,
        gfx_viewer::UiGfxViewer,
        palette_viewer::UiPaletteViewer,
        rom_info::UiRomInfo,
        tiles16x16::UiTiles16x16,
    },
    editor_prototypes::{block_editor::UiBlockEditor, code_editor::UiCodeEditor},
    project_creator::UiProjectCreator,
    tab_viewer::EditorToolTabViewer,
    tool::{DockableEditorTool, DockableEditorToolEnum},
};

pub struct UiMainWindow {
    project: Option<ProjectRef>,
    style:   Arc<Style>,

    project_creator:    Option<UiProjectCreator>,
    dock_tree:          Tree<DockableEditorToolEnum>,
    last_open_tool_idx: usize,
}

impl eframe::App for UiMainWindow {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.set_style(self.style.clone());
        CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.dark_mode = true;

            self.main_menu_bar(ctx, frame);

            DockArea::new(&mut self.dock_tree)
                .style(StyleBuilder::from_egui(&ctx.style()).with_tab_scroll_area(false).build())
                .show(ctx, &mut EditorToolTabViewer { project_ref: &mut self.project });

            if let Some(project_creator) = &mut self.project_creator {
                if !project_creator.update(ui, &mut self.project) {
                    self.project_creator = None;
                }
            }
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
            project_creator: None,
            dock_tree: Tree::default(),
            last_open_tool_idx: 0,
        }
    }

    fn open_tool<ToolType>(&mut self, tool: ToolType)
    where
        ToolType: 'static + DockableEditorTool + Into<DockableEditorToolEnum>,
    {
        log::info!("Opened {}", tool.title().text());
        if self.last_open_tool_idx < usize::MAX {
            self.dock_tree.push_to_focused_leaf(tool.into());
            self.last_open_tool_idx += 1;
        }
    }

    fn main_menu_bar(&mut self, ctx: &Context, frame: &mut Frame) {
        let is_project_loaded = self.project.is_some();

        TopBottomPanel::top("main_top_bar").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New project").clicked() {
                        self.project_creator = Some(UiProjectCreator::default());
                        ui.close_menu();
                    }
                    if ui.button("Save ROM dump").clicked() {
                        ui.close_menu();

                        match FileDialog::new()
                            .add_filter("Text File (*.txt)", &["txt"])
                            .add_filter("All (*.*)", &["*"])
                            .save_file()
                        {
                            Some(path) => {
                                use std::fmt::Write;
                                let mut dump = String::with_capacity(4096);
                                write!(&mut dump, "{:?}", self.project.as_ref().unwrap().borrow().rom_data.disassembly)
                                    .unwrap();
                                std::fs::write(path, dump).unwrap();
                            }
                            None => log::error!("Cannot save ROM dump."),
                        }
                    }
                    if ui.button("Exit").clicked() {
                        frame.close();
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if ui.button("Address converter").clicked() {
                        self.open_tool(UiAddressConverter::default());
                        ui.close_menu();
                    }
                    ui.set_enabled(is_project_loaded);
                    if ui.button("Internal ROM Header").clicked() {
                        let rom_info =
                            UiRomInfo::new(&self.project.as_ref().unwrap().borrow().rom_data.internal_header);
                        self.open_tool(rom_info);
                        ui.close_menu();
                    }
                    if ui.button("Disassembly").clicked() {
                        self.open_tool(UiDisassembler::default());
                        ui.close_menu();
                    }
                    if ui.button("Color palettes").clicked() {
                        self.open_tool(UiPaletteViewer::default());
                        ui.close_menu();
                    }
                    if ui.button("GFX files").clicked() {
                        self.open_tool(UiGfxViewer::default());
                        ui.close_menu();
                    }
                    if ui.button("16x16 tiles").clicked() {
                        self.open_tool(UiTiles16x16::default());
                        ui.close_menu();
                    }
                });

                ui.menu_button("Prototypes", |ui| {
                    if ui.button("Block editor").clicked() {
                        self.open_tool(UiBlockEditor::default());
                        ui.close_menu();
                    }
                    if ui.button("Code editor").clicked() {
                        self.open_tool(UiCodeEditor::default());
                        ui.close_menu();
                    }
                });
            });
        });
    }
}
