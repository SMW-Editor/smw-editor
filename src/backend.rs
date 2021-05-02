use std::time::Instant;

use glium::{
    glutin,
    glutin::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    Display,
    Surface,
};
use imgui::Context;
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use smwe_project::ProjectRef;

use crate::frame_context::FrameContext;

pub struct Backend {
    event_loop: EventLoop<()>,
    display:    Display,
    context:    Context,
    platform:   WinitPlatform,
    renderer:   Renderer,
}

impl Backend {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        log::info!("Setting up the rendering backend");

        let event_loop = EventLoop::new();

        let glutin_context = glutin::ContextBuilder::new() //
            .with_vsync(true);

        let size = LogicalSize::new(width, height);
        let window_builder = WindowBuilder::new() //
            .with_title(title)
            .with_inner_size(size);

        let display = Display::new(window_builder, glutin_context, &event_loop) //
            .expect("Failed to create Display.");

        let mut context = Context::create();

        let platform = {
            let mut platform = WinitPlatform::init(&mut context);
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(context.io_mut(), window, HiDpiMode::Rounded);
            platform
        };

        let renderer = Renderer::init(&mut context, &display).expect("Failed to create Renderer.");

        Backend { event_loop, display, context, platform, renderer }
    }

    pub fn run<AppCode>(self, mut app_code: AppCode, mut project_ref: Option<ProjectRef>)
    where
        AppCode: 'static + FnMut(&mut FrameContext) -> bool,
    {
        let Backend { event_loop, display, mut context, mut platform, mut renderer, .. } = self;
        let mut last_frame = Instant::now();

        log::info!("Starting the main loop");
        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                context.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                let window = gl_window.window();
                platform.prepare_frame(context.io_mut(), window).expect("Failed to prepare frame.");
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let ui = context.frame();
                let mut ctx = FrameContext {
                    project_ref: &mut project_ref,
                    renderer:    &mut renderer,
                    display:     &display,
                    ui:          &ui,
                };
                if app_code(&mut ctx) {
                    let gl_window = display.gl_window();
                    let mut target = display.draw();
                    target.clear_color(0.0, 0.0, 0.0, 1.0);
                    platform.prepare_render(&ui, gl_window.window());
                    let draw_data = ui.render();
                    renderer.render(&mut target, draw_data).expect("Failed to render a frame.");
                    target.finish().expect("Failed to swap buffers.");
                } else {
                    log::info!("Exiting");
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                log::info!("Exiting");
                *control_flow = ControlFlow::Exit;
            }
            event => {
                let gl_window = display.gl_window();
                let window = gl_window.window();
                platform.handle_event(context.io_mut(), window, &event);
            }
        });
    }
}
