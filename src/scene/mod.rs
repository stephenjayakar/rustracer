extern crate sdl2;

mod geo;
mod objects;

pub use geo::{Point, Ray, Vector};
use objects::{Material, Object, Plane, Sphere, BSDF};

use crate::common::{Spectrum, GENERIC_ERROR};

pub struct Scene {
    planes: Vec<Plane>,
    spheres: Vec<Sphere>,
}

pub struct RayIntersection<'a> {
    pub distance: f64,
    pub object: &'a dyn Object,
}

impl Scene {
    // pub fn new<'a>(objects: Vector<&'a dyn Object>) -> Scene {
    // 	unimplemented!();
    // }

    pub fn new_preset() -> Scene {
        let red_diffuse_material = Material::new(
            BSDF::Diffuse,
            Spectrum::new(255, 0, 0),
            Spectrum::new(0, 0, 0),
        );
        let blue_diffuse_material = Material::new(
            BSDF::Diffuse,
            Spectrum::new(0, 0, 255),
            Spectrum::new(0, 0, 0),
        );
        let green_diffuse_material = Material::new(
            BSDF::Diffuse,
            Spectrum::new(0, 255, 0),
            Spectrum::new(0, 0, 0),
        );
        let grey_diffuse_material = Material::new(
            BSDF::Diffuse,
            Spectrum::new(200, 200, 200),
            Spectrum::new(0, 0, 0),
        );
        let reddish_white_light_material = Material::new(
            BSDF::Diffuse,
            Spectrum::new(0, 0, 100),
            Spectrum::new(200, 200, 255),
        );
        let spheres = vec![
            Sphere::new(Point::new(0.0, 0.0, -20.0), 2.0, red_diffuse_material),
            Sphere::new(Point::new(5.0, 0.0, -20.0), 2.0, red_diffuse_material),
            Sphere::new(Point::new(-5.0, 0.0, -20.0), 2.0, grey_diffuse_material),
            Sphere::new(Point::new(10.0, 0.0, -20.0), 2.0, red_diffuse_material),
            Sphere::new(Point::new(4.0, 4.0, -18.0), 1.5, reddish_white_light_material),
        ];
        let planes = vec![
			// bottom wall
			Plane::new(
				Point::new(0.0, -5.0, 0.0),
				Vector::new(0.0, 1.0, 0.0),
				grey_diffuse_material,
			),
			// left wall
			Plane::new(
				Point::new(-14.0, 0.0, 0.0),
				Vector::new(1.0, 0.0, 0.0),
				red_diffuse_material,
			),
			// right wall
			Plane::new(
				Point::new(14.0, 0.0, 0.0),
				Vector::new(-1.0, 0.0, 0.0),
				blue_diffuse_material,
			),
			// back wall
			Plane::new(
				Point::new(0.0, 0.0, -30.0),
				Vector::new(0.0, 0.0, 1.0),
				green_diffuse_material,
			),
			// top wall
			Plane::new(
				Point::new(0.0, 9.0, 0.0),
				Vector::new(0.0, -1.0, 0.0),
				grey_diffuse_material,
			),
		];
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

    pub fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
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
}
