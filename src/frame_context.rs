use eframe::{egui::Context, Frame};
use smwe_project::ProjectRef;

pub struct FrameContext<'f, 'p> {
    pub project_ref: &'f mut Option<ProjectRef<'p>>,
    pub egui_ctx:    &'f Context,
    pub frame:       &'f mut Frame,
}
