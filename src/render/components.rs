use bevy::prelude::Color;
use bevy::prelude::*;

pub const DEFAULT_COLOR: Color = Color::Srgba(bevy::color::palettes::css::BEIGE);
pub const DEFAULT_PALETTE: [Color; 3] = [
    Color::srgb(
        0x98 as f32 / 255.0,
        0xC1 as f32 / 255.0,
        0xD9 as f32 / 255.0,
    ),
    Color::srgb(
        0x05 as f32 / 255.0,
        0x3C as f32 / 255.0,
        0x5E as f32 / 255.0,
    ),
    Color::srgb(
        0x1F as f32 / 255.0,
        0x7A as f32 / 255.0,
        0x8C as f32 / 255.0,
    ),
];

/*
 * Shape surface rendering..
 */
#[derive(Copy, Clone, Component)]
pub struct ColliderRender {
    pub color: Color,
}

impl Default for ColliderRender {
    fn default() -> Self {
        DEFAULT_COLOR.into()
    }
}

impl From<Color> for ColliderRender {
    fn from(color: Color) -> Self {
        Self { color }
    }
}

impl ColliderRender {
    pub fn with_id(i: usize) -> Self {
        DEFAULT_PALETTE[i % DEFAULT_PALETTE.len()].into()
    }
}

#[derive(Copy, Clone, Component, Default)]
pub struct ColliderRenderTargets {
    pub target: Option<Entity>,
    pub outline_target: Option<Entity>,
}

/*
 * Shape outline rendering.
 */
#[derive(Copy, Clone, Component)]
pub struct ColliderOutlineRender {
    pub color: Color,
    pub thickness: f32,
}

impl ColliderOutlineRender {
    pub fn new(color: Color, thickness: f32) -> Self {
        Self { color, thickness }
    }
}

impl Default for ColliderOutlineRender {
    fn default() -> Self {
        DEFAULT_COLOR.into()
    }
}

impl From<Color> for ColliderOutlineRender {
    fn from(color: Color) -> Self {
        Self {
            color,
            thickness: 1.0,
        }
    }
}

impl ColliderOutlineRender {
    pub fn with_id(i: usize) -> Self {
        DEFAULT_PALETTE[i % DEFAULT_PALETTE.len()].into()
    }
}

/*
 * Joint rendering.
 */
pub const DEFAULT_JOINT_ANCHOR_COLOR: Color = Color::srgb(0.0, 0.0, 1.0);
pub const DEFAULT_JOINT_SEPARATION_COLOR: Color = Color::srgb(1.0, 0.0, 1.0);

#[derive(Copy, Clone, Component)]
pub struct JointRender {
    pub anchor_color: Color,
    pub separation_color: Color,
}

impl JointRender {
    pub fn new(anchor_color: Color, separation_color: Color) -> Self {
        Self {
            anchor_color,
            separation_color,
        }
    }
}

impl Default for JointRender {
    fn default() -> Self {
        Self::new(DEFAULT_JOINT_ANCHOR_COLOR, DEFAULT_JOINT_SEPARATION_COLOR)
    }
}
