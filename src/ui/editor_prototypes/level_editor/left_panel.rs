use egui::{vec2, DragValue, Slider, Ui};
use smwe_widgets::value_switcher::{ValueSwitcher, ValueSwitcherButtons};

use super::UiLevelEditor;

impl UiLevelEditor {
    pub(super) fn left_panel(&mut self, ui: &mut Ui) {
        if cfg!(debug_assertions) {
            ui.add_space(ui.spacing().item_spacing.y);
            ui.group(|ui| {
                ui.allocate_space(vec2(ui.available_width(), 0.));
                self.debug_panel(ui);
            });
        }
    }

    fn debug_panel(&mut self, ui: &mut Ui) {
        let mut need_update_level = false;
        let mut need_update = false;
        need_update_level |= {
            let switcher = ValueSwitcher::new(&mut self.level_num, "Level", ValueSwitcherButtons::MinusPlus)
                .range(0..=0x1FF)
                .hexadecimal(3, false, true);
            ui.add(switcher).changed()
        };
        need_update_level |= {
            let switcher = ValueSwitcher::new(&mut self.sprite_id, "Sprite ID", ValueSwitcherButtons::MinusPlus)
                .range(0..=0xFF)
                .hexadecimal(2, false, true);
            ui.add(switcher).changed()
        };
        ui.horizontal(|ui| {
            need_update |= ui
                .add(DragValue::new(&mut self.palette_line).clamp_range(0x0..=0xF).hexadecimal(1, false, true))
                .changed();
            ui.label("Palette");
        });
        need_update |= ui.checkbox(&mut self.blue_pswitch, "Blue P-Switch").changed();
        need_update |= ui.checkbox(&mut self.silver_pswitch, "Silver P-Switch").changed();
        need_update |= ui.checkbox(&mut self.on_off_switch, "ON/OFF Switch").changed();
        ui.checkbox(&mut self.run_sprites, "Run sprites");
        if ui.button("»").clicked() {
            self.update_cpu_sprite_id();
            // self.draw_sprites(state, ui.ctx());
        }

        ui.add(Slider::new(&mut self.zoom, 1.0..=3.0).step_by(0.25));

        ui.checkbox(&mut self.always_show_grid, "Always show grid");

        if need_update_level {
            self.update_cpu();
            self.update_cpu_sprite_id();
        }
        if need_update || need_update_level {
            self.update_renderer();
        }
    }
}
