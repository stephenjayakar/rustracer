use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::BHShape;

use std::f64::consts::PI;

use super::super::common::Spectrum;
use super::{Point, Ray, Vector};

const PLANE_THICKNESS: f64 = 0.0001;
const PLANE_WIDTH: f64 = 200.0;

#[derive(Clone, Copy, Debug)]
pub enum BSDF {
    Diffuse,
}

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub bsdf: BSDF,
    pub reflectance: Spectrum,
    pub emittance: Spectrum,
}

pub struct LightSample {
	pub pdf: f64,
	pub wi: Vector,
}

pub trait Object {
	/// Returns distance if intersection occurs.
    fn intersect(&self, ray: &Ray) -> Option<f64>;
    fn surface_normal(&self, point: Point) -> Vector;
    fn material(&self) -> &Material;
	/// Returns a random point on the object.  Used for importance sampling.
	fn random_point(&self) -> Point;
	/// TODO: This should probably also return a vector... replace random point
	fn sample_l(&self, intersection_point: Point) -> LightSample;
}

pub struct Sphere {
    center: Point,
    radius: f64,
    material: Material,
	node_index: usize,
}

pub struct Plane {
    point: Point,
    normal: Vector,
    material: Material,
	node_index: usize,
}

impl Material {
    pub fn new(bsdf: BSDF, reflectance: Spectrum, emittance: Spectrum) -> Material {
        Material {
            bsdf,
            reflectance,
            emittance,
        }
    }

    pub fn bsdf(&self, wi: Vector, wo: Vector) -> Spectrum {
        match self.bsdf {
            BSDF::Diffuse => self.reflectance * (1.0 / PI),
        }
    }
}

impl Sphere {
    pub fn new(center: Point, radius: f64, material: Material) -> Sphere {
        Sphere {
            center,
            radius,
            material,
			node_index: 0,
        }
    }
}

impl Object for Sphere {
	/// Sphere intersection from bheisler.
    fn intersect(&self, ray: &Ray) -> Option<f64> {
        let l: Vector = self.center - ray.origin;
        let adj = l.dot(ray.direction);
        let d2 = l.dot(l) - (adj * adj);
        let radius2 = self.radius * self.radius;
        if d2 > radius2 {
            return None;
        }
        let thc = (radius2 - d2).sqrt();
        let t0 = adj - thc;
        let t1 = adj + thc;

        if t0 < 0.0 && t1 < 0.0 {
            return None;
        }

        let distance = if t0 < t1 { t0 } else { t1 };
        Some(distance)
    }

    fn surface_normal(&self, point: Point) -> Vector {
        (point - self.center).normalized()
    }

    fn material(&self) -> &Material {
        &self.material
    }

	fn random_point(&self) -> Point {
		let random_vector = Vector::random_sphere();
		let point = self.center.clone();
		point + (random_vector * self.radius)
	}

	fn sample_l(&self, intersection_point: Point) -> LightSample {
		let p = intersection_point;
		let s = self.random_point();
		let ps = s - p;
		let wi = ps.normalized();
		let d_c = (self.center - p).norm();
		let d_s = ps.norm();
		let cos_a = (d_c.powi(2) + self.radius.powi(2) - d_s.powi(2)) / (2.0 * d_c * self.radius);
		let pdf = 2.0 * PI * (1.0 - cos_a);
		LightSample {
			pdf,
			wi,
		}
	}
}

impl Plane {
    pub fn new(point: Point, normal: Vector, material: Material) -> Plane {
        Plane {
            point,
            normal,
            material,
			node_index: 0,
        }
    }
}

impl Object for Plane {
    fn intersect(&self, ray: &Ray) -> Option<f64> {
        let d = (self.point - ray.origin).dot(self.normal) / ray.direction.dot(self.normal);
        if d > 0.0 {
            Some(d)
        } else {
            None
        }
    }

    fn surface_normal(&self, _: Point) -> Vector {
        self.normal
    }

    fn material(&self) -> &Material {
        &self.material
    }

	fn random_point(&self) -> Point {
		unimplemented!()
	}

	fn sample_l(&self, intersection_point: Point) -> LightSample {
		unimplemented!()
	}
}

impl Bounded for Sphere {
    fn aabb(&self) -> AABB {
        let half_size = Vector::new(self.radius, self.radius, self.radius);
        let min = self.center - half_size;
        let max = self.center + half_size;

		// RIP IN PEACE
		let min = point_to_bvh_point(min);
		let max = point_to_bvh_point(max);
        AABB::with_bounds(min, max)
    }
}

impl BHShape for Sphere {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

impl Bounded for Plane {
    fn aabb(&self) -> AABB {
		let lower_point = self.point - (self.normal * PLANE_THICKNESS);
		
		unimplemented!();
    }
}

impl BHShape for Plane {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

fn point_to_bvh_point(p: Point) -> bvh::nalgebra::Point3<f32> {
	bvh::nalgebra::Point3::new(p.x() as f32, p.y() as f32, p.z() as f32)
}
