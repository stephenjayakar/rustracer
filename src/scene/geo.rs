use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

impl Ray {
    // normalizes the direction vector
    pub fn new(origin: Point, direction: Vector) -> Ray {
        Ray {
            origin,
            direction: direction.normalized(),
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
        self.x * other_vector.x + self.y * other_vector.y + self.z * other_vector.z
    }

    pub fn scale(&self, scalar: f64) -> Vector {
        Vector::new(scalar * self.x, scalar * self.y, scalar * self.z)
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, other: Point) -> Self::Output {
        Vector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, other: Vector) -> Self::Output {
        Point::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Self::Output {
        Vector::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Self::Output {
        Vector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}
