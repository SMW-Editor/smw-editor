use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use egui::{ColorImage, Context, Id, Response, TextureFilter, TextureHandle, Ui, Vec2, Widget};
use itertools::Itertools;
use thiserror::Error;

#[derive(Clone)]
pub struct AnimationState {
    frames:        Vec<TextureHandle>,
    loop_progress: f32,
    id:            Id,
    started:       bool,
}

#[derive(Copy, Clone, Debug, Error)]
#[error("All animation frames must have the same size")]
pub struct AnimationsFramesSizeError;

#[derive(Copy, Clone, Debug, Error)]
#[error("Each animation must have at least one frame")]
pub struct EmptyAnimationError;

pub struct Flipbook<'a> {
    animation: &'a mut AnimationState,
    size:      Vec2,
    duration:  f32,
    looped:    bool,
    reverse:   bool,
}

impl AnimationState {
    pub fn from_images(frames: Vec<ColorImage>, ctx: &Context) -> anyhow::Result<Self> {
        if frames.is_empty() {
            Err(EmptyAnimationError.into())
        } else {
            let mut hasher = DefaultHasher::new();
            let handles = frames
                .into_iter()
                .map(|frame| {
                    frame.pixels.hash(&mut hasher);
                    ctx.load_texture(format!("anim_frame_{}", hasher.finish()), frame, TextureFilter::Nearest)
                })
                .collect_vec();
            Self::from_texture_handles(handles, Id::new(hasher.finish()))
        }
    }

    pub fn from_texture_handles(handles: Vec<TextureHandle>, id: impl Into<Id>) -> anyhow::Result<Self> {
        if handles.is_empty() {
            Err(EmptyAnimationError.into())
        } else if !handles.iter().map(|handle| handle.size()).all_equal() {
            Err(AnimationsFramesSizeError.into())
        } else {
            Ok(Self { frames: handles, loop_progress: 0.0, id: id.into(), started: false })
        }
    }
}

impl<'a> Widget for Flipbook<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { animation, duration, size, looped, reverse } = self;
        let reset_anim = || ui.ctx().animate_value_with_time(animation.id, if reverse { 1.0 } else { 0.0 }, 0.0);

        if !animation.started {
            animation.started = true;
            reset_anim();
        }

        animation.loop_progress =
            ui.ctx().animate_value_with_time(animation.id, if reverse { 0.0 } else { 1.0 }, duration);

        if looped && animation.loop_progress == 1.0 {
            animation.loop_progress = reset_anim();
        }

        let frame_idx = (animation.loop_progress * animation.frames.len() as f32) as usize;
        let frame = &animation.frames[frame_idx.min(animation.frames.len() - 1)];
        ui.image(frame, size)
    }
}

impl<'a> Flipbook<'a> {
    pub fn new(animation: &'a mut AnimationState, size: impl Into<Vec2>) -> Self {
        Self { animation, size: size.into(), duration: 1.0, looped: false, reverse: false }
    }

    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub fn fps(mut self, fps: f32) -> Self {
        self.duration = self.animation.frames.len() as f32 / fps;
        self
    }

    pub fn looped(mut self, looped: bool) -> Self {
        self.looped = looped;
        self
    }

    pub fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }
}
