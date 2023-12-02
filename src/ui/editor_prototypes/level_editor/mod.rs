mod level_renderer;
mod object_layer;
mod properties;

use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use egui::{CentralPanel, DragValue, SidePanel, Ui, WidgetText, *};
use egui_glow::CallbackFn;
use inline_tweak::tweak;
use smwe_emu::{emu::CheckedMem, rom::Rom, Cpu};
use smwe_render::color::Abgr1555;
use smwe_widgets::value_switcher::{ValueSwitcher, ValueSwitcherButtons};

use self::{level_renderer::LevelRenderer, object_layer::EditableObjectLayer, properties::LevelProperties};
use crate::ui::tool::DockableEditorTool;

pub struct UiLevelEditor {
    gl:             Rc<glow::Context>,
    cpu:            Cpu,
    level_renderer: Arc<Mutex<LevelRenderer>>,

    level_num:      u16,
    blue_pswitch:   bool,
    silver_pswitch: bool,
    on_off_switch:  bool,
    run_sprites:    bool,
    palette_line:   u8,
    sprite_id:      u8,
    timestamp:      std::time::Instant,

    offset:           Vec2,
    zoom:             f32,
    tile_size_px:     f32,
    pixels_per_point: f32,
    always_show_grid: bool,

    level_properties: LevelProperties,
    layer1:           EditableObjectLayer,
}

impl UiLevelEditor {
    pub fn new(gl: Rc<glow::Context>, rom: Arc<Rom>) -> Self {
        let level_renderer = Arc::new(Mutex::new(LevelRenderer::new(&gl)));
        let mut editor = Self {
            gl,
            cpu: Cpu::new(CheckedMem::new(rom)),
            level_renderer,
            level_num: 0x105,
            blue_pswitch: false,
            silver_pswitch: false,
            on_off_switch: false,
            run_sprites: false,
            palette_line: 0,
            sprite_id: 0,
            timestamp: std::time::Instant::now(),
            offset: Vec2::ZERO,
            zoom: 2.,
            tile_size_px: 16.,
            pixels_per_point: 1.,
            always_show_grid: false,
            level_properties: LevelProperties::default(),
            layer1: EditableObjectLayer::default(),
        };
        editor.init_cpu();
        editor.update_cpu_sprite();
        editor.update_renderer();
        editor
    }
}

// UI
impl DockableEditorTool for UiLevelEditor {
    fn update(&mut self, ui: &mut Ui) {
        self.pixels_per_point = ui.ctx().pixels_per_point();

        CentralPanel::default().frame(Frame::none()).show_inside(ui, |ui| self.central_panel(ui));
        SidePanel::right("level_editor.debug_panel").resizable(false).show_inside(ui, |ui| self.debug_panel(ui));

        // Auto-play animations
        let ft = std::time::Duration::from_secs_f32(1. / 60.);
        let now = std::time::Instant::now();
        while now - self.timestamp > ft {
            self.timestamp += ft;
            self.update_timers();
            self.update_anim_frame();
            if self.run_sprites {
                self.update_cpu_sprite();
                //self.level_renderer.lock().unwrap().upload_level(&self.gl, &mut self.cpu);
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
    fn debug_panel(&mut self, ui: &mut Ui) {
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
            self.update_cpu_sprite_id();
            // self.draw_sprites(state, ui.ctx());
        }

        ui.add(Slider::new(&mut self.zoom, 1.0..=3.0).step_by(0.25));

        ui.checkbox(&mut self.always_show_grid, "Always show grid");

        if need_update_level {
            self.update_cpu();
            self.update_cpu_sprite_id();
        }
        if need_update || need_update_level {
            self.update_renderer();
        }
    }

    fn central_panel(&mut self, ui: &mut Ui) {
        let bg_color = self.cpu.mem.load_u16(0x7E0701);
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

            if self.always_show_grid || ui.input(|i| i.modifiers.shift_only()) {
                let spacing = self.zoom * self.tile_size_px / self.pixels_per_point;
                let stroke = Stroke::new(1., Color32::from_white_alpha(tweak!(70)));
                for col in 0..(screen_size.x / spacing) as u32 {
                    let x_offset = (self.offset.x * self.zoom / self.pixels_per_point).rem_euclid(spacing);
                    let x_coord = col as f32 * spacing + x_offset;
                    ui.painter().vline(rect.min.x + x_coord, rect.min.y..=rect.max.y, stroke);
                }
                for row in 0..(screen_size.y / spacing) as u32 {
                    let y_offset = (self.offset.y * self.zoom / self.pixels_per_point).rem_euclid(spacing);
                    let y_coord = row as f32 * spacing + y_offset;
                    ui.painter().hline(rect.min.x..=rect.max.x, rect.min.y + y_coord, stroke);
                }
            }
        });
    }
}

// Internals
impl UiLevelEditor {
    fn init_cpu(&mut self) {
        smwe_emu::emu::decompress_sublevel(&mut self.cpu, self.level_num);
        println!("Updated CPU");
        self.level_renderer.lock().unwrap().upload_level(&self.gl, &mut self.cpu);
        self.level_properties = LevelProperties::parse_from_ram(&mut self.cpu);
        self.layer1 = EditableObjectLayer::parse_from_ram(&mut self.cpu, self.level_properties.is_vertical)
            .expect("Failed to parse objects from ExtRAM");
    }

    fn update_cpu(&mut self) {
        smwe_emu::emu::decompress_extram(&mut self.cpu, self.level_num);
        println!("Updated CPU");
        self.level_renderer.lock().unwrap().upload_level(&self.gl, &mut self.cpu);
    }

    fn update_timers(&mut self) {
        let m = self.cpu.mem.load_u8(0x13);
        self.cpu.mem.store_u8(0x13, m.wrapping_add(1));
        let m = self.cpu.mem.load_u8(0x14);
        self.cpu.mem.store_u8(0x14, m.wrapping_add(1));
    }

    fn update_cpu_sprite(&mut self) {
        self.cpu.mem.wram[0x300..0x400].fill(0xE0);
        smwe_emu::emu::exec_sprites(&mut self.cpu);
        self.level_renderer.lock().unwrap().upload_sprites(&self.gl, &mut self.cpu);
    }

    fn update_cpu_sprite_id(&mut self) {
        let mut cpu = self.cpu.clone();
        cpu.mem.wram[0x300..0x400].fill(0xE0);
        smwe_emu::emu::exec_sprite_id(&mut cpu, self.sprite_id);
        self.level_renderer.lock().unwrap().upload_sprites(&self.gl, &mut cpu);
    }

    fn update_anim_frame(&mut self) {
        self.cpu.mem.store_u8(0x14AD, self.blue_pswitch as u8);
        self.cpu.mem.store_u8(0x14AE, self.silver_pswitch as u8);
        self.cpu.mem.store_u8(0x14AF, self.on_off_switch as u8);
        smwe_emu::emu::fetch_anim_frame(&mut self.cpu);
        self.level_renderer
            .lock()
            .expect("Cannot lock mutex on level_renderer")
            .upload_gfx(&self.gl, &self.cpu.mem.vram);
    }

    fn update_renderer(&mut self) {
        self.update_anim_frame();

        let level_renderer = self.level_renderer.lock().expect("Cannot lock mutex on level_renderer");
        level_renderer.upload_palette(&self.gl, &self.cpu.mem.cgram);
        level_renderer.upload_gfx(&self.gl, &self.cpu.mem.vram);
    }
}
