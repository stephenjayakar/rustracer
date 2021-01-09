use bvh::bvh::BVH;

mod geo;
mod objects;

pub use geo::{Point, Ray, Vector};
use objects::{Material, Object, Triangle, Sphere, BSDF};

use crate::common::{EPS, Spectrum};

/// The Scene is static. Please don't change it unless you update the acceleration structures!
pub struct Scene {
	objects: Vec<Object>,
	bvh: BVH,
	light_indexes: Vec<usize>,
}

pub struct RayIntersection<'a> {
    distance: f64,
    object: &'a Object,
	ray: Ray,
}

impl<'a> RayIntersection<'a> {
	pub fn distance(&self) -> f64 {
		self.distance
	}

	pub fn object(&self) -> &'a Object {
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

struct CornellBox {
	triangles: Vec<Triangle>,
	half_length: f64,
	box_z_offset: f64,
	red_diffuse_material: Material,
	green_diffuse_material: Material,
	blue_diffuse_material: Material,
	grey_diffuse_material: Material,
	sphere_light: Sphere,
}

impl Scene {
    fn new(triangles: Vec<Triangle>, spheres: Vec<Sphere>) -> Scene {
		let mut objects = Vec::new();
		for triangle in triangles {
			objects.push(Object::Triangle(triangle));
		}
		for sphere in spheres {
			objects.push(Object::Sphere(sphere));
		}

		let mut light_indexes = Vec::new();
		for i in 0..objects.len() {
			let object = objects.get(i).unwrap();
			if !object.material().emittance.is_black() {
				light_indexes.push(i);
			}
		}

		let bvh = BVH::build(&mut objects);

		Scene { objects, bvh, light_indexes }
    }

    fn load_teapot(scale: f64, offset: Point) -> Vec<Triangle> {
		let (models, _) = tobj::load_obj("obj/teapot.obj", true).unwrap();
		let m = &models[0];
		let mesh = &m.mesh;
		let material = Material::new(
			BSDF::Diffuse,
			Spectrum::grey(),
			Spectrum::black(),
		);

		let points: Vec<Point> = (0..mesh.positions.len() / 3).map(|v| {
			// scale
			let v = Vector::new(
                mesh.positions[3 * v].into(),
                mesh.positions[3 * v + 1].into(),
                mesh.positions[3 * v + 2].into(),
			);
			offset + (v * scale)
			// TODO: offset
		}).collect();

		let mut triangles = Vec::new();
		let mut next_face = 0;
		for f in 0..mesh.num_face_indices.len() {
			let end = next_face + mesh.num_face_indices[f] as usize;
			let face_indices: Vec<_> = mesh.indices[next_face..end].iter().collect();

			let triangle = Triangle::new(
				points[*face_indices[0] as usize],
				points[*face_indices[1] as usize],
				points[*face_indices[2] as usize],
				material,
			);
			triangles.push(triangle);
			next_face = end
		}
		triangles
    }

    pub fn new_teapot() -> Scene {
		let triangles = Scene::load_teapot(0.2, Point::new(0.0, 0.0, -40.0));
		Scene::new(triangles, Vec::new())
    }

	fn cornell_box() -> CornellBox {
		let half_length = 20.0;
		let box_z_offset = -48.0;
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
		// insert the light at the top of the scene, halfway through the triangle
        let sphere_light = Sphere::new(Point::new(0.0,
												  half_length + light_radius * 0.6,
												  box_z_offset - half_length / 2.0),
									   light_radius,
									   white_light_material);

		let z = box_z_offset - half_length;
		let p0 = Point::new(-half_length, -half_length, 1.0);
		let p1 = Point::new(-half_length, -half_length, z);
		let p2 = Point::new(half_length, -half_length, z);
		let p3 = Point::new(half_length, -half_length, 1.0);
		let p4 = Point::new(-half_length, half_length,  z);
		let p5 = Point::new(half_length, half_length, z);
		let p6 = Point::new(-half_length, half_length, 1.0);
		let p7 = Point::new(half_length, half_length, 1.0);
		let p8 = Point::new(-half_length, half_length, 1.0);
		let p9 = Point::new(-half_length, half_length, z);
		let p10 = Point::new(half_length, half_length, z);
		let p11 = Point::new(half_length, half_length, 1.0);
		// let p12 = Point::new(-half_length, half_length, 1.0);
		// let p13 = Point::new(half_length, half_length, 1.0);

        let triangles = vec![
			// bottom wall
			Triangle::new(
				p1,
				p0,
				p2,
				grey_diffuse_material,
			),
			Triangle::new(
				p3,
				p2,
				p0,
				grey_diffuse_material,
			),
			// top wall
			Triangle::new(
				p4,
				p5,
				p6,
				grey_diffuse_material,
			),
			Triangle::new(
				p7,
				p6,
				p5,
				grey_diffuse_material,
			),
			// back wall
			Triangle::new(
				p4,
				p1,
				p2,
				green_diffuse_material,
			),
			Triangle::new(
				p2,
				p5,
				p4,
				green_diffuse_material,
			),
			// left wall
			Triangle::new(
				p8,
				p0,
				p9,
				red_diffuse_material,
			),
			Triangle::new(
				p9,
				p8,
				p1,
				red_diffuse_material,
			),
			// right wall
			Triangle::new(
				p3,
				p11,
				p2,
				blue_diffuse_material,
			),
			Triangle::new(
				p10,
				p2,
				p11,
				blue_diffuse_material,
			),
		];

		CornellBox {
			triangles,
			sphere_light,
			half_length,
			box_z_offset,
		    red_diffuse_material,
		    green_diffuse_material,
		    blue_diffuse_material,
		    grey_diffuse_material,
		}
	}

	pub fn new_specular() -> Scene {
		let cb = Scene::cornell_box();
		let (half_length,
			 box_z_offset,
			 red_diffuse_material,
			 triangles,
		) = (cb.half_length, cb.box_z_offset, cb.red_diffuse_material, cb.triangles);
		let mirror_material = Material::new(
			BSDF::Specular,
			Spectrum::white(),
			Spectrum::black(),
		);
		let sphere_radius = 6.0;
        let spheres = vec![
			cb.sphere_light,
			Sphere::new(Point::new(-half_length / 3.0,
								   -half_length + sphere_radius,
								   box_z_offset - 2.0 * half_length / 3.0),
						sphere_radius,
						mirror_material),
			Sphere::new(Point::new(half_length / 3.0,
								   -half_length + sphere_radius,
								   box_z_offset - half_length / 3.0),
						sphere_radius,
						red_diffuse_material),
        ];

        Scene::new(triangles, spheres)
	}

    pub fn new_diffuse() -> Scene {
		let cb = Scene::cornell_box();
		let (half_length,
			 box_z_offset,
			 grey_diffuse_material,
			 red_diffuse_material,
			 triangles,
		) = (cb.half_length, cb.box_z_offset, cb.grey_diffuse_material, cb.red_diffuse_material, cb.triangles);
		let sphere_radius = 6.0;
        let spheres = vec![
			cb.sphere_light,
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
        ];

        Scene::new(triangles, spheres)
    }

	/// Intersects the scene with the given ray and takes ownership of it,
	/// in order to populate it in the intersection object without copying.
    pub fn intersect(&self, ray: Ray) -> Option<RayIntersection> {
        let mut min_dist = f64::INFINITY;
        let mut min_object: Option<&Object> = None;
		let hit_obj_aabbs = self.bvh.traverse(&ray_to_bvh_ray(&ray), &self.objects);
		for object in hit_obj_aabbs {
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

	pub fn lights(&self) -> Vec<&Object> {
		self.light_indexes.iter().map(|i| {
			self.objects.get(*i).unwrap()
		}).collect()
	}
}

pub fn ray_to_bvh_ray(ray: &Ray) -> bvh::ray::Ray {
	let origin = bvh::nalgebra::Point3::new(ray.origin.x() as f32, ray.origin.y() as f32, ray.origin.z() as f32);
	let direction = bvh::nalgebra::Vector3::new(ray.direction.x() as f32, ray.direction.y() as f32, ray.direction.z() as f32);
	bvh::ray::Ray::new(origin, direction)
}
