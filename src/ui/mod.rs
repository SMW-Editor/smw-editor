mod dev_utils;
mod editing_mode;
mod editor_prototypes;
mod project_creator;
mod style;
mod tab_viewer;
mod tool;

use std::sync::Arc;

use eframe::{CreationContext, Frame};
use egui::*;
use egui_dock::{DockArea, DockState, Style as DockStyle};
use egui_phosphor::Variant;
use smwe_emu::rom::Rom;

use crate::{
    project::{Project, ProjectRef},
    ui::{
        dev_utils::address_converter::UiAddressConverter,
        editor_prototypes::{
            block_editor::UiBlockEditor,
            level_editor::UiLevelEditor,
            sprite_map_editor::UiSpriteMapEditor,
        },
        project_creator::UiProjectCreator,
        tab_viewer::EditorToolTabViewer,
        tool::DockableEditorTool,
    },
};

pub struct UiMainWindow {
    gl:                 Arc<glow::Context>,
    project_creator:    Option<UiProjectCreator>,
    dock_style:         DockStyle,
    dock_state:         DockState<Box<dyn DockableEditorTool>>,
    last_open_tool_idx: usize,
}

impl UiMainWindow {
    pub fn new(project: Option<ProjectRef>, cc: &CreationContext) -> Self {
        let mut fonts = FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_visuals(Visuals::dark());

        if let Some(project) = project {
            cc.egui_ctx.data_mut(|data| {
                let project = project.borrow();
                data.insert_temp(Project::project_title_id(), project.title.clone());
                data.insert_temp(Project::rom_id(), Arc::clone(&project.rom));
            });
        }

        let mut dock_style = DockStyle::from_egui(&cc.egui_ctx.style());
        dock_style.tab.tab_body.inner_margin = Margin::ZERO;

        Self {
            gl: Arc::clone(cc.gl.as_ref().expect("must use the glow renderer")),
            project_creator: None,
            dock_style,
            dock_state: DockState::new(vec![]),
            last_open_tool_idx: 0,
        }
    }
}

impl eframe::App for UiMainWindow {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.main_menu_bar(ctx);

            DockArea::new(&mut self.dock_state).style(self.dock_style.clone()).show(ctx, &mut EditorToolTabViewer);

            if let Some(project_creator) = &mut self.project_creator {
                if !project_creator.update(ui) {
                    self.project_creator = None;
                }
            }
        });
    }
}

impl UiMainWindow {
    fn open_tool<ToolType>(&mut self, tool: ToolType)
    where
        ToolType: 'static + DockableEditorTool,
    {
        if self.last_open_tool_idx < usize::MAX {
            log::info!("Opened {}", tool.title().text());
            self.dock_state.push_to_focused_leaf(Box::new(tool));
            self.last_open_tool_idx += 1;
        }
    }

    fn main_menu_bar(&mut self, ctx: &Context) {
        let rom: Option<Arc<Rom>> = ctx.data(|data| data.get_temp(Id::new("rom")));

        TopBottomPanel::top("main_top_bar").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New project").clicked() {
                        self.project_creator = Some(UiProjectCreator::default());
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if ui.button("Address converter").clicked() {
                        self.open_tool(UiAddressConverter::default());
                        ui.close_menu();
                    }
                });

                ui.menu_button("Prototypes", |ui| {
                    if ui.button("Block editor").clicked() {
                        self.open_tool(UiBlockEditor::default());
                        ui.close_menu();
                    }
                    if ui.add_enabled(rom.is_some(), Button::new("Level editor")).clicked() {
                        self.open_tool(UiLevelEditor::new(Arc::clone(&self.gl), rom.clone().unwrap()));
                        ui.close_menu();
                    }
                    if ui.add_enabled(rom.is_some(), Button::new("Sprite map editor")).clicked() {
                        self.open_tool(UiSpriteMapEditor::new(Arc::clone(&self.gl), rom.clone().unwrap()));
                        ui.close_menu();
                    }
                });
            });
        });
    }
}
