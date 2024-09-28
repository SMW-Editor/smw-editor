use egui::*;

use super::{keyboard_shortcuts::*, UiSpriteMapEditor};

impl UiSpriteMapEditor {
    pub(super) fn menu_bar(&mut self, ui: &mut Ui) {
        ui.menu_button("File", |ui| self.menu_file(ui));
        ui.menu_button("Edit", |ui| self.menu_edit(ui));
        ui.menu_button("View", |ui| self.menu_view(ui));
    }

    fn menu_file(&mut self, ui: &mut Ui) {
        if ui.add(Button::new("New").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_NEW))).clicked() {
            self.create_new_map();
            ui.close_menu();
        }
        if ui.add(Button::new("Save").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_SAVE))).clicked() {
            self.save_map_dialog();
            ui.close_menu();
        }
        if ui.add(Button::new("Open").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_OPEN))).clicked() {
            self.open_map_dialog();
            ui.close_menu();
        }
    }

    fn menu_edit(&mut self, ui: &mut Ui) {
        if ui
            .add_enabled(
                self.sprite_tiles.can_undo(),
                Button::new("Undo").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_UNDO)),
            )
            .clicked()
        {
            self.handle_undo();
            ui.close_menu();
        }
        if ui
            .add_enabled(
                self.sprite_tiles.can_redo(),
                Button::new("Redo").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_REDO)),
            )
            .clicked()
        {
            self.handle_redo();
            ui.close_menu();
        }
        if ui.add(Button::new("Copy").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_COPY))).clicked() {
            ui.output_mut(|output| self.handle_copy(output));
            ui.close_menu();
        }
        if ui.add(Button::new("Cut").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_CUT))).clicked() {
            ui.output_mut(|output| self.handle_cut(output));
            ui.close_menu();
        }
        if ui.add(Button::new("Select all").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_SELECT_ALL))).clicked() {
            self.select_all_tiles();
            ui.close_menu();
        }
        if ui
            .add_enabled(
                !self.selected_sprite_tile_indices.is_empty(),
                Button::new("Unselect all").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_UNSELECT_ALL)),
            )
            .clicked()
        {
            self.unselect_all_tiles();
            ui.close_menu();
        }
    }

    fn menu_view(&mut self, ui: &mut Ui) {
        if ui
            .add_enabled(
                self.zoom < Self::MAX_ZOOM,
                Button::new("Zoom +25%").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_ZOOM_IN)),
            )
            .clicked()
        {
            self.zoom += 0.25;
            ui.close_menu();
        }
        if ui
            .add_enabled(
                self.zoom > Self::MIN_ZOOM,
                Button::new("Zoom -25%").shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_ZOOM_OUT)),
            )
            .clicked()
        {
            self.zoom -= 0.25;
            ui.close_menu();
        }
        if ui.checkbox(&mut self.always_show_grid, "Always show grid").clicked() {
            ui.close_menu();
        }
    }
}
