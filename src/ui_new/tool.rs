use eframe::egui::Ui;
use crate::frame_context::EFrameContext;

pub trait UiTool {
    fn update(&mut self, ui: &mut Ui, ctx: &mut EFrameContext) -> bool;
}
