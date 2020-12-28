extern crate sdl2;

use std::ops::{Add, AddAssign, Mul};

pub const GENERIC_ERROR: &str = "Something went wrong, sorry!";
pub const DEFAULT_SCREEN_WIDTH: u32 = 1200;
pub const DEFAULT_SCREEN_HEIGHT: u32 = 1200;
pub const EPS: f64 = 0.0000001;

// struct that is essentially a wrapper on top of SDL2::Color, but allows accumulation
#[derive(Clone, Copy, Debug)]
pub struct Spectrum {
    r: f32,
    g: f32,
    b: f32,
}

impl Spectrum {
    pub fn to_sdl2_color(&self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGB(self.r(), self.g(), self.b())
    }

    pub fn new(r: u32, g: u32, b: u32) -> Spectrum {
        Spectrum { r: r as f32 / 255.0,
				   g: g as f32 / 255.0,
				   b: b as f32 / 255.0}
    }

	const fn new_f(r: f32, g: f32, b: f32) -> Spectrum {
		Spectrum { r, g, b }
	}

    pub fn is_black(&self) -> bool {
		let color = self.r + self.g + self.b;
		color <= 0.0 + EPS as f32
    }

    fn to_u8(val: f32) -> u8 {
        // maybe make this debug somehow?
        if val > std::u8::MAX as f32 {
            std::u8::MAX
        } else {
            val as u8
        }
    }

    fn to_color(intensity: f32) -> u8 {
        Spectrum::to_u8(intensity * 255.0)
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

    fn ri(&self) -> f32 {
		f32::min(1.0, self.r)
    }

    fn gi(&self) -> f32 {
		f32::min(1.0, self.g)
    }

    fn bi(&self) -> f32 {
		f32::min(1.0, self.b)
    }

	pub const fn black() -> Spectrum {
		Spectrum::new_f(0.0, 0.0, 0.0)
	}

	pub const fn white() -> Spectrum {
		Spectrum::new_f(1.0, 1.0, 1.0)
	}

	pub const fn grey() -> Spectrum {
		Spectrum::new_f(0.78, 0.78, 0.78)
	}

	pub const fn red() -> Spectrum {
		Spectrum::new_f(1.0, 0.0, 0.0)
	}

	pub const fn blue() -> Spectrum {
		Spectrum::new_f(0.0, 0.0, 1.0)
	}

	pub const fn green() -> Spectrum {
		Spectrum::new_f(0.0, 1.0, 0.0)
	}
}

// note: this will not panic on overflow.  be careful!
impl Add for Spectrum {
    type Output = Spectrum;
    fn add(self, other: Spectrum) -> Self::Output {
        Spectrum::new_f(self.r + other.r, self.g + other.g, self.b + other.b)
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
        Spectrum::new_f(
            self.ri() * other.ri(),
            self.gi() * other.gi(),
            self.bi() * other.bi())
    }
}

impl Mul<f32> for Spectrum {
    type Output = Spectrum;
    fn mul(self, other: f32) -> Self::Output {
        // should probably panic if out of range
        let new_r = self.r * other;
        let new_g = self.g * other;
        let new_b = self.b * other;
        debug_assert!(new_r <= 255.0 && new_g <= 255.0 && new_b <= 255.0);
        Spectrum::new_f(
			new_r,
			new_g,
			new_b,
        )
    }
}
