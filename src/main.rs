#![allow(dead_code)]
#![feature(clamp)]

use std::env;

extern crate sdl2;

use rayon::prelude::*;

mod canvas;
mod common;
mod scene;

use canvas::Canvas;
use common::{Spectrum, DEFAULT_SCREEN_HEIGHT, DEFAULT_SCREEN_WIDTH};
use scene::{Point, Ray, Scene, Vector, RayIntersection};

use std::time::Instant;

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
    pub fn render(&self) {
        (0..self.config.screen_width).into_par_iter().for_each(|i| {
            (0..self.config.screen_height).into_par_iter().for_each(|j| {
				let color = self.render_helper(i, j);
				self.canvas.draw_pixel(i, j, color);
			});
		});
    }

	fn render_helper(&self, i: u32, j: u32) -> Spectrum {
        let vector = self.screen_to_world(i, j);
        let ray = Ray::new(self.config.origin, vector);
        let color = self.cast_ray(ray, 1);
		color
	}

    /// Algorithm to covert pixel positions in screen-space to a 3D Vector in world-space.
    /// Assumes the camera is pointing in -z at the origin.
    fn screen_to_world(&self, i: u32, j: u32) -> Vector {
        let z = 1.0;
        let (iw, jh) = (
            (i as f64 + 0.5) / (self.config.screen_width as f64),
            (j as f64 + 0.5) / (self.config.screen_height as f64),
        );
        let fov = self.config.fov;
        let half_fov = fov * 0.5;

        let start = f64::sin(-half_fov);
        let total = -2.0 * start;
        let xi = start + iw * total;
        let yi = -start - jh * total;

        let direction = Vector::new_normalized(xi, yi, -z);
        direction
    }

	fn zero_bounce_radiance(&self, intersection: &RayIntersection) -> Spectrum {
		intersection.object().material().emittance
	}

	fn one_bounce_radiance_hemisphere(&self, intersection: &RayIntersection) -> Spectrum {
        let object = intersection.object();
		let ray = intersection.ray();
        let intersection_point: Point = intersection.point();
		let normal: Vector = object.surface_normal(intersection_point);

        let emittance = object.material().emittance;
        let num_samples = 512;
        let mut l = Spectrum::black();
        for _ in 0..num_samples {
            // direct lighting
            let wo = ray.direction;
            let wi = Vector::random_hemisphere(normal);
            let bounced_ray = Ray::new(intersection_point, wi);
			let other_emittance = self.cast_ray(bounced_ray, 0);

            if !other_emittance.is_black() {
				let reflected = object.material().bsdf(wi, wo);
				let cos_theta = f32::abs(wi.dot(normal) as f32);
                let color =
                    other_emittance * reflected * cos_theta * 2.0 * std::f32::consts::PI;
                l += color;
			}
			// TODO: figure out what's going with PDF
		}
		l = l * (1.0 / num_samples as f32) + emittance;
		l
	}

	fn one_bounce_radiance_importance(&self, intersection: &RayIntersection) -> Spectrum {
		let mut l = Spectrum::black();
        let object = intersection.object();
        let emittance = object.material().emittance;
		let ray = intersection.ray();
		let intersection_point = intersection.point();
		let normal = object.surface_normal(intersection_point);
		let num_light_samples = 16;

		for light in self.scene.lights() {
			for _ in 0..num_light_samples {
				let wo = ray.direction;
				let sample = light.sample_l(intersection_point);
				let (pdf, wi) = (sample.pdf, sample.wi);
				let bounced_ray = Ray::new(intersection_point, wi);
				let other_emittance = self.cast_ray(bounced_ray, 0);

				if !other_emittance.is_black() {
					let reflected = object.material().bsdf(wi, wo);
					let cos_theta = f32::abs(wi.dot(normal) as f32);
					let color =
						other_emittance * reflected * cos_theta * pdf as f32;
					l += color;
				}
			}
			l = l * (1.0 / num_light_samples as f32);
		}
		l += emittance;
		l
	}

    fn cast_ray(&self, ray: Ray, bounces_left: u32) -> Spectrum {
        if let Some(ray_intersection) = self.scene.intersect(ray) {
            match bounces_left {
                0 => {
                    self.zero_bounce_radiance(&ray_intersection)
                }
                1 => {
					self.one_bounce_radiance_importance(
						&ray_intersection
					)
                }
                _ => {
                    // global illumination
                    unimplemented!()
                }
            }
        } else {
            Spectrum::black()
        }
    }

	/// Renderer that paints grey for intersections, and black otherwise
	pub fn debug_render(&self) {
        for i in 0..self.config.screen_width {
            for j in 0..self.config.screen_height {
				let color = self.debug_render_helper(i, j);
				self.canvas.draw_pixel(i, j, color);
			}
		}
	}

	fn debug_render_helper(&self, i: u32, j: u32) -> Spectrum {
        let vector = self.screen_to_world(i, j);
        let ray = Ray::new(self.config.origin, vector);
		if let Some(_) = self.scene.intersect(ray) {
			Spectrum::grey()
		} else {
			Spectrum::black()
		}
	}

	fn draw_axis(&self) {
		let (start_x, start_y) = (10, 10);
		let length = 15;
		let x_axis_color = Spectrum::red();
		let y_axis_color = Spectrum::green();

		for j in 0..length {
			self.canvas.draw_pixel(start_x, start_y + j, y_axis_color);
		}

		for i in 0..length {
			self.canvas.draw_pixel(start_x + i, start_y + length, x_axis_color);
		}
	}

    pub fn start(&self) {
		let start = Instant::now();
        self.render();
		let duration = start.elapsed();
		println!("Rendering took: {:?}", duration);
		self.draw_axis();
        self.canvas.start();
    }

	/// Helpful function to test a pixel's behavior.  Use this in combination
	/// with the mouse_down pixel print implemented
	fn test(&self) {
		let i = 1189;//227;
		let j = 855;//312;
		println!("{:?}", self.render_helper(i, j));
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
                .get(2)
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
    let raytracer = Raytracer::new(config);
	// raytracer.test();
    raytracer.start();
}
