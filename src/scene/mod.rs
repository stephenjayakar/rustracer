extern crate sdl2;

mod geo;
mod objects;

pub use geo::{Point, Ray, Vector};
use objects::{Material, Object, Plane, Sphere, BSDF};

use crate::common::{Spectrum, GENERIC_ERROR, EPS};

pub struct Scene {
    planes: Vec<Plane>,
    spheres: Vec<Sphere>,
	// TODO: Figure out how to cache this in a thread safe way
	// lights: Vec<&'a dyn Object>,
}

pub struct RayIntersection<'a> {
    distance: f64,
    object: &'a dyn Object,
	ray: Ray,
}

impl<'a> RayIntersection<'a> {
	pub fn distance(&self) -> f64 {
		self.distance
	}

	pub fn object(&self) -> &'a dyn Object {
		self.object
	}

	pub fn ray(&self) -> &Ray {
		&self.ray
	}

	pub fn point(&self) -> Point {
        let min_dist = self.distance();
		let ray = self.ray();
        let scaled_vector = ray.direction * min_dist;
        let intersection_point = ray.origin + scaled_vector;
        // bumping the point a little out of the object to prevent self-collision
        let surface_normal: Vector = self.object.surface_normal(intersection_point);
        intersection_point + (surface_normal * EPS)
	}
}

impl Scene {
    fn new(planes: Vec<Plane>, spheres: Vec<Sphere>) -> Scene {
		Scene { planes, spheres }
    }

    pub fn new_preset() -> Scene {
        let red_diffuse_material = Material::new(
            BSDF::Diffuse,
            Spectrum::red(),
            Spectrum::black(),
        );
        let blue_diffuse_material = Material::new(
            BSDF::Diffuse,
            Spectrum::blue(),
            Spectrum::black(),
        );
        let green_diffuse_material = Material::new(
            BSDF::Diffuse,
            Spectrum::green(),
            Spectrum::black(),
        );
        let grey_diffuse_material = Material::new(
            BSDF::Diffuse,
            Spectrum::grey(),
            Spectrum::black(),
        );
        let white_light_material = Material::new(
            BSDF::Diffuse,
            Spectrum::black(),
            Spectrum::white(),
        );
        let spheres = vec![
            Sphere::new(Point::new(0.0, 0.0, -20.0), 2.0, red_diffuse_material),
            Sphere::new(Point::new(5.0, 0.0, -20.0), 2.0, red_diffuse_material),
            Sphere::new(Point::new(-5.0, 0.0, -20.0), 2.0, grey_diffuse_material),
            Sphere::new(Point::new(10.0, 0.0, -20.0), 2.0, red_diffuse_material),
            Sphere::new(Point::new(4.0, 4.0, -18.0), 1.5, white_light_material),
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
        Scene::new(planes, spheres)
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

	/// Intersects the scene with the given ray and takes ownership of it,
	/// in order to populate it in the intersection object without copying.
    pub fn intersect(&self, ray: Ray) -> Option<RayIntersection> {
        let mut min_dist = f64::INFINITY;
        let mut min_object = None;
        for i in 0..self.num_objects() {
            let object = self.get_object_by_index(i);
            if let Some(d) = object.intersect(&ray) {
                if d < min_dist {
                    min_dist = d;
                    min_object = Some(object);
                }
            }
        }
        match min_object {
            Some(object) => Some(RayIntersection {
                object,
				ray,
                distance: min_dist,
            }),
            None => None,
        }
    }

	pub fn lights(&self) -> Vec<&dyn Object> {
		let mut lights = Vec::<&dyn Object>::new();
		for plane in &self.planes {
			if !plane.material().emittance.is_black() {
				lights.push(plane);
			}
		}
		for sphere in &self.spheres {
			if !sphere.material().emittance.is_black() {
				lights.push(sphere);
			}
		}
		lights
	}
}
