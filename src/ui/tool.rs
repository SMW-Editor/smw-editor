#![allow(clippy::enum_variant_names)]

use eframe::egui::Ui;
use egui::WidgetText;

pub trait DockableEditorTool {
    fn update(&mut self, ui: &mut Ui);
    fn title(&self) -> WidgetText;
    fn on_closed(&mut self) {}
}
