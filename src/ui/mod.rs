mod color;
mod dev_utils;
mod editor_prototypes;
mod project_creator;
mod tab_viewer;
mod tool;

use std::{rc::Rc, sync::Arc};

use eframe::{CreationContext, Frame};
use egui::*;
use egui_dock::{DockArea, Style as DockStyle, Tree};
use rfd::FileDialog;
use smwe_emu::emu::CheckedMem;
use smwe_project::ProjectRef;
use wdc65816::Cpu;

use crate::ui::{
    dev_utils::{
        address_converter::UiAddressConverter,
        disassembler::UiDisassembler,
        gfx_viewer::UiGfxViewer,
        palette_viewer::UiPaletteViewer,
        rom_info::UiRomInfo,
        tiles16x16::UiTiles16x16,
    },
    editor_prototypes::{block_editor::UiBlockEditor, code_editor::UiCodeEditor, level_editor::UiLevelEditor},
    project_creator::UiProjectCreator,
    tab_viewer::EditorToolTabViewer,
    tool::{DockableEditorTool, DockableEditorToolEnum},
};

#[derive(Debug)]
pub struct EditorState {
    project: Option<ProjectRef>,
    cpu:     Option<Cpu<CheckedMem>>,
    gl:      Arc<glow::Context>,
}

#[derive(Debug)]
pub struct UiMainWindow {
    state:              EditorState,
    project_creator:    Option<UiProjectCreator>,
    dock_tree:          Tree<DockableEditorToolEnum>,
    last_open_tool_idx: usize,
}

impl UiMainWindow {
    pub fn new(project: Option<ProjectRef>, cc: &CreationContext) -> Self {
        let mut fonts = FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_visuals(Visuals::dark());

        Self {
            state:              EditorState {
                project,
                cpu: None,
                gl: Arc::clone(cc.gl.as_ref().expect("must use the glow renderer")),
            },
            project_creator:    None,
            dock_tree:          Tree::default(),
            last_open_tool_idx: 0,
        }
    }
}

impl eframe::App for UiMainWindow {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if let Some(project) = &self.state.project {
            self.state.cpu.get_or_insert_with(|| {
                let rom = Rc::clone(&project.borrow().rom);
                let mem = CheckedMem::new(rom);
                Cpu::new(mem)
            });
        }

        CentralPanel::default().show(ctx, |ui| {
            self.main_menu_bar(ctx, frame);

            DockArea::new(&mut self.dock_tree)
                .style(DockStyle::from_egui(&ctx.style()))
                .scroll_area_in_tabs(false)
                .show(ctx, &mut EditorToolTabViewer { state: &mut self.state });

            if let Some(project_creator) = &mut self.project_creator {
                if !project_creator.update(ui, &mut self.state) {
                    self.project_creator = None;
                }
            }
        });
    }
}

impl UiMainWindow {
    fn open_tool<ToolType>(&mut self, tool: ToolType)
    where
        ToolType: 'static + DockableEditorTool + Into<DockableEditorToolEnum>,
    {
        if self.last_open_tool_idx < usize::MAX {
            log::info!("Opened {}", tool.title().text());
            self.dock_tree.push_to_focused_leaf(tool.into());
            self.last_open_tool_idx += 1;
        }
    }

    fn main_menu_bar(&mut self, ctx: &Context, frame: &mut Frame) {
        let is_project_loaded = self.state.project.is_some();

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
                                write!(
                                    &mut dump,
                                    "{:?}",
                                    self.state.project.as_ref().unwrap().borrow().old_rom_data.disassembly
                                )
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
                            UiRomInfo::new(&self.state.project.as_ref().unwrap().borrow().old_rom_data.internal_header);
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
                    if ui.button("Level editor").clicked() {
                        self.open_tool(UiLevelEditor::default());
                        ui.close_menu();
                    }
                });
            });
        });
    }
}
