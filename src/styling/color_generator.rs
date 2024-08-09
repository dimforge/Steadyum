use bevy::prelude::*;

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
        Color::srgb(
            self.rng.rand_float(),
            self.rng.rand_float(),
            self.rng.rand_float(),
        )
    }

    pub fn outline_color(color: Color) -> Color {
        if cfg!(feature = "dim2") {
            let [h, s, l, a] = Hsla::from(color).to_f32_array();
            Color::hsla(h, s, l * 1.2, a)
        } else {
            color
        }
    }

    pub fn gen_region_color(&mut self, region: usize) -> Color {
        if self.region_colors.len() <= region {
            self.region_colors.resize(region + 1, None);
        }

        let mut seeded_rng = oorandom::Rand32::new(region as u64 % 10);
        let color = &mut self.region_colors[region];

        if color.is_none() {
            *color = Some(Color::srgb(
                seeded_rng.rand_float(),
                seeded_rng.rand_float(),
                seeded_rng.rand_float(),
            ));
        }

        color.unwrap()
    }
}
