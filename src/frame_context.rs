use glium::Display;
use imgui::Ui;
use imgui_glium_renderer::Renderer;
use nsmwe_project::ProjectRef;

pub struct FrameContext<'f, 'ui> {
    pub project_ref: &'f mut Option<ProjectRef>,
    pub renderer:    &'f mut Renderer,
    pub display:     &'f Display,
    pub ui:          &'f Ui<'ui>,
}
