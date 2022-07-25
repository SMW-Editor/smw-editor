use eframe::{egui::Context, Frame};
use glium::Display;
use imgui::Ui as ImUi;
use imgui_glium_renderer::Renderer;
use smwe_project::ProjectRef;

pub struct FrameContext<'f, 'ui> {
    pub project_ref: &'f mut Option<ProjectRef>,
    pub renderer:    &'f mut Renderer,
    pub display:     &'f Display,
    pub ui:          &'f ImUi<'ui>,
}

pub struct EFrameContext<'f> {
    pub project_ref: &'f mut Option<ProjectRef>,
    pub egui_ctx:    &'f Context,
    pub frame:       &'f mut Frame,
}
