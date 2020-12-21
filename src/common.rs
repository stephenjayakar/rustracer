extern crate sdl2;

use std::ops::{Add, AddAssign, Mul};

// struct that is essentially a wrapper on top of SDL2::Color, but allows accumulation
#[derive(Clone, Copy, Debug)]
pub struct Spectrum {
    r: u8,
    g: u8,
    b: u8,
}

impl Spectrum {
    pub fn to_sdl2_color(&self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGB(self.r, self.g, self.b)
    }

    pub fn new(r: u8, g: u8, b: u8) -> Spectrum {
        Spectrum {
            r, g, b
        }
    }
}

// note: this will panic on overflow.  be careful!
impl Add for Spectrum {
    type Output = Spectrum;
    fn add(self, other: Spectrum) -> Self::Output {
        Spectrum::new(self.r + other.r,
                      self.g + other.g,
                      self.b + other.b)
    }
}

impl AddAssign for Spectrum {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Mul<f64> for Spectrum {
    type Output = Spectrum;
    fn mul(self, other: f64) -> Self::Output {
        // should probably panic if out of range
        let new_r = self.r as f64 * other;
        let new_g = self.g as f64 * other;
        let new_b = self.b as f64 * other;
        debug_assert!(new_r <= 255.0 &&
                      new_g <= 255.0 &&
                      new_b <= 255.0);
        unsafe {
            Spectrum::new((self.r as f64 * other).to_int_unchecked(),
                          (self.g as f64 * other).to_int_unchecked(),
                          (self.b as f64 * other).to_int_unchecked())
        }
    }
}

