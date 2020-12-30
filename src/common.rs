extern crate sdl2;

use std::ops::{Add, AddAssign, Mul};

pub const GENERIC_ERROR: &str = "Something went wrong, sorry!";
pub const DEFAULT_SCREEN_WIDTH: u32 = 1200;
pub const DEFAULT_SCREEN_HEIGHT: u32 = 1200;
pub const EPS: f64 = 0.0000001;

// struct that is essentially a wrapper on top of SDL2::Color, but allows accumulation
#[derive(Clone, Copy, Debug)]
pub struct Spectrum {
    r: f64,
    g: f64,
    b: f64,
}

impl Spectrum {
    pub fn to_sdl2_color(&self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGB(self.r(), self.g(), self.b())
    }

    pub const fn new(r: u8, g: u8, b: u8) -> Spectrum {
        Spectrum { r: r as f64 / 255.0,
				   g: g as f64 / 255.0,
				   b: b as f64 / 255.0}
    }

	pub const fn new_f(r: f64, g: f64, b: f64) -> Spectrum {
		Spectrum { r, g, b }
	}

    pub fn is_black(&self) -> bool {
		let color = self.r + self.g + self.b;
		color <= 0.0 + EPS as f64
    }

    fn to_u8(val: f64) -> u8 {
        // maybe make this debug somehow?
        if val > std::u8::MAX as f64 {
            std::u8::MAX
        } else {
            val as u8
        }
    }

    fn to_color(intensity: f64) -> u8 {
		// pow(...) is gamma correction
        Spectrum::to_u8(f64::powf(intensity.clamp(0.0, 1.0), 1.0 / 2.2) * 255.0)
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

	// Unsure how I feel about these, as we're clamping on return
    // fn ri(&self) -> f64 {
	// 	f64::min(1.0, self.r)
    // }

    // fn gi(&self) -> f64 {
	// 	f64::min(1.0, self.g)
    // }

    // fn bi(&self) -> f64 {
	// 	f64::min(1.0, self.b)
    // }

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

	pub const fn purple() -> Spectrum {
		Spectrum::new_f(0.5, 0.0, 0.5)
	}
}

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
            self.r * other.r,
            self.g * other.g,
            self.b * other.b)
    }
}

impl Mul<f64> for Spectrum {
    type Output = Spectrum;
    fn mul(self, other: f64) -> Self::Output {
        let new_r = self.r * other;
        let new_g = self.g * other;
        let new_b = self.b * other;
        Spectrum::new_f(
			new_r,
			new_g,
			new_b,
        )
    }
}
