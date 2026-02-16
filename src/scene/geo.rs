extern crate nalgebra as na;

use std::f32::consts::PI;
use std::fmt;
use std::ops::{Add, Mul, Sub};

use na::base::Vector3;
use na::geometry::Point3;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

impl Ray {
    // normalizes the direction vector
    #[inline(always)]
    pub fn new(origin: Point, direction: Vector) -> Ray {
        Ray {
            origin,
            direction: direction.normalized(),
        }
    }

    /// Create a ray with an already-normalized direction (skip redundant normalize)
    #[inline(always)]
    pub fn new_prenormalized(origin: Point, direction: Vector) -> Ray {
        Ray { origin, direction }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    p: Point3<f32>,
}

impl Point {
    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32) -> Point {
        Point {
            p: Point3::new(x, y, z),
        }
    }

    pub fn origin() -> Point {
        Point::new(0.0, 0.0, 0.0)
    }

    #[inline(always)]
    fn new_from_na(p: Point3<f32>) -> Point {
        Point { p }
    }

    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.p[0]
    }

    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.p[1]
    }

    #[inline(always)]
    pub fn z(&self) -> f32 {
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
    v: Vector3<f32>,
}

impl Vector {
    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32) -> Vector {
        Vector {
            v: Vector3::new(x, y, z),
        }
    }

    #[inline(always)]
    fn new_from_na(v: Vector3<f32>) -> Vector {
        Vector { v }
    }

    #[inline(always)]
    pub fn new_normalized(x: f32, y: f32, z: f32) -> Vector {
        Vector::new(x, y, z).normalized()
    }

    /// Uniform hemisphere sampling (optimized: avoid acos, use sin_cos)
    pub fn random_hemisphere() -> Vector {
        let xi1 = fastrand::f32();
        let xi2 = fastrand::f32();

        let cos_theta = xi1;
        let sin_theta = f32::sqrt(1.0 - xi1 * xi1);
        let phi = 2.0 * PI * xi2;
        let (sin_phi, cos_phi) = f32::sin_cos(phi);
        let xs = sin_theta * cos_phi;
        let ys = sin_theta * sin_phi;
        let zs = cos_theta;

        Vector::new(xs, ys, zs)
    }

    /// Transform local hemisphere sample to world space using an ONB built from normal.
    /// Uses Frisvad's method - no normalize calls needed for the basis vectors.
    pub fn to_coord_space(&self, normal: Vector) -> Vector {
        let n = normal.v;
        let (t, b) = if n.z < -0.9999999 {
            // Handle singularity when normal points straight down
            (Vector3::new(0.0, -1.0, 0.0), Vector3::new(-1.0, 0.0, 0.0))
        } else {
            let a = 1.0 / (1.0 + n.z);
            let b_val = -n.x * n.y * a;
            (
                Vector3::new(1.0 - n.x * n.x * a, b_val, -n.x),
                Vector3::new(b_val, 1.0 - n.y * n.y * a, -n.y),
            )
        };
        Vector::new_from_na(t * self.v.x + b * self.v.y + n * self.v.z)
    }

    /// Samples uniformly on a unit sphere
    pub fn random_sphere() -> Vector {
        let xi1 = fastrand::f32();
        let xi2 = fastrand::f32();

        let theta = 2.0 * PI * xi1;
        let phi = f32::acos(1.0 - 2.0 * xi2);
        let xs = f32::sin(phi) * f32::cos(theta);
        let ys = f32::sin(phi) * f32::sin(theta);
        let zs = f32::cos(phi);
        Vector::new(xs, ys, zs)
    }

    #[inline(always)]
    pub fn norm(&self) -> f32 {
        self.v.norm()
    }

    #[inline(always)]
    pub fn normalized(&self) -> Vector {
        Vector::new_from_na(self.v.normalize())
    }

    #[inline(always)]
    pub fn dot(&self, other_vector: Vector) -> f32 {
        self.v.dot(&other_vector.v)
    }

    #[inline(always)]
    pub fn cross(&self, other_vector: Vector) -> Vector {
        Vector::new_from_na(self.v.cross(&other_vector.v))
    }

    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.v[0]
    }

    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.v[1]
    }

    #[inline(always)]
    pub fn z(&self) -> f32 {
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
    #[inline(always)]
    fn sub(self, other: Point) -> Self::Output {
        Vector::new_from_na(self.p - other.p)
    }
}

impl Add<Vector> for Point {
    type Output = Point;
    #[inline(always)]
    fn add(self, other: Vector) -> Self::Output {
        Point::new_from_na(self.p + other.v)
    }
}

impl Sub<Vector> for Point {
    type Output = Point;
    #[inline(always)]
    fn sub(self, other: Vector) -> Self::Output {
        Point::new_from_na(self.p - other.v)
    }
}

impl Add for Vector {
    type Output = Vector;
    #[inline(always)]
    fn add(self, other: Vector) -> Self::Output {
        Vector::new_from_na(self.v + other.v)
    }
}

impl Sub for Vector {
    type Output = Vector;
    #[inline(always)]
    fn sub(self, other: Vector) -> Self::Output {
        Vector::new_from_na(self.v - other.v)
    }
}

impl Mul<f32> for Vector {
    type Output = Vector;
    #[inline(always)]
    fn mul(self, other: f32) -> Vector {
        Vector::new_from_na(self.v * other)
    }
}
