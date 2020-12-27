#![allow(dead_code)]

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

    /// For each pixel of the output image, casts ray(s) into the `Scene` and writes the according
    /// `Spectrum` value to the `Canvas`.
    pub fn render(&mut self) {
        for i in 0..self.config.screen_width {
            for j in 0..self.config.screen_height {
                let vector = self.screen_to_world(i, j);
                let ray = Ray::new(self.config.origin, vector);
                let color = self.cast_ray(&ray, 1);
                self.canvas.draw_pixel(i, j, color);
            }
        }
    }

    /// Algorithm to covert pixel positions in screen-space to a 3D Vector in world-space.
    /// Assumes the camera is pointing in -z at the origin.
    fn screen_to_world(&self, i: u32, j: u32) -> Vector {
        let z = 2.0;
        let (iw, jh) = (
            (i as f64 + 0.5) / (self.config.screen_width as f64),
            (j as f64 + 0.5) / (self.config.screen_height as f64),
        );
        let fov = self.config.fov;
        let half_fov = fov * 0.5;

        let start = f64::sin(-half_fov) * z;
        let total = -2.0 * start;
        let xi = start + iw * total;
        let yi = -start - jh * total;

        let direction = Vector::new_normalized(xi, yi, -z);
        direction
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
                    let NUM_SAMPLES = 400;
                    let mut L = Spectrum::new(0, 0, 0);
                    for _ in 0..NUM_SAMPLES {
                        // direct lighting
                        let wo = ray.direction;
                        let wi = Vector::random_hemisphere(surface_normal);
                        let bounced_ray = Ray::new(intersection_point, wi);

                        let reflected = object.material().bsdf(wi, wo);
                        let other_emittance = self.cast_ray(&bounced_ray, 0);
                        if !other_emittance.is_black() {
                            let color =
                                emittance + (other_emittance * reflected * f64::abs(wi.z()));
                            L += color;
                        } else {
                            L += emittance;
                        }
                    }
                    // TODO: divide by num samples
                    L * 2.0 * std::f64::consts::PI * (4.0 / NUM_SAMPLES as f64)
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
