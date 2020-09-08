extern crate sdl2;

use std::ops::{Add, AddAssign, Mul, Sub};

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



#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
    pub bounces_left: u32,
}

impl Ray {
    // normalizes the direction vector
    pub fn new(origin: Point, direction: Vector, bounces_left: u32) -> Ray {
	Ray {
	    origin,
	    direction: direction.normalized(),
	    bounces_left,
	}
    }

    pub fn get_intersection_point(&self, scalar: f64) -> Point {
	let scaled_vector = self.direction.scale(scalar);
	self.origin + scaled_vector
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Point {
	Point { x, y, z }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

fn norm(x: f64, y: f64, z: f64) -> f64 {
    f64::sqrt(f64::powi(x, 2) + f64::powi(y, 2) + f64::powi(z, 2))
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
	Vector { x, y, z }
    }

    pub fn new_normalized(x: f64, y: f64, z: f64) -> Vector {
	let mut vector = Vector::new(x, y, z);
	vector.normalize();
	vector
    }

    pub fn norm(&self) -> f64 {
	norm(self.x, self.y, self.z)
    }

    pub fn normalize(&mut self) {
	let inverse_norm = 1.0 / self.norm();
	self.x *= inverse_norm;
	self.y *= inverse_norm;
	self.z *= inverse_norm;
    }

    pub fn normalized(&self) -> Vector {
	let mut return_vector = *self;
	return_vector.normalize();
	return_vector
    }

    pub fn dot(&self, other_vector: Vector) -> f64 {
	self.x * other_vector.x +
	    self.y * other_vector.y +
	    self.z * other_vector.z
    }

    pub fn scale(&self, scalar: f64) -> Vector {
	Vector::new(
	    scalar * self.x,
	    scalar * self.y,
	    scalar * self.z,
	)
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, other: Point) -> Self::Output {
	Vector::new(
	    self.x - other.x,
	    self.y - other.y,
	    self.z - other.z,
	)
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, other: Vector) -> Self::Output {
	Point::new(
	    self.x + other.x,
	    self.y + other.y,
	    self.z + other.z,
	)
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Self::Output {
	Vector::new(
	    self.x + other.x,
	    self.y + other.y,
	    self.z + other.z,
	)
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Self::Output {
	Vector::new(
	    self.x - other.x,
	    self.y - other.y,
	    self.z - other.z,
	)
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

