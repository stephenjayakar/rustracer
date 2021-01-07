use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;

use std::f64::consts::PI;

use super::super::common::{Spectrum, EPS};
use super::{Point, Ray, Vector};

#[derive(Clone, Copy, Debug)]
pub enum BSDF {
    Diffuse,
    Specular,
    Glass(f64),
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
            Object::Triangle(triangle) => triangle.intersect(ray),
            Object::Sphere(sphere) => sphere.intersect(ray),
        }
    }

    pub fn surface_normal(&self, point: Point) -> Vector {
        match self {
            Object::Triangle(triangle) => triangle.surface_normal(),
            Object::Sphere(sphere) => sphere.surface_normal(point),
        }
    }

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
                let wi = ps.normalized();
                let d_c = (sphere.center - p).norm();
                let d_s = ps.norm();
                let cos_a = (d_c.powi(2) + sphere.radius.powi(2) - d_s.powi(2))
                    / (2.0 * d_c * sphere.radius);
                let pdf = 2.0 * PI * (1.0 - cos_a);
                LightSample { pdf, wi }
            }
        }
    }

    pub fn bsdf(&self, wi: Vector, wo: Vector) -> Spectrum {
        let material = self.material();
        match material.bsdf {
            BSDF::Diffuse => material.reflectance * (1.0 / PI),
            BSDF::Specular => Spectrum::black(),
            BSDF::Glass(_) => Spectrum::black(),
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
                // a reflection is a rotation 180deg around the z axis,
                // and then you flip the direction.
                let wi = reflect(wo, normal);
                let pdf = 1.0;
                let cos_theta = f64::abs(wi.dot(normal));
                // undoing the cos theta multiplication in the raytracer
                let reflected = material.reflectance * (1.0 / cos_theta);
                BSDFSample { wi, pdf, reflected }
            }
            BSDF::Glass(eta) => {
                match refract(wo, normal, eta) {
                    // Total internal reflection
                    None => {
                        // TODO: Refactor this so I don't copy and paste reflect code.
                        let wi = reflect(wo, normal);
                        let pdf = 1.0;
                        let inv_cos_theta = 1.0 / f64::abs(wi.dot(normal));

                        // undoing the cos theta multiplication in the raytracer
                        let reflected = material.reflectance * (inv_cos_theta);
                        BSDFSample { wi, pdf, reflected }
                    }
                    // Refraction
                    Some(refraction) => {
                        let (wi, R, eta) = (refraction.wi, refraction.R, refraction.eta);
                        let rand = fastrand::f64();

                        if rand > R {
                            // reflect
                            let wi = reflect(wo, normal);
                            let pdf = R;
                            let inv_cos_theta = 1.0 / f64::abs(wi.dot(normal));

                            // undoing the cos theta multiplication in the raytracer
                            let reflected = material.reflectance * (inv_cos_theta);
                            BSDFSample { wi, pdf, reflected }
                        } else {
                            // refract
                            let pdf = 1.0 - R;
                            let inv_cos_theta = 1.0 / f64::abs(wi.dot(normal));
                            let reflected =
                                material.reflectance * (inv_cos_theta) * pdf * f64::powi(eta, 2);
                            BSDFSample { wi, pdf, reflected }
                        }
                    }
                }
            }
        }
    }
}

/// Scratchapixel
fn reflect(v: Vector, n: Vector) -> Vector {
    v - n * 2.0 * v.dot(n)
}

/// Convenience struct, as refract is aware
struct Refraction {
    wi: Vector,
    R: f64,
    eta: f64,
}

/// Scratchapixel
fn refract(v: Vector, normal: Vector, eta: f64) -> Option<Refraction> {
    let mut n = normal;
    let n_dot_v = n.dot(v);
    let mut cosv = n_dot_v.clamp(-1.0, 1.0);
    let mut etav = 1.0;
    let mut etat = eta;
    if cosv < 0.0 {
        cosv = -cosv;
    } else {
        n = -n;
        let temp = etav;
        etav = etat;
        etat = temp;
    }
    let eta = etav / etat;
    let k = 1.0 - eta * eta * (1.0 - cosv * cosv);
    if k < 0.0 {
        None
    } else {
        let R = f64::powi((etav - etat) / (etav + etat), 2);
        let wi = v * eta + n * (eta * cosv - f64::sqrt(k));
        Some(Refraction { wi, R, eta })
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

#[derive(Debug)]
pub struct Sphere {
    center: Point,
    radius: f64,
    material: Material,
    node_index: usize,
}

#[derive(Debug)]
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

    fn surface_normal(&self, point: Point) -> Vector {
	(point - self.center).normalized()
    }

    pub fn intersect(&self, ray: &Ray) -> Option<f64> {
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

        let mut t = t0;
        if t > t1 {
            t = t1;
        }
        if t < 0.0 + EPS {
            t = t1;
            if t < 0.0 {
                return None;
            }
        }
	// if t < 0.1 {
	//     println!("{:#?} {:#?} {} {} ldotl: {}", self, ray, t0, t1, l.dot(l));
	// }
        Some(t)
    }
}

impl Triangle {
    pub fn new(p1: Point, p2: Point, p3: Point, material: Material) -> Triangle {
        let normal = ((p2 - p1).cross(p3 - p1)).normalized();
        Triangle {
            p1,
            p2,
            p3,
            normal,
            material,
            node_index: 0,
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<f64> {
        let e1 = self.p2 - self.p1;
        let e2 = self.p3 - self.p1;
        let h = ray.direction.cross(e2);
        let a = e1.dot(h);
        if f64::abs(a) < EPS {
            return None;
        }
        let f = 1.0 / a;
        let s = ray.origin - self.p1;
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 {
            return None;
        }
        let q = s.cross(e1);
        let v = f * ray.direction.dot(q);
        if v < 0.0 || u > 1.0 {
            return None;
        }
        let t = f * e2.dot(q);
        if t > EPS {
            Some(t)
        } else {
            None
        }
    }

    fn surface_normal(&self) -> Vector { self.normal }
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
        let min_x = f64::min(f64::min(self.p1.x(), self.p2.x()), self.p3.x());
        let min_y = f64::min(f64::min(self.p1.y(), self.p2.y()), self.p3.y());
        let min_z = f64::min(f64::min(self.p1.z(), self.p2.z()), self.p3.z());
        let max_x = f64::max(f64::max(self.p1.x(), self.p2.x()), self.p3.x());
        let max_y = f64::max(f64::max(self.p1.y(), self.p2.y()), self.p3.y());
        let max_z = f64::max(f64::max(self.p1.z(), self.p2.z()), self.p3.z());

        let min = Point::new(min_x, min_y, min_z);
        let max = Point::new(max_x, max_y, max_z);

        AABB::with_bounds(point_to_bvh_point(min), point_to_bvh_point(max))
    }
}

fn point_to_bvh_point(p: Point) -> bvh::nalgebra::Point3<f32> {
    bvh::nalgebra::Point3::new(p.x() as f32, p.y() as f32, p.z() as f32)
}

struct ObjectTestSetupVariables {
    material: Material,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> ObjectTestSetupVariables {
        ObjectTestSetupVariables {
            material: Material::new(BSDF::Diffuse, Spectrum::black(), Spectrum::black()),
        }
    }

    /// Sphere intersection should only happen with the outside of the sphere.  Therefore, if we start a ray
    /// from within the sphere and intersection, the intersection point should be close to the surface.
    #[test]
    fn test_within_sphere() {
        let material = setup().material;
        let center = Point::origin();
        let radius = 2.0;
        let sphere = Sphere::new(center, radius, material);

        let p = Point::new(1.0, 1.0, 1.0);
        let v = Vector::new(1.0, 1.0, 1.0).normalized();
        let ray = Ray::new(p, v);
        let distance = sphere
            .intersect(&ray)
            .expect("The ray should intersect the sphere");
        assert_eq!(distance, 2.0 - f64::sqrt(3.0));
    }
}
