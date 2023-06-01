use egui::*;
use inline_tweak::tweak;
use smwe_widgets::value_switcher::{ValueSwitcher, ValueSwitcherButtons};

use crate::ui::{editor_prototypes::sprite_map_editor::UiSpriteMapEditor, tool::DockableEditorTool};

impl DockableEditorTool for UiSpriteMapEditor {
    fn update(&mut self, ui: &mut Ui) {
        if !self.initialized {
            self.update_cpu();
            self.update_renderers();
            self.pixels_per_point = ui.ctx().pixels_per_point();
            self.initialized = true;
        }

        self.handle_keyboard(ui);

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
    pub(super) fn left_panel(&mut self, ui: &mut Ui) {
        self.tile_selector(ui);
        ui.add_space(tweak!(10.));
        self.tile_selection_preview(ui);
        ui.add_space(tweak!(10.));
        self.palette_row_selector(ui);
        ui.add_space(tweak!(10.));
        self.debug_toggles(ui);
    }

    pub(super) fn central_panel(&mut self, ui: &mut Ui) {
        self.top_menu(ui);
        self.editing_area(ui);
    }

    pub(super) fn top_menu(&mut self, ui: &mut Ui) {
        Frame::menu(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                let level_switcher = ValueSwitcher::new(&mut self.level_num, "Level", ValueSwitcherButtons::MinusPlus)
                    .range(0..=0x1FF)
                    .hexadecimal(3, false, true);
                if ui.add(level_switcher).changed() {
                    self.update_cpu();
                    self.update_renderers();
                }
                ui.separator();

                self.editing_mode_selector(ui);
                ui.separator();

                ui.horizontal(|ui| {
                    let zoom_slider = Slider::new(&mut self.zoom, 1.0..=4.0).step_by(0.25).suffix("x");
                    ui.add(zoom_slider);
                    ui.label("Zoom");
                });
                ui.separator();

                ui.checkbox(&mut self.always_show_grid, "Always show grid");

                ui.add_space(ui.available_width());
            });
        });
    }
}
