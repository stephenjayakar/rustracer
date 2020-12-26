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

    /// Basically copied this from my 184 project, as the naive way that I was going
    /// to implement was biased towards vectors going towards the normal.  Oops.
    /// TODO: Figure out how this works.
    pub fn random_hemisphere(normal: Vector) -> Vector {
        // creating a random vector in object space
        let xi1 = fastrand::f64();
        let xi2 = fastrand::f64();

        let theta = f64::acos(xi1);
        let phi = 2.0 * std::f64::consts::PI * xi2;
        let xs = f64::sin(theta) * f64::cos(phi);
        let ys = f64::sin(theta) * f64::sin(phi);
        let zs = f64::cos(theta);
        // TODO: transform with normal

        Vector::new(xs, ys, zs)
    }

    pub fn norm(&self) -> f64 {
        self.v.norm()
    }

    pub fn normalized(&self) -> Vector {
        Vector::new_from_na(self.v.normalize())
    }

    pub fn dot(&self, other_vector: Vector) -> f64 {
        self.v.dot(&other_vector.v)
    }

    pub fn scale(&self, scalar: f64) -> Vector {
        Vector::new_from_na(self.v * scalar)
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
