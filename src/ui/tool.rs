use eframe::egui::Ui;

use crate::frame_context::FrameContext;

pub trait UiTool {
    fn update(&mut self, ui: &mut Ui, ctx: &mut FrameContext) -> bool;
}
