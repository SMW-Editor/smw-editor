use eframe::egui::{TextEdit, Ui, Window};
use egui_extras::{Column, TableBuilder};
use smwe_rom::disassembler::serialization::LineKind;

use crate::{frame_context::FrameContext, ui::tool::UiTool};

pub struct UiCodeEditor {}

impl Default for UiCodeEditor {
    fn default() -> Self {
        log::info!("Opened Code Editor");
        Self {}
    }
}

impl UiTool for UiCodeEditor {
    fn update(&mut self, ui: &mut Ui, ctx: &mut FrameContext) -> bool {
        let mut running = true;

        Window::new("Code Editor").open(&mut running).vscroll(true).show(ui.ctx(), |ui| {
            TableBuilder::new(ui).columns(Column::auto_with_initial_suggestion(300.), 2).body(|tb| {
                let mut project = ctx.project_ref.as_mut().unwrap().borrow_mut();
                tb.rows(15., project.rom_data.disassembly.code_lines.len(), |i, mut tr| {
                    match &mut project.rom_data.disassembly.code_lines[i] {
                        LineKind::Label { label, comment } => {
                            tr.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        TextEdit::singleline(label).code_editor().clip_text(false).desired_width(0.),
                                    );
                                    ui.monospace(":");
                                });
                            });
                            tr.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.monospace(if comment.is_empty() { " " } else { ";" });
                                    ui.add(
                                        TextEdit::singleline(comment).code_editor().clip_text(false).desired_width(0.),
                                    );
                                });
                            });
                        }
                        LineKind::Op { op, arg: _, comment } => {
                            tr.col(|ui| {
                                ui.add(TextEdit::singleline(op).code_editor().clip_text(false).desired_width(0.));
                            });
                            tr.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.monospace(if comment.is_empty() { " " } else { ";" });
                                    ui.add(
                                        TextEdit::singleline(comment).code_editor().clip_text(false).desired_width(0.),
                                    );
                                });
                            });
                        }
                        LineKind::Empty {} => {
                            tr.col(|_| {});
                            tr.col(|_| {});
                        }
                        _ => {}
                    }
                })
            });
        });

        if !running {
            log::info!("Closed Code Editor");
        }
        running
    }
}
