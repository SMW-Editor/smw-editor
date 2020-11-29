extern crate glium;
extern crate imgui;
extern crate imgui_glium_renderer;
extern crate imgui_winit_support;
extern crate inline_tweak;
extern crate nfd2;

pub mod app;
pub mod project;
pub mod ui;

mod backend;

pub use crate::{
    app::App,
    project::{
        OptProjectRef,
        Project,
    },
    ui::*,
};