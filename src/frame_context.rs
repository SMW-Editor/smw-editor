use eframe::{egui::Context, Frame};
use smwe_project::ProjectRef;

pub struct FrameContext<'f> {
    pub project_ref: &'f mut Option<ProjectRef>,
    pub egui_ctx:    &'f Context,
    pub frame:       &'f mut Frame,
}
