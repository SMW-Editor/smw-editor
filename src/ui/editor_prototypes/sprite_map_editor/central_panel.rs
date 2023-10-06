use std::sync::Arc;

use duplicate::duplicate;
use egui::*;
use egui_glow::CallbackFn;
use egui_phosphor::regular as icons;
use inline_tweak::tweak;
use smwe_math::coordinates::*;
use smwe_render::tile_renderer::TileUniforms;
use smwe_widgets::value_switcher::{ValueSwitcher, ValueSwitcherButtons};

use super::{keyboard_shortcuts::*, UiSpriteMapEditor};
use crate::ui::editing_mode::*;

impl UiSpriteMapEditor {
    pub(super) fn central_panel(&mut self, ui: &mut Ui) {
        Frame::menu(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                self.editor_toolbar_menu(ui);
            });
        });
        ui.add_space(ui.spacing().item_spacing.y);
        self.editing_area(ui);
    }

    pub(super) fn editor_toolbar_menu(&mut self, ui: &mut Ui) {
        let level_switcher = ValueSwitcher::new(&mut self.level_num, "Level", ValueSwitcherButtons::LeftRight)
            .range(0..=0x1FF)
            .hexadecimal(3, false, true);
        if ui.add(level_switcher).changed() {
            self.update_cpu();
            self.update_renderers();
        }
        ui.separator();

        self.editing_mode_selector(ui);
        ui.separator();

        ui.horizontal(|ui| {
            let zoom_slider = Slider::new(&mut self.zoom, Self::MIN_ZOOM..=Self::MAX_ZOOM).step_by(0.25).suffix("x");
            ui.label(icons::MAGNIFYING_GLASS_PLUS);
            ui.add(zoom_slider);
        });

        ui.add_space(ui.available_width());
    }

    pub(super) fn editing_mode_selector(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            duplicate! {
                [
                    icon mode_name shortcut mode_desc mode_pattern mode_value;

                    [icons::CURSOR]
                    ["Insertion tool"]
                    [SHORTCUT_MODE_INSERT]
                    ["Right-click to insert tile, single click to select, drag to move."]
                    [EditingMode::Move(_)]
                    [EditingMode::Move(None)];

                    [icons::RECTANGLE]
                    ["Rectangular selection"]
                    [SHORTCUT_MODE_SELECT]
                    ["Left-click and drag to select tiles."]
                    [EditingMode::Select]
                    [EditingMode::Select];

                    [icons::ERASER]
                    ["Eraser"]
                    [SHORTCUT_MODE_ERASE]
                    ["Delete tiles while left mouse button is pressed."]
                    [EditingMode::Erase]
                    [EditingMode::Erase];

                    [icons::EYEDROPPER]
                    ["Probe"]
                    [SHORTCUT_MODE_PROBE]
                    ["Pick a tile from the canvas on left-click."]
                    [EditingMode::Probe]
                    [EditingMode::Probe];

                    [icons::ARROWS_OUT_LINE_HORIZONTAL]
                    ["Horizontal flip"]
                    [SHORTCUT_MODE_FLIP_HORIZONTALLY]
                    ["Horizontally mirror selection or individual tile on left-click. Hold Ctrl to temporarily switch to vertical flip."]
                    [EditingMode::FlipHorizontally]
                    [EditingMode::FlipHorizontally];

                    [icons::ARROWS_OUT_LINE_VERTICAL]
                    ["Vertical flip"]
                    [SHORTCUT_MODE_FLIP_VERTICALLY]
                    ["Vertical mirror selection or individual tile on left-click. Hold Ctrl to temporarily switch to horizontal flip."]
                    [EditingMode::FlipVertically]
                    [EditingMode::FlipVertically];
                ]
                {
                    let button = if matches!(self.editing_mode, mode_pattern) {
                        Button::new(icon).fill(Color32::from_rgb(tweak!(200), tweak!(30), tweak!(70)))
                    } else {
                        Button::new(icon)
                    };

                    let tooltip = |ui: &mut Ui| {
                        ui.horizontal(|ui| {
                            ui.strong(mode_name);
                            ui.weak(ui.ctx().format_shortcut(&shortcut));
                        });
                        ui.label(mode_desc);
                    };

                    if ui.add(button).on_hover_ui_at_pointer(tooltip).clicked() {
                        self.editing_mode = mode_value;
                    }
                }
            }
        });
    }

    pub(super) fn editing_area(&mut self, ui: &mut Ui) {
        ScrollArea::both()
            .drag_to_scroll(false)
            .min_scrolled_height(ui.available_height())
            .min_scrolled_width(ui.available_width())
            .show(ui, |ui| {
                let editing_area_rect = ui.available_rect_before_wrap();
                let canvas_rect = Rect::from_center_size(editing_area_rect.center(), self.canvas_size().0);
                let max_rect = editing_area_rect.union(canvas_rect);
                let margin = Margin {
                    left:   canvas_rect.min.x - max_rect.min.x,
                    right:  max_rect.max.x - canvas_rect.max.x,
                    top:    canvas_rect.min.y - max_rect.min.y,
                    bottom: max_rect.max.y - canvas_rect.max.y,
                };
                Frame::canvas(ui.style()).inner_margin(Margin::same(0.)).outer_margin(margin).show(ui, |ui| {
                    self.canvas(ui);
                });
            });
    }

    pub(super) fn canvas(&mut self, ui: &mut Ui) {
        let (canvas_rect, response) = ui.allocate_exact_size(self.canvas_size().0, Sense::click_and_drag());

        // Tiles
        ui.painter().add(PaintCallback {
            rect:     canvas_rect,
            callback: {
                let sprite_renderer = Arc::clone(&self.sprite_renderer);
                let gfx_bufs = self.gfx_bufs;
                let screen_size = canvas_rect.size() * self.pixels_per_point;
                let zoom = self.zoom;
                Arc::new(CallbackFn::new(move |_info, painter| {
                    sprite_renderer
                        .lock()
                        .expect("Cannot lock mutex on sprite renderer")
                        .paint(painter.gl(), &TileUniforms { gfx_bufs, screen_size, offset: Vec2::ZERO, zoom });
                }))
            },
        });

        // Grid
        if self.always_show_grid || ui.input(|i| i.modifiers.shift_only()) {
            let spacing = self.zoom * self.tile_size_px / self.pixels_per_point;
            let stroke = Stroke::new(1., Color32::from_white_alpha(tweak!(70)));
            for cell in 0..33 {
                let position = cell as f32 * spacing;
                ui.painter().hline(canvas_rect.min.x..=canvas_rect.max.x, canvas_rect.min.y + position, stroke);
                ui.painter().vline(canvas_rect.min.x + position, canvas_rect.min.y..=canvas_rect.max.y, stroke);
            }
        }

        // DEBUG: show selection bounds
        #[cfg(debug_assertions)]
        if self.debug_selection_bounds {
            if let Some(mut bounds) = self.selection_bounds {
                bounds.0.max += Vec2::splat(self.tile_size_px);
                ui.painter().rect_stroke(
                    bounds.to_screen(self.pixels_per_point, self.zoom).0.translate(canvas_rect.left_top().to_vec2()),
                    Rounding::ZERO,
                    Stroke::new(2., Color32::BLUE),
                );
            }
        }

        // Interaction
        if let Some(hover_pos) = response.hover_pos() {
            let canvas_top_left_pos = OnScreen(canvas_rect.left_top());

            let relative_pointer_offset = OnScreen(hover_pos - canvas_rect.left_top());
            let relative_pointer_pos = relative_pointer_offset.to_pos2();

            let grid_cell_pos = relative_pointer_offset
                .to_grid(self.pixels_per_point, self.zoom, self.tile_size_px)
                .clamp(OnGrid::<Vec2>::ZERO, self.grid_size)
                .to_canvas(self.tile_size_px)
                .to_pos2();

            let (holding_shift_only, holding_ctrl_only) =
                ui.input(|input| (input.modifiers.shift_only(), input.modifiers.command_only()));

            let mut should_highlight_hovered = true;

            // Keyboard shortcuts
            ui.input(|input| self.kb_shortcut_paste(input, canvas_top_left_pos));

            // Editing tools
            if self.editing_mode.inserted(&response) {
                self.handle_edition_insert(grid_cell_pos);
            }

            if let Some(selection) = self.editing_mode.selected(&response) {
                if let Selection::Drag(Some(selection_rect)) = selection {
                    ui.painter().rect_stroke(
                        selection_rect.0,
                        Rounding::ZERO,
                        Stroke::new(1., ui.visuals().selection.bg_fill),
                    );
                }
                self.handle_selection_plot(selection, !holding_ctrl_only, canvas_top_left_pos);
            }

            if let Some(drag_data) = self.editing_mode.dropped(&response) {
                self.handle_edition_drop(drag_data, holding_shift_only, canvas_top_left_pos);
            }

            if let Some(drag_data) = self.editing_mode.moving(&response) {
                self.handle_edition_dragging(drag_data, holding_shift_only, canvas_top_left_pos);
                should_highlight_hovered = false;
            }

            if self.editing_mode.erased(&response) {
                self.handle_edition_erase(relative_pointer_pos);
            }

            if self.editing_mode.probed(&response) {
                self.handle_edition_probe(relative_pointer_pos);
            }

            if let Some(flip_direction) = self.editing_mode.flipped(&response) {
                self.handle_edition_flip(relative_pointer_pos, flip_direction);
            }

            // Highlighting
            if should_highlight_hovered {
                self.higlight_hovered_tiles(ui, relative_pointer_pos, OnScreen(canvas_rect.left_top()));
            }
        }

        self.highlight_selected_tiles(ui, OnScreen(canvas_rect.left_top()));
        self.hovering_selected_tile = false;
    }
}
