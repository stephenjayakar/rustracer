extern crate nalgebra as na;

use na::base::Vector3;
use na::geometry::Point3;

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
    p: Point3<f64>,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Point {
        let p = Point3::new(x, y, z);
        Point::new_from_na(p)
    }

    fn new_from_na(p: Point3<f64>) -> Point {
        Point { p }
    }

    pub fn x(&self) -> f64 {
        self.p[0]
    }

    pub fn y(&self) -> f64 {
        self.p[1]
    }

    pub fn z(&self) -> f64 {
        self.p[2]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    v: Vector3<f64>,
}

fn norm(x: f64, y: f64, z: f64) -> f64 {
    f64::sqrt(f64::powi(x, 2) + f64::powi(y, 2) + f64::powi(z, 2))
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        let v = Vector3::new(x, y, z);
        Vector::new_from_na(v)
    }

    fn new_from_na(v: Vector3<f64>) -> Vector {
        Vector { v }
    }

    pub fn new_normalized(x: f64, y: f64, z: f64) -> Vector {
        Vector::new(x, y, z).normalized()
    }

    pub fn norm(&self) -> f64 {
        norm(self.x(), self.y(), self.z())
    }

    pub fn normalized(&self) -> Vector {
        let inverse_norm = 1.0 / self.norm();
        let x = self.x() * inverse_norm;
        let y = self.y() * inverse_norm;
        let z = self.z() * inverse_norm;

        Vector::new(x, y, z)
    }

    pub fn dot(&self, other_vector: Vector) -> f64 {
        self.x() * other_vector.x() + self.y() * other_vector.y() + self.z() * other_vector.z()
    }

    pub fn scale(&self, scalar: f64) -> Vector {
        Vector::new(scalar * self.x(), scalar * self.y(), scalar * self.z())
    }

    pub fn x(&self) -> f64 {
        self.v[0]
    }

    pub fn y(&self) -> f64 {
        self.v[1]
    }

    pub fn z(&self) -> f64 {
        self.v[2]
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, other: Point) -> Self::Output {
        Vector::new_from_na(self.p - other.p)
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, other: Vector) -> Self::Output {
        Point::new_from_na(self.p + other.v)
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Self::Output {
        Vector::new_from_na(self.v + other.v)
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Self::Output {
        Vector::new_from_na(self.v - other.v)
    }
}
