extern crate sdl2;

mod geo;
mod objects;

pub use geo::{Point, Ray, Vector};
use objects::{Material, Object, Plane, Sphere, BSDF};

use crate::common::{Spectrum, GENERIC_ERROR, EPS};

pub struct Scene {
	objects: Vec<Box<dyn Object>>,
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
		let mut objects = Vec::<Box<dyn Object>>::new();
		for plane in planes {
			objects.push(Box::new(plane));
		}
		for sphere in spheres {
			objects.push(Box::new(sphere));
		}
		Scene { objects }
    }

	/// Creates a Cornell box of sorts
    pub fn new_preset() -> Scene {
		let half_length = 20.0;
		let box_z_offset = -50.0;
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
		let light_radius = 7.0;
		let sphere_radius = 6.0;
        let spheres = vec![
			Sphere::new(Point::new(-half_length / 3.0,
								   -half_length + sphere_radius,
								   box_z_offset - 2.0 * half_length / 3.0),
						sphere_radius,
						grey_diffuse_material),
			Sphere::new(Point::new(half_length / 3.0,
								   -half_length + sphere_radius,
								   box_z_offset - half_length / 3.0),
						sphere_radius,
						red_diffuse_material),
			// insert the light at the top of the scene, halfway through the plane
            Sphere::new(Point::new(0.0, half_length + light_radius * 0.6, box_z_offset - half_length / 2.0), light_radius, white_light_material),
        ];

        let planes = vec![
			// bottom wall
			Plane::new(
				Point::new(0.0, -half_length, 0.0),
				Vector::new(0.0, 1.0, 0.0),
				grey_diffuse_material,
			),
			// left wall
			Plane::new(
				Point::new(-half_length, 0.0, 0.0),
				Vector::new(1.0, 0.0, 0.0),
				red_diffuse_material,
			),
			// right wall
			Plane::new(
				Point::new(half_length, 0.0, 0.0),
				Vector::new(-1.0, 0.0, 0.0),
				blue_diffuse_material,
			),
			// back wall is only half length depth
			Plane::new(
				Point::new(0.0, 0.0, box_z_offset - half_length),
				Vector::new(0.0, 0.0, 1.0),
				green_diffuse_material,
			),
			// top wall
			Plane::new(
				Point::new(0.0, half_length, 0.0),
				Vector::new(0.0, -1.0, 0.0),
				grey_diffuse_material,
			),
		];
        Scene::new(planes, spheres)
    }

    fn num_objects(&self) -> usize {
		self.objects.len()
    }

	/// Intersects the scene with the given ray and takes ownership of it,
	/// in order to populate it in the intersection object without copying.
    pub fn intersect(&self, ray: Ray) -> Option<RayIntersection> {
        let mut min_dist = f64::INFINITY;
        let mut min_object = None;
        for object in &self.objects {
            if let Some(d) = object.intersect(&ray) {
                if d < min_dist {
                    min_dist = d;
                    min_object = Some(object);
                }
            }
        }
        match min_object {
            Some(object) => Some(RayIntersection {
                object: object.as_ref(),
				ray,
                distance: min_dist,
            }),
            None => None,
        }
    }

	pub fn lights(&self) -> Vec<&dyn Object> {
		let mut lights = Vec::<&dyn Object>::new();
		for object in &self.objects {
			if !object.material().emittance.is_black() {
				lights.push(object.as_ref());
			}
		}
		lights
	}
}
