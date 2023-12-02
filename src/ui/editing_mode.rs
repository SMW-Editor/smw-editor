#![allow(dead_code)]

use egui::{PointerButton, Pos2, Rect, Response, Vec2};
use smwe_math::coordinates::OnScreen;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EditingMode {
    Draw,
    Erase,
    Move(Option<Drag>),
    Probe,
    Select,
    FlipHorizontally,
    FlipVertically,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Selection {
    Click(Option<OnScreen<Pos2>>),
    Drag(Option<OnScreen<Rect>>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Drag {
    pub from: OnScreen<Pos2>,
    pub to:   OnScreen<Pos2>,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct SnapToGrid {
    pub cell_origin: OnScreen<Vec2>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FlipDirection {
    Horizontal,
    Vertical,
}

impl Drag {
    #[inline]
    pub fn delta(self) -> OnScreen<Vec2> {
        OnScreen(self.to.0 - self.from.0)
    }
}

impl EditingMode {
    pub fn inserted(self, response: &Response) -> bool {
        match self {
            Self::Move(_) => response.clicked_by(PointerButton::Secondary),
            Self::Draw => response.clicked_by(PointerButton::Primary) || response.dragged_by(PointerButton::Primary),
            _ => false,
        }
    }

    pub fn moving(&mut self, response: &Response) -> Option<Drag> {
        match self {
            Self::Move(drag) => response
                .dragged_by(PointerButton::Primary)
                .then(|| {
                    *drag = response.ctx.input(|i| {
                        i.pointer
                            .press_origin()
                            .map(OnScreen)
                            .zip(response.interact_pointer_pos().map(OnScreen))
                            .map(|(from, to)| Drag { from, to })
                    });
                    *drag
                })
                .flatten(),
            _ => None,
        }
    }

    /// If the mode is `Move`, its inner value is cleared.
    /// Must be called after [`Self::moving`] to get correct data in the return value.
    pub fn dropped(&mut self, response: &Response) -> Option<Drag> {
        match self {
            Self::Move(drag) => response.drag_released_by(PointerButton::Primary).then_some(drag.take()).flatten(),
            _ => None,
        }
    }

    pub fn selected(self, response: &Response) -> Option<Selection> {
        match self {
            Self::Move(_) => response
                .clicked_by(PointerButton::Primary)
                .then(|| Selection::Click(response.interact_pointer_pos().map(OnScreen))),
            Self::Select => response.dragged_by(PointerButton::Primary).then(|| {
                let rect = response.ctx.input(|i| {
                    i.pointer.press_origin().and_then(|origin| {
                        response.interact_pointer_pos().map(|current| Rect::from_two_pos(origin, current)).map(OnScreen)
                    })
                });
                Selection::Drag(rect)
            }),
            _ => None,
        }
    }

    pub fn erased(self, response: &Response) -> bool {
        match self {
            Self::Erase => response.clicked_by(PointerButton::Primary) || response.dragged_by(PointerButton::Primary),
            _ => false,
        }
    }

    pub fn probed(self, response: &Response) -> bool {
        match self {
            Self::Probe => response.clicked_by(PointerButton::Primary),
            _ => false,
        }
    }

    pub fn flipped(self, response: &Response) -> Option<FlipDirection> {
        if response.clicked_by(PointerButton::Primary) {
            let invert = response.ctx.input(|input| input.modifiers.command_only());
            match (self, invert) {
                (Self::FlipHorizontally, false) | (Self::FlipVertically, true) => Some(FlipDirection::Horizontal),
                (Self::FlipVertically, false) | (Self::FlipHorizontally, true) => Some(FlipDirection::Vertical),
                _ => None,
            }
        } else {
            None
        }
    }
}
