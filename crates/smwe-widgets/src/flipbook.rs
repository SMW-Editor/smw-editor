use egui::*;
use itertools::Itertools;
use thiserror::Error;

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Error)]
#[error("Frame size is invalid: {0:?}")]
pub struct AtlasInvalidSizeError([usize; 2]);

#[derive(Copy, Clone, Debug, Error)]
#[error("Frame size is invalid: {0:?}")]
pub struct FrameInvalidSizeError([usize; 2]);

#[derive(Copy, Clone, Debug, Error)]
#[error("All animation frames must have the same size")]
pub struct FramesUnequalSizeError;

#[derive(Copy, Clone, Debug, Error)]
#[error("Frame size {frame_size:?} extends outside atlas size {atlas_size:?}")]
pub struct FrameBiggerThanAtlasError {
    frame_size: [usize; 2],
    atlas_size: [usize; 2],
}

#[derive(Copy, Clone, Debug, Error)]
#[error("Atlas size {atlas_size:?} is not perfectly divisible by frame size {frame_size:?}")]
pub struct AtlasSizeIndivisibleByFrameSizeError {
    frame_size: [usize; 2],
    atlas_size: [usize; 2],
}

#[derive(Copy, Clone, Debug, Error)]
#[error("Each animation must have at least one frame")]
pub struct EmptyAnimationError;

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct AnimationState {
    id:            Id,
    atlas:         TextureHandle,
    frame_count:   usize,
    frame_size:    [usize; 2],
    loop_progress: f32,
    started:       bool,
}

pub struct Flipbook<'a> {
    animation: &'a mut AnimationState,
    size:      Vec2,
    duration:  f32,
    looped:    bool,
    reverse:   bool,
}

// -------------------------------------------------------------------------------------------------

impl AnimationState {
    pub fn from_frames(frames: Vec<ColorImage>, id: impl Into<Id>, ctx: &Context) -> anyhow::Result<Self> {
        if frames.is_empty() {
            Err(EmptyAnimationError.into())
        } else if !frames.iter().map(|frame| frame.size).all_equal() {
            Err(FramesUnequalSizeError.into())
        } else {
            let frame_count = frames.len();
            let frame_size = frames[0].size;
            let atlas_size = [frame_size[0], frame_size[1] * frames.len()];
            let mut atlas = ColorImage::new(atlas_size, Color32::BLACK);
            for (idx, frame) in frames.into_iter().enumerate() {
                let top_left_idx = idx * frame_size[1];
                for y in 0..frame_size[1] {
                    let atlas_y = y + top_left_idx;
                    for x in 0..frame_size[0] {
                        atlas[(x, atlas_y)] = frame[(x, y)];
                    }
                }
            }
            Self::from_atlas(atlas, frame_count, frame_size, id, ctx)
        }
    }

    pub fn from_atlas(
        atlas: ColorImage, frame_count: usize, frame_size: [usize; 2], id: impl Into<Id>, ctx: &Context,
    ) -> anyhow::Result<Self> {
        if atlas.size.contains(&0) {
            Err(AtlasInvalidSizeError(atlas.size).into())
        } else if frame_size.contains(&0) {
            Err(FrameInvalidSizeError(frame_size).into())
        } else if (atlas.size[0] < frame_size[0]) || (atlas.size[1] < frame_size[1]) {
            Err(FrameBiggerThanAtlasError { atlas_size: atlas.size, frame_size }.into())
        } else if (atlas.size[0] % frame_size[0] != 0) || (atlas.size[1] % frame_size[1] != 0) {
            Err(AtlasSizeIndivisibleByFrameSizeError { atlas_size: atlas.size, frame_size }.into())
        } else {
            let id: Id = id.into();
            let atlas = ctx.load_texture(format!("anim_atlas_{id:?}"), atlas, TextureOptions::NEAREST);
            Ok(Self { atlas, frame_count, frame_size, loop_progress: 0.0, id, started: false })
        }
    }
}

impl<'a> Widget for Flipbook<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::focusable_noninteractive());
        let shape = self.to_shape(ui, rect);
        ui.painter().add(shape);
        response
    }
}

// UI
impl<'a> Flipbook<'a> {
    pub fn to_shape(self, ui: &Ui, rect: Rect) -> Shape {
        let Self { animation, duration, size: _, looped, reverse } = self;
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

        let frame_idx =
            ((animation.loop_progress * animation.frame_count as f32) as usize).min(animation.frame_count - 1);
        let atlas_size = animation.atlas.size_vec2();
        let frames_in_row = atlas_size.x as usize / animation.frame_size[0];
        let uv = Rect::from_min_max(
            Pos2::new(
                ((animation.frame_size[0] * (frame_idx % frames_in_row)) as f32 + 0.5) / atlas_size.x,
                ((animation.frame_size[1] * (frame_idx / frames_in_row)) as f32 + 0.5) / atlas_size.y,
            ),
            Pos2::new(
                ((animation.frame_size[0] * ((frame_idx % frames_in_row) + 1)) as f32 - 0.5) / atlas_size.x,
                ((animation.frame_size[1] * ((frame_idx / frames_in_row) + 1)) as f32 - 0.5) / atlas_size.y,
            ),
        );

        Shape::image(animation.atlas.id(), rect, uv, Color32::WHITE)
    }
}

// Builder
impl<'a> Flipbook<'a> {
    pub fn new(animation: &'a mut AnimationState, size: impl Into<Vec2>) -> Self {
        Self { animation, size: size.into(), duration: 1.0, looped: false, reverse: false }
    }

    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub fn fps(mut self, fps: f32) -> Self {
        self.duration = self.animation.frame_count as f32 / fps;
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
