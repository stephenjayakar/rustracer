extern crate sdl2;

mod geo;
mod objects;

pub use geo::{Point, Ray, Vector};
use objects::{Material, Object, Plane, Sphere, BSDF};

use crate::common::{Spectrum, EPS, GENERIC_ERROR};

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
