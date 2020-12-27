extern crate sdl2;

use std::ops::{Add, AddAssign, Mul};

pub const GENERIC_ERROR: &str = "Something went wrong, sorry!";
pub const DEFAULT_SCREEN_WIDTH: u32 = 1200;
pub const DEFAULT_SCREEN_HEIGHT: u32 = 1200;
pub const EPS: f64 = 0.0000001;

// struct that is essentially a wrapper on top of SDL2::Color, but allows accumulation
#[derive(Clone, Copy, Debug)]
pub struct Spectrum {
    r: u32,
    g: u32,
    b: u32,
}

impl Spectrum {
    pub fn to_sdl2_color(&self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGB(self.r(), self.g(), self.b())
    }

    pub fn new(r: u32, g: u32, b: u32) -> Spectrum {
        Spectrum { r, g, b }
    }

    pub fn is_black(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0
    }

    fn to_u8(val: u32) -> u8 {
        // maybe make this debug somehow?
        if val > std::u8::MAX as u32 {
            std::u8::MAX
        } else {
            val as u8
        }
    }

    fn to_intensity(val: u8) -> f64 {
        val as f64 / 255.0
    }

    fn to_color(intensity: f64) -> u32 {
        (intensity * 255.0) as u32
    }

    pub fn r(&self) -> u8 {
        Spectrum::to_u8(self.r)
    }

    pub fn g(&self) -> u8 {
        Spectrum::to_u8(self.g)
    }

    pub fn b(&self) -> u8 {
        Spectrum::to_u8(self.b)
    }

    pub fn ri(&self) -> f64 {
        Spectrum::to_intensity(self.r())
    }

    pub fn gi(&self) -> f64 {
        Spectrum::to_intensity(self.g())
    }

    pub fn bi(&self) -> f64 {
        Spectrum::to_intensity(self.b())
    }
}

// note: this will not panic on overflow.  be careful!
impl Add for Spectrum {
    type Output = Spectrum;
    fn add(self, other: Spectrum) -> Self::Output {
        Spectrum::new(self.r + other.r, self.g + other.g, self.b + other.b)
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
        Spectrum::new(
            Spectrum::to_color(self.ri() * other.ri()),
            Spectrum::to_color(self.gi() * other.gi()),
            Spectrum::to_color(self.bi() * other.bi()),
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
