use std::sync::Arc;

use duplicate::duplicate;
use egui::*;
use egui_glow::CallbackFn;
use egui_phosphor as icons;
use inline_tweak::tweak;
use itertools::Itertools;
use smwe_widgets::{
    value_switcher::{ValueSwitcher, ValueSwitcherButtons},
    vram_view::{ViewedVramTiles, VramView},
};

use crate::ui::{
    editing_mode::{EditingMode, Selection},
    editor_prototypes::sprite_map_editor::UiSpriteMapEditor,
    tool::DockableEditorTool,
    EditorState,
};

impl DockableEditorTool for UiSpriteMapEditor {
    fn update(&mut self, ui: &mut Ui, state: &mut EditorState) {
        if !self.initialized {
            self.update_cpu(state);
            self.update_renderers(state);
            self.pixels_per_point = ui.ctx().pixels_per_point();
            self.initialized = true;
        }

        self.handle_keyboard(ui, state);

        TopBottomPanel::top("sprite_map_editor.top_panel")
            .resizable(false)
            .show_inside(ui, |ui| self.top_panel(ui, state));
        SidePanel::left("sprite_map_editor.left_panel")
            .resizable(false)
            .show_inside(ui, |ui| self.left_panel(ui, state));
        CentralPanel::default().show_inside(ui, |ui| self.central_panel(ui, state));
    }

    fn title(&self) -> WidgetText {
        "Sprite Tile Editor".into()
    }

    fn on_closed(&mut self) {
        self.destroy();
    }
}

impl UiSpriteMapEditor {
    pub(super) fn handle_keyboard(&mut self, ui: &mut Ui, _state: &mut EditorState) {
        ui.input(|input| {
            let move_distance = if input.modifiers.shift_only() { self.scale } else { 1. };

            if input.key_pressed(Key::Delete) {
                for idx in self.selected_sprite_tile_indices.drain().sorted().rev() {
                    self.sprite_tiles.remove(idx);
                }
                self.upload_tiles();
            } else if input.key_pressed(Key::ArrowUp) {
                self.move_selected_tiles_by(vec2(0., -move_distance), input.modifiers.shift_only());
            } else if input.key_pressed(Key::ArrowDown) {
                self.move_selected_tiles_by(vec2(0., move_distance), input.modifiers.shift_only());
            } else if input.key_pressed(Key::ArrowLeft) {
                self.move_selected_tiles_by(vec2(-move_distance, 0.), input.modifiers.shift_only());
            } else if input.key_pressed(Key::ArrowRight) {
                self.move_selected_tiles_by(vec2(move_distance, 0.), input.modifiers.shift_only());
            }
        });
    }

    pub(super) fn top_panel(&mut self, ui: &mut Ui, state: &mut EditorState) {
        ui.horizontal(|ui| {
            self.level_selector(ui, state);

            ui.add_space(tweak!(84.) * self.pixels_per_point);
            ui.separator();

            self.editing_mode_selector(ui, state);

            ui.separator();

            ui.checkbox(&mut self.always_show_grid, "Always show grid");
        });
    }

    pub(super) fn level_selector(&mut self, ui: &mut Ui, state: &mut EditorState) {
        let level_switcher = ValueSwitcher::new(&mut self.level_num, "Level", ValueSwitcherButtons::MinusPlus)
            .range(0..=0x1FF)
            .hexadecimal(3, false, true);

        if ui.add(level_switcher).changed() {
            self.update_cpu(state);
            self.update_renderers(state);
        }
    }

    pub(super) fn editing_mode_selector(&mut self, ui: &mut Ui, _state: &mut EditorState) {
        duplicate! {
            [
                icon mode_name mode_desc mode_pattern mode_value;

                [icons::ARROWS_OUT_CARDINAL]
                ["Move mode"]
                ["Double click to insert tile, single click to select, drag to move."]
                [EditingMode::Move(_)]
                [EditingMode::Move(None)];

                [icons::RECTANGLE]
                ["Select mode"]
                ["Left-click and drag to select tiles."]
                [EditingMode::Select]
                [EditingMode::Select];

                [icons::PENCIL]
                ["Draw mode"]
                ["Insert tiles while left mouse button is pressed."]
                [EditingMode::Draw]
                [EditingMode::Draw];

                [icons::ERASER]
                ["Erase mode"]
                ["Delete tiles while left mouse button is pressed."]
                [EditingMode::Erase]
                [EditingMode::Erase];

                [icons::EYEDROPPER]
                ["Probe mode"]
                ["Pick a tile from the canvas on left-click."]
                [EditingMode::Probe]
                [EditingMode::Probe];
            ]
            {
                let button = if matches!(self.editing_mode, mode_pattern) {
                    Button::new(icon).fill(Color32::from_rgb(tweak!(200), tweak!(30), tweak!(70)))
                } else {
                    Button::new(icon)
                };

                let tooltip = |ui: &mut Ui| {
                    ui.strong(mode_name);
                    ui.label(mode_desc);
                };

                if ui.add(button).on_hover_ui_at_pointer(tooltip).clicked() {
                    self.editing_mode = mode_value;
                }
            }
        }
    }

    pub(super) fn left_panel(&mut self, ui: &mut Ui, _state: &mut EditorState) {
        let vram_renderer = Arc::clone(&self.vram_renderer);
        let gfx_bufs = self.gfx_bufs;

        // Tile selector
        ui.label("VRAM view");
        ui.add(
            VramView::new(Arc::clone(&vram_renderer), gfx_bufs)
                .viewed_tiles(ViewedVramTiles::SpritesOnly)
                .selection(&mut self.selected_vram_tile)
                .zoom(2.),
        );

        // Selected tile preview
        ui.add_space(tweak!(10.));
        ui.label("Selection preview");
        let px = self.pixels_per_point;
        let zoom = tweak!(8.);
        let (rect, _response) = ui.allocate_exact_size(Vec2::splat(zoom * 8. / px), Sense::hover());

        let screen_size = rect.size() * px;
        let offset = vec2(-(self.selected_vram_tile.0 as f32), -32. - self.selected_vram_tile.1 as f32) * zoom;

        ui.painter().add(PaintCallback {
            rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                vram_renderer.lock().expect("Cannot lock mutex on selected tile view's tile renderer").paint(
                    painter.gl(),
                    gfx_bufs,
                    screen_size,
                    offset,
                    zoom,
                );
            })),
        });
    }

    pub(super) fn central_panel(&mut self, ui: &mut Ui, _state: &mut EditorState) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            let sprite_renderer = Arc::clone(&self.sprite_renderer);
            let gfx_bufs = self.gfx_bufs;
            let editing_area_size = tweak!(256.) * self.zoom;
            let (canvas_rect, response) =
                ui.allocate_exact_size(Vec2::splat(editing_area_size / self.pixels_per_point), Sense::click_and_drag());
            let screen_size = canvas_rect.size() * self.pixels_per_point;
            let scale_pp = self.scale / self.pixels_per_point;
            let zoom = self.zoom;

            ui.painter().add(PaintCallback {
                rect:     canvas_rect,
                callback: Arc::new(CallbackFn::new(move |_info, painter| {
                    sprite_renderer.lock().expect("Cannot lock mutex on sprite renderer").paint(
                        painter.gl(),
                        gfx_bufs,
                        screen_size,
                        Vec2::ZERO,
                        zoom,
                    );
                })),
            });

            // Grid
            if self.always_show_grid || ui.input(|i| i.modifiers.shift_only()) {
                self.draw_grid(ui, canvas_rect);
            }

            if let Some(hover_pos) = response.hover_pos() {
                let relative_pos = hover_pos - canvas_rect.left_top();
                let hovered_tile = (relative_pos / scale_pp / self.zoom).floor();
                let hovered_tile = hovered_tile.clamp(vec2(0., 0.), vec2(31., 31.));
                let hovered_tile_exact_offset = hovered_tile * scale_pp * self.zoom;
                let grid_cell_pos = (hovered_tile * self.scale).to_pos2();

                // Highlight hovered cell/tile
                match self.editing_mode {
                    EditingMode::Move(_) => {
                        // todo: highlight tile if present
                    }
                    EditingMode::Erase => {
                        // todo: highlight tile if present
                    }
                    EditingMode::Draw => {
                        self.highlight_tile_at(
                            ui,
                            canvas_rect.left_top() + hovered_tile_exact_offset,
                            Color32::from_white_alpha(tweak!(100)),
                        );
                    }
                    _ => {}
                }

                // Interaction
                if self.editing_mode.inserted(&response) {
                    if self.last_inserted_tile != grid_cell_pos {
                        self.add_selected_tile_at(grid_cell_pos);
                        self.last_inserted_tile = grid_cell_pos;
                    }
                    self.selected_sprite_tile_indices.clear();
                }

                if let Some(selection) = self.editing_mode.selected(&response) {
                    let should_clear = ui.input(|i| !i.modifiers.shift_only());
                    match selection {
                        Selection::Click(Some(origin)) => {
                            let pos = origin - canvas_rect.left_top();
                            self.select_tile_at(pos.to_pos2(), should_clear);
                        }
                        Selection::Drag(Some(selection_rect)) => {
                            ui.painter().rect_stroke(
                                selection_rect,
                                Rounding::none(),
                                Stroke::new(1., ui.visuals().selection.bg_fill),
                            );
                            self.select_tiles_in(
                                selection_rect.translate(-canvas_rect.left_top().to_vec2()),
                                should_clear,
                            );
                        }
                        _ => {}
                    }
                }

                if let Some(drag_data) = self.editing_mode.dropped(&response) {
                    let snap_to_grid = ui.input(|i| i.modifiers.shift_only());
                    self.move_selected_tiles_by(drag_data.delta() / self.zoom * self.pixels_per_point, snap_to_grid);
                }

                if let Some(drag_data) = self.editing_mode.moving(&response) {
                    let snap_to_grid = ui.input(|i| i.modifiers.shift_only());
                    // todo higlight moved tiles
                }

                if self.editing_mode.erased(&response) {
                    self.delete_tiles_at(relative_pos.to_pos2());
                    self.selected_sprite_tile_indices.clear();
                }

                if self.editing_mode.probed(&response) {
                    self.probe_tile_at(relative_pos.to_pos2());
                    self.selected_sprite_tile_indices.clear();
                }
            }

            self.highlight_selected_tiles(ui, canvas_rect.left_top());
        });
    }

    pub(super) fn highlight_tile_at(&self, ui: &mut Ui, pos: Pos2, color: impl Into<Color32>) {
        ui.painter().rect_filled(
            Rect::from_min_size(pos, Vec2::splat(self.scale * self.zoom / self.pixels_per_point)),
            Rounding::none(),
            color,
        );
    }

    pub(super) fn highlight_selected_tiles(&self, ui: &mut Ui, canvas_pos: Pos2) {
        for tile in self.selected_sprite_tile_indices.iter().map(|&idx| &self.sprite_tiles[idx]) {
            self.highlight_tile_at(
                ui,
                canvas_pos + tile.pos().to_vec2() / self.pixels_per_point * self.zoom,
                Color32::from_white_alpha(tweak!(40)),
            );
        }
    }

    pub(super) fn draw_grid(&self, ui: &mut Ui, canvas_rect: Rect) {
        let spacing = self.zoom * self.scale / self.pixels_per_point;
        for cell in 0..33 {
            let position = cell as f32 * spacing;
            ui.painter().hline(
                canvas_rect.min.x..=canvas_rect.max.x,
                canvas_rect.min.y + position,
                Stroke::new(1., Color32::from_white_alpha(70)),
            );
            ui.painter().vline(
                canvas_rect.min.x + position,
                canvas_rect.min.y..=canvas_rect.max.y,
                Stroke::new(1., Color32::from_white_alpha(70)),
            );
        }
    }
}
