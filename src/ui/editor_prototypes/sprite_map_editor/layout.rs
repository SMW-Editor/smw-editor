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
    editing_mode::{EditingMode, Selection, SnapToGrid},
    editor_prototypes::sprite_map_editor::{math::tile_contains_point, UiSpriteMapEditor},
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

            // Select all
            if input.modifiers.command_only() && input.key_pressed(Key::A) {
                self.mark_tiles_as_selected(0..self.sprite_tiles.len());
            }

            // Unselect all
            if input.key_pressed(Key::Escape) {
                self.unselect_all_tiles();
            }

            // Delete
            if input.key_pressed(Key::Delete) {
                for idx in self.selected_sprite_tile_indices.drain().sorted().rev() {
                    self.sprite_tiles.remove(idx);
                }
                self.selection_bounds = None;
                self.upload_tiles();
            }

            // Move selection
            duplicate! {
                [
                    key          offset;
                    [ArrowUp]    [vec2(0., -move_distance)];
                    [ArrowDown]  [vec2(0.,  move_distance)];
                    [ArrowLeft]  [vec2(-move_distance, 0.)];
                    [ArrowRight] [vec2( move_distance, 0.)];
                ]
                if input.key_pressed(Key::key) {
                    self.move_selected_tiles_by(
                        offset,
                        input.modifiers.shift_only().then_some(SnapToGrid::default()),
                    );
                }
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

        // Debug settings
        ui.label("Debug");
        ui.checkbox(&mut self.debug_selection_bounds, "Show selection bounds");
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

            // Tiles
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

            // DEBUG: show selection bounds
            if self.debug_selection_bounds {
                if let Some(mut bounds) = self.selection_bounds {
                    let scaling = self.zoom / self.pixels_per_point;
                    bounds.min = canvas_rect.left_top() + (bounds.min.to_vec2() * scaling);
                    bounds.max = canvas_rect.left_top() + ((bounds.max.to_vec2() + Vec2::splat(self.scale)) * scaling);
                    ui.painter().rect_stroke(bounds, Rounding::none(), Stroke::new(2., Color32::BLUE));
                }
            }

            // Interaction
            if let Some(hover_pos) = response.hover_pos() {
                let canvas_top_left_pos = canvas_rect.left_top();

                let relative_pointer_offset = hover_pos - canvas_rect.left_top();
                let relative_pointer_pos = relative_pointer_offset.to_pos2();

                let hovered_tile_offset = (relative_pointer_offset / scale_pp / self.zoom).floor();
                let hovered_tile_offset = hovered_tile_offset.clamp(vec2(0., 0.), vec2(31., 31.));
                let grid_cell_pos = (hovered_tile_offset * self.scale).to_pos2();

                let holding_shift = ui.input(|i| i.modifiers.shift_only());
                let holding_ctrl = ui.input(|i| i.modifiers.command_only());

                self.higlight_hovered_tiles(ui, relative_pointer_pos, canvas_rect.left_top());

                if self.editing_mode.inserted(&response) {
                    self.handle_edition_insert(grid_cell_pos);
                }

                if let Some(selection) = self.editing_mode.selected(&response) {
                    if let Selection::Drag(Some(selection_rect)) = selection {
                        ui.painter().rect_stroke(
                            selection_rect,
                            Rounding::none(),
                            Stroke::new(1., ui.visuals().selection.bg_fill),
                        );
                    }
                    self.handle_selection_plot(selection, !holding_ctrl, canvas_top_left_pos);
                }

                if let Some(drag_data) = self.editing_mode.dropped(&response) {
                    self.handle_edition_drop_moved(drag_data, holding_shift, canvas_top_left_pos);
                }

                if let Some(drag_data) = self.editing_mode.moving(&response) {
                    self.handle_edition_dragging(drag_data, holding_shift, canvas_top_left_pos);
                }

                if self.editing_mode.erased(&response) {
                    self.handle_edition_erase(relative_pointer_pos);
                }

                if self.editing_mode.probed(&response) {
                    self.handle_edition_probe(relative_pointer_pos);
                }
            }

            self.highlight_selected_tiles(ui, canvas_rect.left_top());
            self.hovering_selected_tile = false;
        });
    }

    pub(super) fn higlight_hovered_tiles(&mut self, ui: &mut Ui, relative_pointer_pos: Pos2, canvas_left_top: Pos2) {
        let scaling_factor = self.zoom / self.pixels_per_point;
        match self.editing_mode {
            EditingMode::Move(_) => {
                if self
                    .selected_sprite_tile_indices
                    .iter()
                    .map(|&i| &self.sprite_tiles[i])
                    .any(|tile| tile_contains_point(tile, relative_pointer_pos, scaling_factor))
                {
                    self.hovering_selected_tile = true;
                } else if let Some(hovered_tile) = self
                    .sprite_tiles
                    .iter()
                    .find(|tile| tile_contains_point(tile, relative_pointer_pos, scaling_factor))
                {
                    let exact_tile_pos = canvas_left_top + (hovered_tile.pos().to_vec2() * scaling_factor);
                    self.highlight_tile_at(ui, exact_tile_pos, Color32::from_white_alpha(tweak!(100)));
                }
            }
            EditingMode::Erase => {
                if let Some(hovered_tile) = self
                    .sprite_tiles
                    .iter()
                    .find(|tile| tile_contains_point(tile, relative_pointer_pos, scaling_factor))
                {
                    self.highlight_tile_at(
                        ui,
                        ((hovered_tile.pos().to_vec2() * scaling_factor) + canvas_left_top.to_vec2()).to_pos2(),
                        Color32::from_rgba_premultiplied(tweak!(255), 0, 0, tweak!(10)),
                    );
                }
            }
            EditingMode::Draw => {
                let scale_pp = self.scale / self.pixels_per_point;
                let hovered_tile = (relative_pointer_pos.to_vec2() / scale_pp / self.zoom).floor();
                let hovered_tile = hovered_tile.clamp(vec2(0., 0.), vec2(31., 31.));
                let hovered_tile_exact_offset = hovered_tile * scale_pp * self.zoom;
                self.highlight_tile_at(
                    ui,
                    canvas_left_top + hovered_tile_exact_offset,
                    Color32::from_white_alpha(tweak!(100)),
                );
            }
            _ => {}
        }
    }

    pub(super) fn highlight_tile_at(&self, ui: &mut Ui, pos: Pos2, color: impl Into<Color32>) {
        ui.painter().rect_filled(
            Rect::from_min_size(pos, Vec2::splat(self.scale * self.zoom / self.pixels_per_point)),
            Rounding::none(),
            color,
        );
    }

    pub(super) fn highlight_selected_tiles(&mut self, ui: &mut Ui, canvas_pos: Pos2) {
        let selection_offset = self.selection_offset.take().unwrap_or_default();
        for tile in self.selected_sprite_tile_indices.iter().map(|&idx| &self.sprite_tiles[idx]) {
            self.highlight_tile_at(
                ui,
                canvas_pos + selection_offset + tile.pos().to_vec2() / self.pixels_per_point * self.zoom,
                Color32::from_white_alpha(if self.hovering_selected_tile { tweak!(100) } else { tweak!(40) }),
            );
        }
    }

    pub(super) fn draw_grid(&self, ui: &mut Ui, canvas_rect: Rect) {
        let spacing = self.zoom * self.scale / self.pixels_per_point;
        let stroke = Stroke::new(1., Color32::from_white_alpha(tweak!(70)));
        for cell in 0..33 {
            let position = cell as f32 * spacing;
            ui.painter().hline(canvas_rect.min.x..=canvas_rect.max.x, canvas_rect.min.y + position, stroke);
            ui.painter().vline(canvas_rect.min.x + position, canvas_rect.min.y..=canvas_rect.max.y, stroke);
        }
    }
}
