use duplicate::duplicate;
use egui::{vec2, Event, Key, KeyboardShortcut, Modifiers, Pos2, Ui};
use smwe_math::coordinates::{OnCanvas, OnScreen};
use smwe_render::tile_renderer::TileJson;

use super::UiSpriteMapEditor;
use crate::ui::editing_mode::{EditingMode, SnapToGrid};

pub(super) const SHORTCUT_UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
pub(super) const SHORTCUT_REDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Y);

pub(super) const SHORTCUT_SELECT_ALL: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::A);
pub(super) const SHORTCUT_UNSELECT_ALL: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Escape);

pub(super) const SHORTCUT_DELETE_SELECTED: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Delete);

pub(super) const SHORTCUT_ZOOM_IN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::PlusEquals);
pub(super) const SHORTCUT_ZOOM_OUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Minus);

pub(super) const SHORTCUT_MODE_INSERT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Num1);
pub(super) const SHORTCUT_MODE_SELECT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Num2);
pub(super) const SHORTCUT_MODE_ERASE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Num3);
pub(super) const SHORTCUT_MODE_PROBE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Num4);
pub(super) const SHORTCUT_MODE_FLIP_HORIZONTALLY: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Num5);
pub(super) const SHORTCUT_MODE_FLIP_VERTICALLY: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Num6);

impl UiSpriteMapEditor {
    pub(super) fn handle_input(&mut self, ui: &Ui) {
        self.kb_shortcut_undo(ui);
        self.kb_shortcut_redo(ui);
        self.kb_shortcut_select_all(ui);
        self.kb_shortcut_unselect_all(ui);
        self.kb_shortcut_delete_selected(ui);
        self.kb_shortcut_move_selection(ui);
        self.kb_shortcut_copy(ui);
        self.kb_shortcut_cut(ui);
        self.kb_shortcuts_tools(ui);
        self.handle_zoom(ui);
    }

    fn kb_shortcut_undo(&mut self, ui: &Ui) {
        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_UNDO)) {
            self.handle_undo();
        }
    }

    fn kb_shortcut_redo(&mut self, ui: &Ui) {
        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_REDO)) {
            self.handle_redo();
        }
    }

    fn kb_shortcut_select_all(&mut self, ui: &Ui) {
        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_SELECT_ALL)) {
            self.mark_tiles_as_selected(0..self.sprite_tiles.read(|tiles| tiles.len()));
        }
    }

    fn kb_shortcut_unselect_all(&mut self, ui: &Ui) {
        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_UNSELECT_ALL)) {
            self.unselect_all_tiles();
        }
    }

    fn kb_shortcut_delete_selected(&mut self, ui: &Ui) {
        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_DELETE_SELECTED)) {
            self.delete_selected_tiles();
        }
    }

    fn kb_shortcut_move_selection(&mut self, ui: &Ui) {
        let move_distance = if ui.input(|input| input.modifiers.shift_only()) { self.tile_size_px } else { 1. };
        duplicate! {
            [
                key          offset;
                [ArrowUp]    [vec2(0., -move_distance)];
                [ArrowDown]  [vec2(0.,  move_distance)];
                [ArrowLeft]  [vec2(-move_distance, 0.)];
                [ArrowRight] [vec2( move_distance, 0.)];
            ]
            if ui.input(|input| input.key_pressed(Key::key)) {
                self.move_selected_tiles_by(
                    OnCanvas(offset),
                    ui.input(|input| input.modifiers.shift_only().then_some(SnapToGrid::default())),
                );
            }
        }
    }

    fn handle_zoom(&mut self, ui: &Ui) {
        if ui.input_mut(|input| input.zoom_delta() > 1. || input.consume_shortcut(&SHORTCUT_ZOOM_IN)) {
            self.zoom = 4.0f32.min(self.zoom + 0.25);
        } else if ui.input_mut(|input| input.zoom_delta() < 1. || input.consume_shortcut(&SHORTCUT_ZOOM_OUT)) {
            self.zoom = 1.0f32.max(self.zoom - 0.25);
        }
    }

    fn kb_shortcut_copy(&mut self, ui: &Ui) {
        if ui.input(|input| input.events.contains(&Event::Copy)) {
            ui.output_mut(|output| self.copy_selected_tiles(output));
        }
    }

    fn kb_shortcut_cut(&mut self, ui: &Ui) {
        if ui.input(|input| input.events.contains(&Event::Cut)) {
            ui.output_mut(|output| self.copy_selected_tiles(output));
            self.delete_selected_tiles();
        }
    }

    pub(super) fn kb_shortcut_paste(&mut self, ui: &Ui, canvas_top_left: OnScreen<Pos2>) {
        ui.input(|input| {
            for event in input.events.iter() {
                if let Event::Paste(pasted_text) = event {
                    if let Ok(pasted_tiles) = serde_json::from_str::<Vec<TileJson>>(pasted_text) {
                        let hover_pos = OnScreen(input.pointer.hover_pos().expect("Failed to get hover position"));
                        let paste_offset = hover_pos
                            .relative_to(canvas_top_left)
                            .to_vec2()
                            .to_canvas(self.pixels_per_point, self.zoom);
                        self.paste_tiles_at(pasted_tiles, paste_offset);
                    }
                    break;
                }
            }
        });
    }

    fn kb_shortcuts_tools(&mut self, ui: &Ui) {
        let modes = [
            (&SHORTCUT_MODE_INSERT, EditingMode::Move(None)),
            (&SHORTCUT_MODE_SELECT, EditingMode::Select),
            (&SHORTCUT_MODE_ERASE, EditingMode::Erase),
            (&SHORTCUT_MODE_PROBE, EditingMode::Probe),
            (&SHORTCUT_MODE_FLIP_HORIZONTALLY, EditingMode::FlipHorizontally),
            (&SHORTCUT_MODE_FLIP_VERTICALLY, EditingMode::FlipVertically),
        ];
        for (shortcut, mode) in modes {
            if ui.input_mut(|input| input.consume_shortcut(shortcut)) {
                self.editing_mode = mode;
                break;
            }
        }
    }
}
