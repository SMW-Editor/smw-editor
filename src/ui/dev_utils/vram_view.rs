use egui::{Button, CentralPanel, DragValue, SidePanel, Ui, WidgetText};

use crate::ui::{tool::DockableEditorTool, EditorState};

pub struct UiVramView {
    level_num:      u16,
    blue_pswitch:   bool,
    silver_pswitch: bool,
    on_off_switch:  bool,
}

impl Default for UiVramView {
    fn default() -> Self {
        Self { level_num: 0, blue_pswitch: false, silver_pswitch: false, on_off_switch: false }
    }
}

impl DockableEditorTool for UiVramView {
    fn update(&mut self, ui: &mut Ui, project_ref: &mut EditorState) {
        SidePanel::left("vram_view.left_panel").resizable(false).show_inside(ui, |ui| self.left_panel(ui));
        CentralPanel::default().show_inside(ui, |ui| self.central_panel(ui, project_ref));
    }

    fn title(&self) -> WidgetText {
        "VRAM View".into()
    }
}

impl UiVramView {
    fn left_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.add_enabled(self.level_num > 0, Button::new(egui_phosphor::MINUS)).clicked() {
                self.level_num -= 1;
            }
            ui.add(DragValue::new(&mut self.level_num).clamp_range(0x0..=0x1FF).hexadecimal(3, false, true));
            if ui.add_enabled(self.level_num < 0x1FF, Button::new(egui_phosphor::PLUS)).clicked() {
                self.level_num += 1;
            }
            ui.label("Level");
        });

        ui.checkbox(&mut self.blue_pswitch, "Blue P-Switch");
        ui.checkbox(&mut self.silver_pswitch, "Silver P-Switch");
        ui.checkbox(&mut self.on_off_switch, "ON/OFF Switch");
    }

    fn central_panel(&mut self, ui: &mut Ui, project_ref: &mut EditorState) {
        //
    }
}
