use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;

use std::f32::consts::PI;

use super::super::common::{Spectrum, EPS};
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
    pub pdf: f32,
    pub wi: Vector,
    pub distance: f32,
}

pub enum Object {
    Triangle(Triangle),
    Sphere(Sphere),
}

impl Object {
    /// Returns distance if intersection occurs.
    #[inline(always)]
    pub fn intersect(&self, ray: &Ray) -> Option<f32> {
        match self {
            Object::Triangle(triangle) => {
                let (p1, p2, p3) = (triangle.p1, triangle.p2, triangle.p3);
                let direction = ray.direction;
                let e1 = p2 - p1;
                let e2 = p3 - p1;
                let s = ray.origin - p1;
                let s1 = direction.cross(e2);
                let s2 = s.cross(e1);
                let scalar = 1.0 / s1.dot(e1);
                let (t, b1, b2) = (
                    s2.dot(e2) * scalar,
                    s1.dot(s) * scalar,
                    s2.dot(direction) * scalar,
                );
                if b1 < 0.0 || b2 < 0.0 || b1 > 1.0 || b2 > 1.0 || b1 + b2 > 1.0 + EPS || t < EPS {
                    None
                } else {
                    Some(t)
                }
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

    #[inline(always)]
    pub fn surface_normal(&self, point: Point) -> Vector {
        match self {
            Object::Triangle(triangle) => triangle.surface_normal(point),
            Object::Sphere(sphere) => sphere.surface_normal(point),
        }
    }

    #[inline(always)]
    pub fn material(&self) -> &Material {
        match self {
            Object::Triangle(triangle) => &triangle.material,
            Object::Sphere(sphere) => &sphere.material,
        }
    }

    pub fn sample_l(&self, intersection_point: Point) -> LightSample {
        match self {
            Object::Triangle(_) => {
                unimplemented!()
            }
            Object::Sphere(sphere) => {
                let p = intersection_point;
                let s = sphere.random_point();
                let ps = s - p;
                let d_s = ps.norm();
                let wi = ps * (1.0 / d_s); // normalize without extra sqrt
                let d_c = (sphere.center - p).norm();
                let cos_a = (d_c * d_c + sphere.radius * sphere.radius - d_s * d_s)
                    / (2.0 * d_c * sphere.radius);
                let pdf = 2.0 * PI * (1.0 - cos_a);
                LightSample {
                    pdf,
                    wi,
                    distance: d_s,
                }
            }
        }
    }

    #[inline(always)]
    pub fn bsdf(&self, _wi: Vector, _wo: Vector) -> Spectrum {
        let material = self.material();
        match material.bsdf {
            BSDF::Diffuse => material.reflectance * (1.0 / PI),
            BSDF::Specular => Spectrum::black(),
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
                BSDFSample { wi, pdf, reflected }
            }
            BSDF::Specular => {
                let wi = wo - normal * 2.0 * wo.dot(normal);
                let pdf = 1.0;
                let cos_theta = f32::abs(wi.dot(normal));
                // undoing the cos theta multiplication in the raytracer
                let reflected = material.reflectance * (1.0 / cos_theta);
                BSDFSample { wi, pdf, reflected }
            }
        }
    }
}

impl Bounded for Object {
    fn aabb(&self) -> AABB {
        match self {
            Object::Triangle(triangle) => triangle.aabb(),
            Object::Sphere(sphere) => sphere.aabb(),
        }
    }
}

impl BHShape for Object {
    fn set_bh_node_index(&mut self, index: usize) {
        match self {
            Object::Triangle(triangle) => {
                triangle.node_index = index;
            }
            Object::Sphere(sphere) => sphere.node_index = index,
        }
    }

    fn bh_node_index(&self) -> usize {
        match self {
            Object::Triangle(triangle) => triangle.node_index,
            Object::Sphere(sphere) => sphere.node_index,
        }
    }
}

pub struct Sphere {
    center: Point,
    radius: f32,
    material: Material,
    node_index: usize,
}

pub struct Triangle {
    p1: Point,
    p2: Point,
    p3: Point,
    vn1: Vector,
    vn2: Vector,
    vn3: Vector,
    plane_normal_not_normalized: Vector,
    material: Material,
    node_index: usize,
}

pub struct BSDFSample {
    pub wi: Vector,
    pub pdf: f32,
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
    pub fn new(center: Point, radius: f32, material: Material) -> Sphere {
        Sphere {
            center,
            radius,
            material,
            node_index: 0,
        }
    }

    fn random_point(&self) -> Point {
        let random_vector = Vector::random_sphere();
        self.center + (random_vector * self.radius)
    }

    fn surface_normal(&self, point: Point) -> Vector {
        (point - self.center).normalized()
    }
}

struct BarycentricCoordinates {
    u: f32,
    v: f32,
    w: f32,
}

impl Triangle {
    pub fn new(
        p1: Point,
        p2: Point,
        p3: Point,
        vn1: Vector,
        vn2: Vector,
        vn3: Vector,
        material: Material,
    ) -> Triangle {
        let (vn1, vn2, vn3) = (vn1.normalized(), vn2.normalized(), vn3.normalized());
        let plane_normal_not_normalized = (p2 - p1).cross(p3 - p1);
        Triangle {
            p1,
            p2,
            p3,
            vn1,
            vn2,
            vn3,
            plane_normal_not_normalized,
            material,
            node_index: 0,
        }
    }

    pub fn new_without_vn(p1: Point, p2: Point, p3: Point, material: Material) -> Triangle {
        let normal = (p2 - p1).cross(p3 - p1);
        Triangle::new(p1, p2, p3, normal, normal, normal, material)
    }

    #[inline(always)]
    fn barycentric_coordinates(&self, p: Point) -> BarycentricCoordinates {
        let v0 = self.p2 - self.p1;
        let v1 = self.p3 - self.p1;
        let v2 = p - self.p1;
        let d00 = v0.dot(v0);
        let d01 = v0.dot(v1);
        let d11 = v1.dot(v1);
        let d20 = v2.dot(v0);
        let d21 = v2.dot(v1);
        let denom = d00 * d11 - d01 * d01;

        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;
        BarycentricCoordinates { u, v, w }
    }

    fn surface_normal(&self, point: Point) -> Vector {
        let b = self.barycentric_coordinates(point);
        self.vn1 * b.u + self.vn2 * b.v + self.vn3 * b.w
    }
}

impl Bounded for Sphere {
    fn aabb(&self) -> AABB {
        let half_size = Vector::new(self.radius, self.radius, self.radius);
        let min = self.center - half_size;
        let max = self.center + half_size;
        AABB::with_bounds(point_to_bvh_point(min), point_to_bvh_point(max))
    }
}

impl Bounded for Triangle {
    fn aabb(&self) -> AABB {
        let min_x = f32::min(f32::min(self.p1.x(), self.p2.x()), self.p3.x());
        let min_y = f32::min(f32::min(self.p1.y(), self.p2.y()), self.p3.y());
        let min_z = f32::min(f32::min(self.p1.z(), self.p2.z()), self.p3.z());
        let max_x = f32::max(f32::max(self.p1.x(), self.p2.x()), self.p3.x());
        let max_y = f32::max(f32::max(self.p1.y(), self.p2.y()), self.p3.y());
        let max_z = f32::max(f32::max(self.p1.z(), self.p2.z()), self.p3.z());

        let min = Point::new(min_x, min_y, min_z);
        let max = Point::new(max_x, max_y, max_z);

        AABB::with_bounds(point_to_bvh_point(min), point_to_bvh_point(max))
    }
}

#[inline(always)]
fn point_to_bvh_point(p: Point) -> bvh::nalgebra::Point3<f32> {
    bvh::nalgebra::Point3::new(p.x(), p.y(), p.z())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_triangle_surface_normal() {
        let material = Material::new(BSDF::Specular, Spectrum::white(), Spectrum::black());

        let triangle = Triangle::new(
            Point::new(-5.0, -5.0, -20.0),
            Point::new(5.0, -5.0, -20.0),
            Point::new(5.0, 5.0, -20.0),
            Vector::new_normalized(-0.4, 0.0, 1.0),
            Vector::new_normalized(0.4, 0.0, 1.0),
            Vector::new_normalized(0.0, 0.0, 1.0),
            material,
        );

        let ipoint = Point::new(4.173316, 3.258237, -20.0);
        triangle.surface_normal(ipoint);
    }
}
