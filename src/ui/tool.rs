use imgui::Ui;
use imgui_glium_renderer::Renderer;

pub trait UiTool {
    fn tick(&mut self, ui: &Ui, renderer: &mut Renderer) -> bool;
}
