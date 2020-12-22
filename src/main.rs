use std::env;

extern crate sdl2;

mod canvas;
mod common;
mod scene;

use canvas::Canvas;
use common::{Spectrum, DEFAULT_SCREEN_HEIGHT, DEFAULT_SCREEN_WIDTH, EPS};
use scene::{Point, Ray, Scene, Vector};

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

struct Raytracer {
    config: Config,
    canvas: Canvas,
    scene: Scene,
}

impl Raytracer {
    fn new(config: Config) -> Raytracer {
        let canvas = Canvas::new(config.screen_width, config.screen_height);
        Raytracer {
            config,
            canvas,
            scene: Scene::new_preset(),
        }
    }

    pub fn render(&mut self) {
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
                let ray = Ray::new(self.config.origin, vector);
                let color = self.cast_ray(&ray, 0);
                self.canvas.draw_pixel(i, j, color);
            }
        }
    }

    fn cast_ray(&self, ray: &Ray, bounces_left: u32) -> Spectrum {
        if let Some(ray_intersection) = self.scene.intersect(ray) {
            let object = ray_intersection.object;
            let min_dist = ray_intersection.distance;
            let mut intersection_point: Point = ray.get_intersection_point(min_dist);
            // bumping the point a little out of the object to prevent self-collision
            let surface_normal: Vector = object.surface_normal(intersection_point);
            intersection_point = intersection_point + surface_normal.scale(EPS);

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
        } else {
            Spectrum::new(0, 0, 0)
        }
    }

    fn start(&mut self) {
        self.render();
        self.canvas.start();
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
    let mut raytracer = Raytracer::new(config);
    raytracer.start();
}
