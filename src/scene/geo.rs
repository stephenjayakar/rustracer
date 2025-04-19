extern crate nalgebra as na;

use std::f64::consts::PI;
use std::fmt;
use std::ops::{Add, Mul, Sub};

use na::base::{Matrix3, Vector3};
use na::geometry::Point3;

use crate::common::EPS;

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

    pub fn origin() -> Point {
        Point::new(0.0, 0.0, 0.0)
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

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "p({} {} {})", self.x(), self.y(), self.z())
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
    pub fn random_hemisphere() -> Vector {
        // creating a random vector in object space
        let xi1 = fastrand::f64();
        let xi2 = fastrand::f64();

        let theta = f64::acos(xi1);
        let phi = 2.0 * PI * xi2;
        let xs = f64::sin(theta) * f64::cos(phi);
        let ys = f64::sin(theta) * f64::sin(phi);
        let zs = f64::cos(theta);

        Vector::new(xs, ys, zs)
    }

    pub fn to_coord_space(&self, normal: Vector) -> Vector {
        // make_coord_space from 184.  make it a function if we use it again
        // TODO: unsure if these clones are necessary
        // special handling if normal is (0, 1, 0), as cross products will be undefined.
        // other behavior
        let mut z = normal.v.clone();
        let mut h = z.clone();
        if f64::abs(h.x) <= f64::abs(h.y) && f64::abs(h.x) <= f64::abs(h.z) {
            h.y = 1.0;
            // TODO: Unsure about these.  meant to fix the cross multiplication issues.
            h.x += EPS;
            h.z += EPS;
        } else {
            h.z = 1.0;
            h.x += EPS;
            h.y += EPS;
        }

        z = z.normalize();
        let y = h.cross(&z).normalize();
        let x = z.cross(&y).normalize();

        let o2w = Matrix3::from_columns(&[x, y, z]);
        Vector::new_from_na(o2w * self.v)
    }

    /// Samples uniformly on a unit sphere and returns the associated vector
    pub fn random_sphere() -> Vector {
        // TODO: figure out how to not repeat this for hemisphere
        let xi1 = fastrand::f64();
        let xi2 = fastrand::f64();

        let theta = 2.0 * PI * xi1;
        let phi = f64::acos(1.0 - 2.0 * xi2);
        let xs = f64::sin(phi) * f64::cos(theta);
        let ys = f64::sin(phi) * f64::sin(theta);
        let zs = f64::cos(phi);
        Vector::new(xs, ys, zs)
    }

    pub fn norm(&self) -> f64 {
        self.v.norm()
    }

    pub fn normalized(&self) -> Vector {
        Vector::new_from_na(self.v.normalize())
    }

    pub fn dot(&self, other_vector: Vector) -> f64 {
        // Unrolled dot product for better performance
        let (x1, y1, z1) = (self.x(), self.y(), self.z());
        let (x2, y2, z2) = (other_vector.x(), other_vector.y(), other_vector.z());
        x1 * x2 + y1 * y2 + z1 * z2
    }

    pub fn cross(&self, other_vector: Vector) -> Vector {
        Vector::new_from_na(self.v.cross(&other_vector.v))
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

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v({} {} {})", self.x(), self.y(), self.z())
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

impl Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, other: Vector) -> Self::Output {
        Point::new_from_na(self.p - other.v)
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

impl Mul<f64> for Vector {
    type Output = Vector;
    fn mul(self, other: f64) -> Vector {
        Vector::new_from_na(self.v * other)
    }
}
