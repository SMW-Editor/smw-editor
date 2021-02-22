use imgui::Ui;

pub trait UiTool {
    fn run(&mut self, ui: &Ui) -> bool;
}
