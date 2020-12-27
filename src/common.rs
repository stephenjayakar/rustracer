extern crate sdl2;

use std::ops::{Add, AddAssign, Mul};

pub const GENERIC_ERROR: &str = "Something went wrong, sorry!";
pub const DEFAULT_SCREEN_WIDTH: u32 = 1200;
pub const DEFAULT_SCREEN_HEIGHT: u32 = 1200;
pub const EPS: f64 = 0.0000001;

// struct that is essentially a wrapper on top of SDL2::Color, but allows accumulation
#[derive(Clone, Copy, Debug)]
pub struct Spectrum {
    r: u8,
    g: u8,
    b: u8,
}

fn color_to_intensity(color: u8) -> f64 {
    color as f64 / 255.0
}

fn intensity_to_color(intensity: f64) -> u8 {
    (intensity * 255.0) as u8
}

impl Spectrum {
    pub fn to_sdl2_color(&self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGB(self.r, self.g, self.b)
    }

    pub fn new(r: u8, g: u8, b: u8) -> Spectrum {
        Spectrum { r, g, b }
    }

    pub fn is_black(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0
    }
}

// note: this will not panic on overflow.  be careful!
impl Add for Spectrum {
    type Output = Spectrum;
    fn add(self, other: Spectrum) -> Self::Output {
        Spectrum::new(
            self.r.saturating_add(other.r),
            self.g.saturating_add(other.g),
            self.b.saturating_add(other.b),
        )
    }
}

impl AddAssign for Spectrum {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Mul for Spectrum {
    type Output = Spectrum;
    fn mul(self, other: Spectrum) -> Self::Output {
        let (r, g, b) = (
            color_to_intensity(self.r),
            color_to_intensity(self.g),
            color_to_intensity(self.b),
        );
        let (other_r, other_g, other_b) = (
            color_to_intensity(other.r),
            color_to_intensity(other.g),
            color_to_intensity(other.b),
        );
        Spectrum::new(
            intensity_to_color(r * other_r),
            intensity_to_color(g * other_g),
            intensity_to_color(b * other_b),
        )
    }
}

impl Mul<f64> for Spectrum {
    type Output = Spectrum;
    fn mul(self, other: f64) -> Self::Output {
        // should probably panic if out of range
        let new_r = self.r as f64 * other;
        let new_g = self.g as f64 * other;
        let new_b = self.b as f64 * other;
        debug_assert!(new_r <= 255.0 && new_g <= 255.0 && new_b <= 255.0);
        unsafe {
            Spectrum::new(
                (self.r as f64 * other).to_int_unchecked(),
                (self.g as f64 * other).to_int_unchecked(),
                (self.b as f64 * other).to_int_unchecked(),
            )
        }
    }
}
