extern crate sdl2;

use std::ops::{Add, AddAssign, Mul};

pub const GENERIC_ERROR: &str = "Something went wrong, sorry!";
pub const EPS: f32 = 0.0000001;

// struct that is essentially a wrapper on top of SDL2::Color, but allows accumulation
#[derive(Clone, Copy, Debug)]
pub struct Spectrum {
    r: f32,
    g: f32,
    b: f32,
}

impl core::fmt::Display for Spectrum {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "r:{} g:{} b:{}", self.r, self.g, self.b)
    }
}

impl Spectrum {
    pub fn to_sdl2_color(&self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGB(self.r(), self.g(), self.b())
    }

    pub fn new(r: u8, g: u8, b: u8) -> Spectrum {
        Spectrum {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    pub fn new_f(r: f32, g: f32, b: f32) -> Spectrum {
        Spectrum { r, g, b }
    }

    #[inline(always)]
    pub fn is_black(&self) -> bool {
        let color = self.r + self.g + self.b;
        color <= EPS
    }

    #[inline(always)]
    fn to_u8(val: f32) -> u8 {
        if val > 255.0 {
            255
        } else {
            val as u8
        }
    }

    #[inline(always)]
    fn to_color(intensity: f32) -> u8 {
        // pow(...) is gamma correction
        Spectrum::to_u8(f32::powf(intensity.clamp(0.0, 1.0), 1.0 / 2.2) * 255.0)
    }

    pub fn r(&self) -> u8 {
        Spectrum::to_color(self.r)
    }

    pub fn g(&self) -> u8 {
        Spectrum::to_color(self.g)
    }

    pub fn b(&self) -> u8 {
        Spectrum::to_color(self.b)
    }

    pub fn black() -> Spectrum {
        Spectrum::new_f(0.0, 0.0, 0.0)
    }

    pub fn white() -> Spectrum {
        Spectrum::new_f(1.0, 1.0, 1.0)
    }

    pub fn grey() -> Spectrum {
        Spectrum::new_f(0.78, 0.78, 0.78)
    }

    pub fn red() -> Spectrum {
        Spectrum::new_f(1.0, 0.0, 0.0)
    }

    pub fn blue() -> Spectrum {
        Spectrum::new_f(0.0, 0.0, 1.0)
    }

    pub fn green() -> Spectrum {
        Spectrum::new_f(0.0, 1.0, 0.0)
    }

    pub fn purple() -> Spectrum {
        Spectrum::new_f(0.5, 0.0, 0.5)
    }
}

impl Add for Spectrum {
    type Output = Spectrum;
    #[inline(always)]
    fn add(self, other: Spectrum) -> Self::Output {
        Spectrum::new_f(self.r + other.r, self.g + other.g, self.b + other.b)
    }
}

impl AddAssign for Spectrum {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        self.r += other.r;
        self.g += other.g;
        self.b += other.b;
    }
}

impl Mul for Spectrum {
    type Output = Spectrum;
    #[inline(always)]
    fn mul(self, other: Spectrum) -> Self::Output {
        Spectrum::new_f(self.r * other.r, self.g * other.g, self.b * other.b)
    }
}

impl Mul<f32> for Spectrum {
    type Output = Spectrum;
    #[inline(always)]
    fn mul(self, other: f32) -> Self::Output {
        Spectrum::new_f(self.r * other, self.g * other, self.b * other)
    }
}

/// Given the probablity to flip heads, returns true if the coin flips heads.
#[inline(always)]
pub fn weighted_coin_flip(probability: f32) -> bool {
    fastrand::f32() <= probability
}
