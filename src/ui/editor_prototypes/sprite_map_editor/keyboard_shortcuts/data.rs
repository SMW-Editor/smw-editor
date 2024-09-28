use egui::{Key::*, KeyboardShortcut as Shortcut, Modifiers};

pub(in super::super) const SHORTCUT_NEW: Shortcut = Shortcut::new(Modifiers::COMMAND, N);
pub(in super::super) const SHORTCUT_SAVE: Shortcut = Shortcut::new(Modifiers::COMMAND, S);
pub(in super::super) const SHORTCUT_OPEN: Shortcut = Shortcut::new(Modifiers::COMMAND, O);

pub(in super::super) const SHORTCUT_UNDO: Shortcut = Shortcut::new(Modifiers::COMMAND, Z);
pub(in super::super) const SHORTCUT_REDO: Shortcut = Shortcut::new(Modifiers::COMMAND, Y);
pub(in super::super) const SHORTCUT_COPY: Shortcut = Shortcut::new(Modifiers::COMMAND, C);
pub(in super::super) const SHORTCUT_CUT: Shortcut = Shortcut::new(Modifiers::COMMAND, X);

pub(in super::super) const SHORTCUT_SELECT_ALL: Shortcut = Shortcut::new(Modifiers::COMMAND, A);
pub(in super::super) const SHORTCUT_UNSELECT_ALL: Shortcut = Shortcut::new(Modifiers::NONE, Escape);

pub(in super::super) const SHORTCUT_DELETE_SELECTED: Shortcut = Shortcut::new(Modifiers::NONE, Delete);

pub(in super::super) const SHORTCUT_ZOOM_IN: Shortcut = Shortcut::new(Modifiers::COMMAND, Plus);
pub(in super::super) const SHORTCUT_ZOOM_OUT: Shortcut = Shortcut::new(Modifiers::COMMAND, Minus);

pub(in super::super) const SHORTCUT_MODE_INSERT: Shortcut = Shortcut::new(Modifiers::NONE, Num1);
pub(in super::super) const SHORTCUT_MODE_SELECT: Shortcut = Shortcut::new(Modifiers::NONE, Num2);
pub(in super::super) const SHORTCUT_MODE_ERASE: Shortcut = Shortcut::new(Modifiers::NONE, Num3);
pub(in super::super) const SHORTCUT_MODE_PROBE: Shortcut = Shortcut::new(Modifiers::NONE, Num4);
pub(in super::super) const SHORTCUT_MODE_FLIP_HORIZONTALLY: Shortcut = Shortcut::new(Modifiers::NONE, Num5);
pub(in super::super) const SHORTCUT_MODE_FLIP_VERTICALLY: Shortcut = Shortcut::new(Modifiers::NONE, Num6);
