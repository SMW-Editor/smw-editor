use imgui::{
    Condition,
    Window,
    Ui,

    im_str,
};

pub fn run(ui: &mut Ui) {
    Window::new(im_str!("Test"))
        .size([400.0, 300.0], Condition::FirstUseEver)
        .build(ui, || {
            ui.text(im_str!("The app runs!"));
        });
}