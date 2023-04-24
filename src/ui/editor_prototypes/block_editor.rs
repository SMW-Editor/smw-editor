use egui::*;
use egui_extras::{Column, TableBuilder};
use inline_tweak::tweak;

use crate::ui::{tool::DockableEditorTool, EditorState};

pub struct UiBlockEditor {
    editing_modes:    Vec<String>,
    editing_mode_idx: usize,

    tile_palettes:         [u32; 4],
    tile_horizontal_flips: [bool; 4],
    tile_vertical_flips:   [bool; 4],
    tile_priorities:       [bool; 4],

    collision_types:    Vec<String>,
    collision_type_idx: usize,

    highlight_same_type: bool,
}

impl Default for UiBlockEditor {
    fn default() -> Self {
        UiBlockEditor {
            editing_modes:         vec![String::from("Blocks"), String::from("Tiles")],
            editing_mode_idx:      0,
            tile_palettes:         [5, 5, 5, 5],
            tile_horizontal_flips: [false, true, false, true],
            tile_vertical_flips:   [false, false, false, false],
            tile_priorities:       [false, false, false, false],
            collision_types:       vec![
                String::from("Hurt Mario"),
                String::from("Collect coin"),
                String::from("Eject coin"),
                String::from("Eject mushroom"),
                String::from("Eject fire flower"),
                String::from("Eject feather"),
                String::from("Eject 1-up"),
            ],
            collision_type_idx:    0,
            highlight_same_type:   false,
        }
    }
}

impl DockableEditorTool for UiBlockEditor {
    fn update(&mut self, ui: &mut Ui, project_ref: &mut EditorState) {
        ui.horizontal(|ui| {
            for (i, mode) in self.editing_modes.iter().enumerate() {
                if ui.add_enabled(self.editing_mode_idx != i, Button::new(mode)).clicked() {
                    self.editing_mode_idx = i;
                }
            }
        });
        SidePanel::left(ui.id().with("block-editor-left"))
            .resizable(false)
            .show_inside(ui, |ui| self.mappings(ui, project_ref));
        SidePanel::right(ui.id().with("block-editor-right"))
            .resizable(false)
            .show_inside(ui, |ui| self.vram(ui, project_ref));
        CentralPanel::default().show_inside(ui, |ui| {
            self.appearance(ui, project_ref);
            self.behaviour(ui, project_ref);
        });
    }

    fn title(&self) -> WidgetText {
        "Block editor".into()
    }
}

impl UiBlockEditor {
    fn mappings(&mut self, ui: &mut Ui, _project_ref: &mut EditorState) {
        ui.heading("Mappings");
    }

    fn vram(&mut self, ui: &mut Ui, _project_ref: &mut EditorState) {
        ui.heading("VRAM");
    }

    fn appearance(&mut self, ui: &mut Ui, _project_ref: &mut EditorState) {
        ui.heading("Appearance");

        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(Layout::left_to_right(Align::Min))
            .columns(Column::auto(), 6)
            .header(tweak!(15.0), |mut tr| {
                tr.col(|ui| {
                    ui.label("  ");
                });
                tr.col(|ui| {
                    ui.label("Source");
                });
                tr.col(|ui| {
                    ui.label("Palette");
                });
                tr.col(|ui| {
                    ui.label("H");
                });
                tr.col(|ui| {
                    ui.label("V");
                });
                tr.col(|ui| {
                    ui.label("P");
                });
            })
            .body(|tb| {
                tb.rows(tweak!(15.0), 4, |i, mut tr| {
                    tr.col(|ui| {
                        ui.label(i.to_string());
                    });
                    tr.col(|ui| {
                        ui.label(format!("muncher: {i}"));
                    });
                    tr.col(|ui| {
                        ui.add(DragValue::new(&mut self.tile_palettes[i]).clamp_range(0..=7));
                    });
                    tr.col(|ui| {
                        ui.checkbox(&mut self.tile_horizontal_flips[i], "");
                    });
                    tr.col(|ui| {
                        ui.checkbox(&mut self.tile_vertical_flips[i], "");
                    });
                    tr.col(|ui| {
                        ui.checkbox(&mut self.tile_priorities[i], "");
                    });
                })
            });
    }

    fn behaviour(&mut self, ui: &mut Ui, _project_ref: &mut EditorState) {
        ui.heading("Behaviour");

        ComboBox::from_label("Collision type")
            .selected_text(&self.collision_types[self.collision_type_idx])
            .width(tweak!(150.0))
            .show_ui(ui, |ui| {
                for (i, mode) in self.collision_types.iter().enumerate() {
                    if ui.button(mode).clicked() {
                        self.collision_type_idx = i;
                        ui.close_menu();
                    }
                }
            });

        ui.checkbox(&mut self.highlight_same_type, "Highlight same type");
    }
}
