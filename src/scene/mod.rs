use std::ops::{Add, Sub};

use crate::common::{Spectrum, EPS, GENERIC_ERROR};

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
    pub bounces_left: u32,
}

impl Ray {
    // normalizes the direction vector
    pub fn new(origin: Point, direction: Vector, bounces_left: u32) -> Ray {
        Ray {
            origin,
            direction: direction.normalized(),
            bounces_left,
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
    fn intersect(&self, ray: &Ray) -> Option<f64>;
    fn surface_normal(&self, point: Point) -> Vector;
    fn material(&self) -> &Material;
}

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
    material: Material,
}

pub struct Plane {
    pub point: Point,
    pub normal: Vector,
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

    pub fn sample_bsdf(&self, wi: Vector, wo: Vector) -> Spectrum {
        match self.bsdf {
            BSDF::Diffuse => self.reflectance * (1.0 / std::f64::consts::PI),
        }
    }

    pub fn bounce(&self, wi: Vector) -> Vector {
        match self.bsdf {
            BSDF::Diffuse => {
                // return random vector
            }
        }
        unimplemented!();
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
    // sphere intersection from bheisler
    // returns intersection distance
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

    fn surface_normal(&self, _point: Point) -> Vector {
        self.normal
    }

    fn material(&self) -> &Material {
        &self.material
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn sphere_intersection() {
//         let origin = Point::new(0.0, 0.0, 0.0);
//         let direction = Vector::new_normalized(0.0, 0.0, -1.0);
//         let ray = Ray::new(&origin, &direction);
//         let sphere = Sphere::new(Point::new(0.0, 0.0, -4.0), 2.0);
//         assert!(sphere.intersect(&ray) != None);

//         let vector_that_misses = Vector::new_normalized(3.0, 0.0, -4.0);
//         let ray_that_misses = Ray::new(&origin, &vector_that_misses);
//         assert!(sphere.intersect(&ray_that_misses) == None);
//     }
// }

pub struct Scene {
    planes: Vec<Plane>,
    spheres: Vec<Sphere>,
}

pub struct RayIntersection<'a> {
    distance: f64,
    object: &'a dyn Object,
}

impl Scene {
    // pub fn new<'a>(objects: Vector<&'a dyn Object>) -> Scene {
    // 	unimplemented!();
    // }

    pub fn new_preset() -> Scene {
        let red_diffuse_material = Material::new(
            BSDF::Diffuse,
            Spectrum::new(100, 0, 0),
            Spectrum::new(0, 0, 0),
        );
        let grey_diffuse_material = Material::new(
            BSDF::Diffuse,
            Spectrum::new(50, 50, 50),
            Spectrum::new(0, 0, 0),
        );
        let blue_light_material = Material::new(
            BSDF::Diffuse,
            Spectrum::new(0, 0, 100),
            Spectrum::new(0, 0, 100),
        );
        let spheres = vec![
            Sphere::new(Point::new(0.0, 0.0, -14.0), 2.0, red_diffuse_material),
            Sphere::new(Point::new(5.0, 0.0, -14.0), 2.0, red_diffuse_material),
            Sphere::new(Point::new(-5.0, 0.0, -14.0), 2.0, red_diffuse_material),
            Sphere::new(Point::new(10.0, 0.0, -14.0), 2.0, red_diffuse_material),
            Sphere::new(Point::new(5.0, 10.0, -14.0), 2.0, blue_light_material),
        ];
        let mut planes = Vec::new();
        let plane = Plane::new(
            Point::new(0.0, 10.0, 0.0),
            Vector::new(0.0, -1.0, 0.0),
            grey_diffuse_material,
        );
        planes.push(plane);

        Scene { planes, spheres }
    }

    // allows indexing across multiple object data structures
    fn get_object_by_index(&self, index: usize) -> &dyn Object {
        let planes_len = self.planes.len();
        let spheres_len = self.spheres.len();

        if index < planes_len {
            self.planes.get(index).expect(GENERIC_ERROR)
        } else if index < planes_len + spheres_len {
            self.spheres.get(index - planes_len).expect(GENERIC_ERROR)
        } else {
            panic!("index out of range of scene");
        }
    }

    fn num_objects(&self) -> usize {
        self.planes.len() + self.spheres.len()
    }

    fn object_intersection(&self, ray: &Ray) -> Option<RayIntersection> {
        let mut min_dist = f64::INFINITY;
        let mut min_object = None;
        for i in 0..self.num_objects() {
            let object = self.get_object_by_index(i);
            if let Some(d) = object.intersect(ray) {
                if d < min_dist {
                    min_dist = d;
                    min_object = Some(object);
                }
            }
        }
        match min_object {
            Some(object) => Some(RayIntersection {
                object,
                distance: min_dist,
            }),
            None => None,
        }
    }

    // for a ray, estimate global illumination
    pub fn cast_ray(&self, ray: &Ray) -> Spectrum {
        match self.object_intersection(ray) {
            Some(ray_intersection) => {
                let object = ray_intersection.object;
                let min_dist = ray_intersection.distance;
                let mut intersection_point: Point = ray.get_intersection_point(min_dist);
                // bumping the point a little out of the object to prevent self-collision
                let surface_normal: Vector = object.surface_normal(intersection_point);
                intersection_point = intersection_point + surface_normal.scale(EPS);

                let bounces_left = ray.bounces_left;
                let emittance = object.material().emittance;
                match bounces_left {
                    0 => {
                        // zero bounce radiance
                        emittance
                    }
                    1 => {
                        // direct lighting
                        unimplemented!()
                    }
                    _ => {
                        // global illumination
                        unimplemented!()
                    }
                }
                // for point_light in self.lights.iter() {
                //     let light_direction = (point_light.position - intersection_point).normalized();
                //     let light_ray = Ray::new(intersection_point, light_direction);

                //     if self.object_intersection(&light_ray).is_none() {
                // 	// lambertian code
                // 	let intensity = f64::abs(light_direction.dot(surface_normal));
                // 	let color_value = (intensity * 255.0) as u8;
                // 	color += Spectrum::new(color_value, color_value, color_value);
                //     }
                // }
            }
            None => Spectrum::new(0, 0, 0),
        }
    }
}
