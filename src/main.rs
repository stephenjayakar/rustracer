use std::env;

extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

mod canvas;
mod common;
mod objects;
mod primitives;

use canvas::Canvas;
use common::Spectrum;
use objects::{Material, Object, Plane, Sphere, BSDF};
use primitives::{Point, Ray, Vector};

const DEFAULT_SCREEN_WIDTH: u32 = 1200;
const DEFAULT_SCREEN_HEIGHT: u32 = 1200;
const EPS: f64 = 0.0000001;

const GENERIC_ERROR: &str = "Something went wrong, sorry!";

struct Config {
    screen_width: u32,
    screen_height: u32,
    fov: f64,
    origin: Point,
}

impl Config {
    // fov is in degrees here
    fn new(width: u32, height: u32, fov_degrees: f64) -> Config {
        Config {
            screen_width: width,
            screen_height: height,
            fov: f64::to_radians(fov_degrees),
            origin: Point::new(0.0, 0.0, 0.0),
        }
    }
}

struct Scene {
    planes: Vec<Plane>,
    spheres: Vec<Sphere>,
}

struct RayIntersection<'a> {
    distance: f64,
    object: &'a dyn Object,
}

impl Scene {
    fn new() -> Scene {
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
    fn cast_ray(&self, ray: &Ray) -> Spectrum {
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

struct Raytracer {
    config: Config,
    canvas: Canvas,
    scene: Scene,
}

impl Raytracer {
    fn new(config: Config, scene: Scene) -> Raytracer {
        let canvas = Canvas::new(config.screen_width, config.screen_height);
        Raytracer {
            config,
            canvas,
            scene,
        }
    }

    fn render(&mut self) {
        // start rendering
        // ray casting algorithm
        let x_width = 2.0 * f64::tan(self.config.fov / 2.0);
        let y_width = 2.0 * f64::tan(self.config.fov / 2.0);

        let x_step = x_width / (self.config.screen_width as f64);
        let x_start = -x_width / 2.0;
        let y_step = y_width / (self.config.screen_height as f64);
        let y_start = y_width / 2.0;

        for i in 0..self.config.screen_width {
            for j in 0..self.config.screen_height {
                let x_component = x_start + x_step * (i as f64);
                let y_component = y_start - y_step * (j as f64);
                let vector = Vector::new(x_component, y_component, -1.0);
                let ray = Ray::new(self.config.origin, vector, 0);
                let color = self.scene.cast_ray(&ray);
                self.canvas.draw_pixel(i, j, color);
            }
        }
    }
}

fn parse_args(args: Vec<String>) -> Option<(u32, u32)> {
    match args.len() {
        3 => {
            let width = args
                .get(1)
                .unwrap()
                .parse()
                .expect("passed in invalid width");
            let height = args
                .get(1)
                .unwrap()
                .parse()
                .expect("passed in invalid width");
            Some((width, height))
        }
        _ => None,
    }
}

fn main() {
    // parse args
    let args: Vec<String> = env::args().collect();
    let (screen_width, screen_height) = match parse_args(args) {
        None => (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT),
        Some((width, height)) => (width, height),
    };

    // set up raytracer
    let config = Config::new(screen_width, screen_height, 90.0);
    let mut raytracer = Raytracer::new(config, Scene::new());
    raytracer.render();
    loop {}
}
