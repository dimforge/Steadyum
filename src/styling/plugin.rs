use crate::styling::ColorGenerator;
use bevy::prelude::*;
use bevy_egui::egui::Visuals;

pub struct StylingPlugin;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Resource)]
pub struct Theme {
    pub dark_mode: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self { dark_mode: true }
    }
}

impl Theme {
    pub fn ui_visuals(&self) -> Visuals {
        if self.dark_mode {
            Visuals::dark()
        } else {
            Visuals::light()
        }
    }

    pub fn background_color(&self) -> Color {
        if self.dark_mode {
            Color::DARK_GRAY
        } else {
            Color::WHITE
        }
    }

    pub fn insertion_preview_color(&self) -> Color {
        if self.dark_mode {
            Color::WHITE
        } else {
            Color::BLACK
        }
    }

    pub fn floor_minor_line_color(&self) -> Color {
        if self.dark_mode {
            Color::rgb(0.1, 0.1, 0.1)
        } else {
            Color::rgb(0.7, 0.7, 0.7)
        }
    }

    pub fn floor_major_line_color(&self) -> Color {
        if self.dark_mode {
            Color::rgb(0.25, 0.25, 0.25)
        } else {
            Color::rgb(0.25, 0.25, 0.25)
        }
    }
}

impl Plugin for StylingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColorGenerator::default())
            .insert_resource(Theme::default())
            .add_systems(Update, super::dark_mode::update_dark_mode);
    }
}
