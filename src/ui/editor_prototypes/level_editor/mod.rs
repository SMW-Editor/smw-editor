mod level_renderer;

use std::sync::{Arc, Mutex};

use egui::{CentralPanel, DragValue, SidePanel, Ui, WidgetText, *};
use egui_glow::CallbackFn;
use smwe_emu::{emu::CheckedMem, Cpu};
use smwe_render::color::Abgr1555;
use smwe_widgets::value_switcher::{ValueSwitcher, ValueSwitcherButtons};

use crate::ui::{
    editor_prototypes::level_editor::level_renderer::LevelRenderer,
    tool::DockableEditorTool,
    EditorState,
};

pub struct UiLevelEditor {
    gl:             Arc<glow::Context>,
    level_renderer: Arc<Mutex<LevelRenderer>>,
    initialized:    bool,
    offset:         Vec2,
    level_num:      u16,
    blue_pswitch:   bool,
    silver_pswitch: bool,
    on_off_switch:  bool,
    run_sprites:    bool,
    palette_line:   u8,
    sprite_id:      u8,
    timestamp:      std::time::Instant,
    zoom:           f32,
}

impl UiLevelEditor {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        let level_renderer = Arc::new(Mutex::new(LevelRenderer::new(&gl)));
        Self {
            gl,
            level_renderer,
            initialized: false,
            level_num: 0,
            offset: Vec2::ZERO,
            blue_pswitch: false,
            silver_pswitch: false,
            on_off_switch: false,
            run_sprites: true,
            palette_line: 0,
            sprite_id: 0,
            timestamp: std::time::Instant::now(),
            zoom: 1.,
        }
    }
}

// UI
impl DockableEditorTool for UiLevelEditor {
    fn update(&mut self, ui: &mut Ui, state: &mut EditorState) {
        if !self.initialized {
            self.update_cpu(state);
            self.update_cpu_sprite(state);
            self.update_renderer(state);
            self.initialized = true;
        }
        SidePanel::left("level_editor.left_panel").resizable(false).show_inside(ui, |ui| self.left_panel(ui, state));
        CentralPanel::default().frame(Frame::none()).show_inside(ui, |ui| self.central_panel(ui, state));

        // Auto-play animations
        let ft = std::time::Duration::from_secs_f32(1. / 60.);
        let now = std::time::Instant::now();
        while now - self.timestamp > ft {
            self.timestamp += ft;
            self.update_timers(state);
            self.update_anim_frame(state);
            if self.run_sprites {
                self.update_cpu_sprite(state);
                //let cpu = state.cpu.as_mut().unwrap();
                //self.level_renderer.lock().unwrap().upload_level(&self.gl, cpu);
            }
        }
        ui.ctx().request_repaint();
    }

    fn title(&self) -> WidgetText {
        "Level Editor".into()
    }

    fn on_closed(&mut self) {
        self.level_renderer.lock().unwrap().destroy(&self.gl);
    }
}

impl UiLevelEditor {
    fn left_panel(&mut self, ui: &mut Ui, state: &mut EditorState) {
        let mut need_update_level = false;
        let mut need_update = false;
        need_update_level |= {
            let switcher = ValueSwitcher::new(&mut self.level_num, "Level", ValueSwitcherButtons::MinusPlus)
                .range(0..=0x1FF)
                .hexadecimal(3, false, true);
            ui.add(switcher).changed()
        };
        need_update_level |= {
            let switcher = ValueSwitcher::new(&mut self.sprite_id, "Sprite ID", ValueSwitcherButtons::MinusPlus)
                .range(0..=0xFF)
                .hexadecimal(2, false, true);
            ui.add(switcher).changed()
        };
        ui.horizontal(|ui| {
            need_update |= ui
                .add(DragValue::new(&mut self.palette_line).clamp_range(0x0..=0xF).hexadecimal(1, false, true))
                .changed();
            ui.label("Palette");
        });
        need_update |= ui.checkbox(&mut self.blue_pswitch, "Blue P-Switch").changed();
        need_update |= ui.checkbox(&mut self.silver_pswitch, "Silver P-Switch").changed();
        need_update |= ui.checkbox(&mut self.on_off_switch, "ON/OFF Switch").changed();
        ui.checkbox(&mut self.run_sprites, "Run sprites");
        if ui.button("»").clicked() {
            self.update_cpu_sprite_id(state);
            // self.draw_sprites(state, ui.ctx());
        }

        ui.add(Slider::new(&mut self.zoom, 1.0..=3.0).step_by(0.25));

        if need_update_level {
            self.update_cpu(state);
            self.update_cpu_sprite_id(state);
        }
        if need_update || need_update_level {
            self.update_renderer(state);
        }
    }

    fn central_panel(&mut self, ui: &mut Ui, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap();
        let bg_color = cpu.mem.load_u16(0x7E0701);
        let bg_color = Color32::from(Abgr1555(bg_color));
        CentralPanel::default().frame(Frame::none().inner_margin(0.).fill(bg_color)).show_inside(ui, |ui| {
            let level_renderer = Arc::clone(&self.level_renderer);
            let (rect, response) =
                ui.allocate_exact_size(vec2(ui.available_width(), ui.available_height()), Sense::click_and_drag());
            let screen_size = rect.size() * ui.ctx().pixels_per_point();

            let zoom = self.zoom;
            if response.dragged_by(PointerButton::Middle) {
                let mut r = level_renderer.lock().unwrap();
                let delta = response.drag_delta();
                self.offset += delta / zoom;
                r.set_offset(self.offset);
            }

            ui.painter().add(PaintCallback {
                rect,
                callback: Arc::new(CallbackFn::new(move |_info, painter| {
                    level_renderer.lock().expect("Cannot lock mutex on level_renderer").paint(
                        painter.gl(),
                        screen_size,
                        zoom,
                    );
                })),
            });
        });
    }
}

// Internals
impl UiLevelEditor {
    fn update_cpu(&mut self, state: &mut EditorState) {
        let project = state.project.as_ref().unwrap().borrow();
        let mut cpu = Cpu::new(CheckedMem::new(project.rom.clone()));
        drop(project);
        smwe_emu::emu::decompress_sublevel(&mut cpu, self.level_num);
        println!("Updated CPU");
        self.level_renderer.lock().unwrap().upload_level(&self.gl, &mut cpu);
        state.cpu = Some(cpu);
    }

    fn update_timers(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        let m = cpu.mem.load_u8(0x13);
        cpu.mem.store_u8(0x13, m.wrapping_add(1));
        let m = cpu.mem.load_u8(0x14);
        cpu.mem.store_u8(0x14, m.wrapping_add(1));
    }

    fn update_cpu_sprite(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        cpu.mem.wram[0x300..0x400].fill(0xE0);
        smwe_emu::emu::exec_sprites(cpu);
        self.level_renderer.lock().unwrap().upload_sprites(&self.gl, cpu);
    }

    fn update_cpu_sprite_id(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        let mut cpu = cpu.clone();
        cpu.mem.wram[0x300..0x400].fill(0xE0);
        smwe_emu::emu::exec_sprite_id(&mut cpu, self.sprite_id);
        self.level_renderer.lock().unwrap().upload_sprites(&self.gl, &mut cpu);
    }

    fn update_anim_frame(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        cpu.mem.store_u8(0x14AD, self.blue_pswitch as u8);
        cpu.mem.store_u8(0x14AE, self.silver_pswitch as u8);
        cpu.mem.store_u8(0x14AF, self.on_off_switch as u8);
        smwe_emu::emu::fetch_anim_frame(cpu);
        self.level_renderer.lock().expect("Cannot lock mutex on level_renderer").upload_gfx(&self.gl, &cpu.mem.vram);
    }

    fn update_renderer(&mut self, state: &mut EditorState) {
        self.update_anim_frame(state);

        // should be set already
        let cpu = state.cpu.as_mut().unwrap();
        let level_renderer = self.level_renderer.lock().expect("Cannot lock mutex on level_renderer");
        level_renderer.upload_palette(&self.gl, &cpu.mem.cgram);
        level_renderer.upload_gfx(&self.gl, &cpu.mem.vram);
    }
}
