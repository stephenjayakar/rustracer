use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::BHShape;

use std::f64::consts::PI;

use super::super::common::{EPS, Spectrum};
use super::{Point, Ray, Vector};

#[derive(Clone, Copy, Debug)]
pub enum BSDF {
    Diffuse,
	Specular,
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

pub enum Object {
	Triangle(Triangle),
	Sphere(Sphere),
}

impl Object {
	/// Returns distance if intersection occurs.
    pub fn intersect(&self, ray: &Ray) -> Option<f64> {
		match self {
			/// Scratchapixel's triangle intersection algorithm.
		    Object::Triangle(triangle) => {
				let v0 = Vector::new(triangle.p1.x(), triangle.p1.y(), triangle.p1.z());
				let v1 = Vector::new(triangle.p2.x(), triangle.p2.y(), triangle.p2.z());
				let v2 = Vector::new(triangle.p3.x(), triangle.p3.y(), triangle.p3.z());
				let orig = Vector::new(ray.origin.x(), ray.origin.y(), ray.origin.z());
				let n = triangle.normal;
				let n_dot_ray = n.dot(ray.direction);

				if f64::abs(n_dot_ray) < EPS { return None }

				let d = n.dot(v0);

				let t = n.dot(orig) + d / n_dot_ray;
				if t < 0.0 { return None }

				let p = orig + ray.direction * t;

				let edge0 = v1 - v0;
				let vp0 = p - v0;
				let c = edge0.cross(vp0);
				if n.dot(c) < 0.0 { return None }

				let edge1 = v2 - v1;
				let vp1 = p - v1;
				let c = edge1.cross(vp1);
				if n.dot(c) < 0.0 { return None }

				let edge2 = v0 - v2;
				let vp2 = p - v2;
				let c = edge2.cross(vp2);
				if n.dot(c) < 0.0 { return None }

				Some(t)
			}
		    Object::Sphere(sphere) => {
				let l: Vector = sphere.center - ray.origin;
				let adj = l.dot(ray.direction);
				let d2 = l.dot(l) - (adj * adj);
				let radius2 = sphere.radius * sphere.radius;
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
		}
	}

    pub fn surface_normal(&self, point: Point) -> Vector {
		match self {
			Object::Triangle(triangle) => {
				triangle.normal
			},
			Object::Sphere(sphere) => {
				(point - sphere.center).normalized()
			}
		}
	}

    pub fn material(&self) -> &Material {
		match self {
			Object::Triangle(triangle) => {
				&triangle.material
			},
			Object::Sphere(sphere) => {
				&sphere.material
			}
		}
	}

	pub fn sample_l(&self, intersection_point: Point) -> LightSample {
		match self {
			Object::Triangle(_) => { unimplemented!() },
			Object::Sphere(sphere) => {
				let p = intersection_point;
				let s = sphere.random_point();
				let ps = s - p;
				let wi = ps.normalized();
				let d_c = (sphere.center - p).norm();
				let d_s = ps.norm();
				let cos_a = (d_c.powi(2) + sphere.radius.powi(2) - d_s.powi(2)) / (2.0 * d_c * sphere.radius);
				let pdf = 2.0 * PI * (1.0 - cos_a);
				LightSample {
					pdf,
					wi,
				}
			}
		}
	}

	pub fn bsdf(&self, wi: Vector, wo: Vector) -> Spectrum {
		let material = self.material();
		match material.bsdf {
            BSDF::Diffuse => material.reflectance * (1.0 / PI),
			BSDF::Specular => { Spectrum::black() }
        }

	}

	/// Use instead of bsdf when you want to bounce the vector.
	pub fn sample_bsdf(&self, wo: Vector, normal: Vector) -> BSDFSample {
		let material = self.material();
		match material.bsdf {
			BSDF::Diffuse => {
				let wi = Vector::random_hemisphere().to_coord_space(normal);
				let pdf = 2.0 * PI;
				let reflected = self.bsdf(wi, wo);
				BSDFSample {
					wi,
					pdf,
					reflected,
				}
			},
			BSDF::Specular => {
				// a reflection is a rotation 180deg around the z axis,
				// and then you flip the direction.
				let wi = wo - normal * 2.0 * wo.dot(normal);
				let pdf = 1.0;
				let cos_theta = f64::abs(wi.dot(normal));
				// undoing the cos theta multiplication in the raytracer
				let reflected = material.reflectance * (1.0 / cos_theta);
				BSDFSample {
					wi,
					pdf,
					reflected,
				}
			},
		}
	}
}

impl Bounded for Object {
	fn aabb(&self) -> AABB {
		match self {
			Object::Triangle(triangle) => {
				triangle.aabb()
			},
			Object::Sphere(sphere) => {
				sphere.aabb()
			}
		}
	}
}

impl BHShape for Object {
    fn set_bh_node_index(&mut self, index: usize) {
		match self {
			Object::Triangle(triangle) => {
				triangle.node_index = index;
			},
			Object::Sphere(sphere) => {
				sphere.node_index = index
			}
		}
    }

    fn bh_node_index(&self) -> usize {
		match self {
			Object::Triangle(triangle) => {
				triangle.node_index
			},
			Object::Sphere(sphere) => {
				sphere.node_index
			}
		}
    }
}

pub struct Sphere {
    center: Point,
    radius: f64,
    material: Material,
	node_index: usize,
}

pub struct Triangle {
	p1: Point,
	p2: Point,
	p3: Point,
	normal: Vector,
    material: Material,
	node_index: usize,
}

pub struct BSDFSample {
	pub wi: Vector,
	pub pdf: f64,
	pub reflected: Spectrum,
}

impl Material {
    pub fn new(bsdf: BSDF, reflectance: Spectrum, emittance: Spectrum) -> Material {
        Material {
            bsdf,
            reflectance,
            emittance,
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

	fn random_point(&self) -> Point {
		let random_vector = Vector::random_sphere();
		let point = self.center.clone();
		point + (random_vector * self.radius)
	}
}

impl Triangle {
	pub fn new(p1: Point, p2: Point, p3: Point, material: Material) -> Triangle {
		let normal = ((p2 - p1).cross(p3 - p1)).normalized();
		Triangle {
			p1, p2, p3,
			normal,
			material,
			node_index: 0,
		}
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

impl Bounded for Triangle {
    fn aabb(&self) -> AABB {
		let min_x = f64::min(f64::min(
			self.p1.x(), self.p2.x()), self.p3.x());
		let min_y = f64::min(f64::min(
			self.p1.y(), self.p2.y()), self.p3.y());
		let min_z = f64::min(f64::min(
			self.p1.z(), self.p2.z()), self.p3.z());
		let max_x = f64::max(f64::max(
			self.p1.x(), self.p2.x()), self.p3.x());
		let max_y = f64::max(f64::max(
			self.p1.y(), self.p2.y()), self.p3.y());
		let max_z = f64::max(f64::max(
			self.p1.z(), self.p2.z()), self.p3.z());

		let min = Point::new(min_x, min_y, min_z);
		let max = Point::new(max_x, max_y, max_z);

		AABB::with_bounds(
			point_to_bvh_point(min), point_to_bvh_point(max),
		)
    }
}

fn point_to_bvh_point(p: Point) -> bvh::nalgebra::Point3<f32> {
	bvh::nalgebra::Point3::new(p.x() as f32, p.y() as f32, p.z() as f32)
}
