extern crate sdl2;

use bvh::bvh::BVH;

mod geo;
mod objects;

pub use geo::{Point, Ray, Vector};
use objects::{Material, Object, Plane, Sphere, BSDF};

use crate::common::{Spectrum, GENERIC_ERROR, EPS};

pub struct Scene {
    planes: Vec<Plane>,
    spheres: Vec<Sphere>,
	sphere_bvh: BVH,
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
    fn new(planes: Vec<Plane>, mut spheres: Vec<Sphere>) -> Scene {
		let sphere_bvh = BVH::build(&mut spheres);
		// build plane bvh
		Scene { planes, spheres, sphere_bvh }
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
        let mut min_object: Option<&dyn Object> = None;
		// test sphere intersections
		// yikes two
		let hit_sphere_aabbs = self.sphere_bvh.traverse(&ray_to_bvh_ray(&ray), &self.spheres);
		for sphere in hit_sphere_aabbs {
            if let Some(d) = sphere.intersect(&ray) {
                if d < min_dist {
                    min_dist = d;
                    min_object = Some(sphere);
                }
            }
		}
		for plane in &self.planes {
            if let Some(d) = plane.intersect(&ray) {
                if d < min_dist {
                    min_dist = d;
                    min_object = Some(plane);
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

pub fn ray_to_bvh_ray(ray: &Ray) -> bvh::ray::Ray {
	let origin = bvh::nalgebra::Point3::new(ray.origin.x() as f32, ray.origin.y() as f32, ray.origin.z() as f32);
	let direction = bvh::nalgebra::Vector3::new(ray.direction.x() as f32, ray.direction.y() as f32, ray.direction.z() as f32);
	bvh::ray::Ray::new(origin, direction)
}
