use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct ColorGenerator {
    rng: oorandom::Rand32,
    region_colors: Vec<Option<Color>>,
}

impl Default for ColorGenerator {
    fn default() -> Self {
        Self {
            rng: oorandom::Rand32::new(123456),
            region_colors: Vec::new(),
        }
    }
}

impl ColorGenerator {
    pub fn gen_color(&mut self) -> Color {
        Color::rgb(
            self.rng.rand_float(),
            self.rng.rand_float(),
            self.rng.rand_float(),
        )
    }

    pub fn outline_color(color: Color) -> Color {
        if cfg!(feature = "dim2") {
            let [h, s, l, a] = color.as_hsla_f32();
            Color::hsla(h, s, l * 1.2, a)
        } else {
            color
        }
    }

    pub fn gen_region_color(&mut self, region: usize) -> Color {
        if self.region_colors.len() <= region {
            self.region_colors.resize(region + 1, None);
        }

        let color = &mut self.region_colors[region];

        if color.is_none() {
            *color = Some(Color::rgb(
                self.rng.rand_float(),
                self.rng.rand_float(),
                self.rng.rand_float(),
            ));
        }

        color.unwrap()
    }
}
