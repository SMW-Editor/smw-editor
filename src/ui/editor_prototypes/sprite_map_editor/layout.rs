use egui::*;

use crate::ui::{editor_prototypes::sprite_map_editor::UiSpriteMapEditor, tool::DockableEditorTool};

impl DockableEditorTool for UiSpriteMapEditor {
    fn update(&mut self, ui: &mut Ui) {
        if !self.initialized {
            self.update_cpu();
            self.update_renderers();
            self.pixels_per_point = ui.ctx().pixels_per_point();
            self.initialized = true;
        }

        self.handle_input(ui);

        SidePanel::left("sprite_map_editor.left_panel").resizable(false).show_inside(ui, |ui| self.left_panel(ui));
        CentralPanel::default().show_inside(ui, |ui| self.central_panel(ui));
    }

    fn title(&self) -> WidgetText {
        "Sprite Tile Editor".into()
    }

    fn on_closed(&mut self) {
        self.destroy();
    }
}

impl UiSpriteMapEditor {
    fn left_panel(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.allocate_space(vec2(ui.available_width(), 0.));
            self.tile_selector(ui);
            self.tile_selection_preview(ui);
        });
        ui.group(|ui| {
            ui.allocate_space(vec2(ui.available_width(), 0.));
            self.palette_row_selector(ui);
        });
        ui.group(|ui| {
            ui.allocate_space(vec2(ui.available_width(), 0.));
            self.debug_toggles(ui);
        });
    }

    fn central_panel(&mut self, ui: &mut Ui) {
        Frame::menu(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                self.editor_toolbar_menu(ui);
            });
        });
        self.editing_area(ui);
    }
}
