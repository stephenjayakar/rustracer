use super::super::common::Spectrum;
use crate::{Point, Ray, Vector};

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

pub trait Object {
	/// Returns distance if intersection occurs.
    fn intersect(&self, ray: &Ray) -> Option<f64>;
    fn surface_normal(&self, point: Point) -> Vector;
    fn material(&self) -> &Material;
}

pub struct Sphere {
    center: Point,
    radius: f64,
    material: Material,
}

pub struct Plane {
    point: Point,
    normal: Vector,
    material: Material,
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
            BSDF::Diffuse => self.reflectance * (1.0 / std::f64::consts::PI),
        }
    }
}

impl Sphere {
    pub fn new(center: Point, radius: f64, material: Material) -> Sphere {
        Sphere {
            center,
            radius,
            material,
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
}

impl Plane {
    pub fn new(point: Point, normal: Vector, material: Material) -> Plane {
        Plane {
            point,
            normal,
            material,
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
}
