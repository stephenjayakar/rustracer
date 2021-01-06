use crate::Config;
use crate::canvas::Canvas;
use crate::common::Spectrum;
use crate::scene::{Point, Ray, Scene, Vector, RayIntersection};
use rayon::prelude::*;

use std::f64::consts::PI;
use std::time::Instant;

const RUSSIAN_ROULETTE_PROBABILITY: f32 = 0.7;

pub struct Raytracer {
    config: Config,
    canvas: Canvas,
    scene: Scene,
}

impl Raytracer {
    pub fn new(config: Config) -> Raytracer {
        let canvas = Canvas::new(config.screen_width, config.screen_height, config.high_dpi, config.image_mode);
        Raytracer {
            config,
            canvas,
            scene: Scene::new_glass(),
        }
    }

    /// For each pixel of the output image, casts ray(s) into the `Scene` and writes the according
    /// `Spectrum` value to the `Canvas`.
    pub fn render(&self) {
		if self.config.single_threaded {
			(0..self.config.screen_width).for_each(|i| {
				(0..self.config.screen_height).for_each(|j| {
					let color = self.render_helper(i, j);
					self.canvas.draw_pixel(i, j, color);
				});
			});
		} else {
			(0..self.config.screen_width).into_par_iter().for_each(|i| {
				(0..self.config.screen_height).into_par_iter().for_each(|j| {
					let color = self.render_helper(i, j);
					self.canvas.draw_pixel(i, j, color);
				});
			});
		}
    }

	fn render_helper(&self, i: u32, j: u32) -> Spectrum {
        let vector = self.screen_to_world(i, j);
        let ray = Ray::new(self.config.origin, vector);
		let mut color = Spectrum::black();
		for _ in 0..self.config.samples_per_pixel {
			color += self.cast_ray(ray, self.config.bounces);
		}
		color = color * (1.0 / self.config.samples_per_pixel as f64);
		color
	}

    /// Algorithm to covert pixel positions in screen-space to a 3D Vector in world-space.
    /// Assumes the camera is pointing in -z at the origin.
    fn screen_to_world(&self, i: u32, j: u32) -> Vector {
        let z = 1.7;
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

	/// Radiance from immediate scene intersections.  Should only paint lights.
	fn zero_bounce_radiance(&self, intersection: &RayIntersection) -> Spectrum {
		intersection.object().material().emittance
	}

	/// Simulating one bounce radiance by using hemisphere sampling for the bounce direction.
	fn one_bounce_radiance_hemisphere(&self, intersection: &RayIntersection) -> Spectrum {
        let object = intersection.object();
		let ray = intersection.ray();
        let intersection_point: Point = intersection.point();
		let normal: Vector = object.surface_normal(intersection_point);

        let num_samples = self.config.light_samples;
        let mut l = Spectrum::black();
        for _ in 0..num_samples {
            // direct lighting
            let wo = ray.direction;
            let wi = Vector::random_hemisphere().to_coord_space(normal);
            let bounced_ray = Ray::new(intersection_point, wi);
			let other_emittance = self.cast_ray(bounced_ray, 0);

            if !other_emittance.is_black() {
				let reflected = object.bsdf(wi, wo);
				let cos_theta = f64::abs(wi.dot(normal));
                let color =
                    other_emittance * reflected * cos_theta * 2.0 * PI;
                l += color;
			}
		}
		l = l * (1.0 / num_samples as f64) + self.zero_bounce_radiance(intersection);
		l
	}

	/// One bounce radiance where we prioritize rays that go towards light sources.
	fn one_bounce_radiance_importance(&self, intersection: &RayIntersection) -> Spectrum {
		let mut l = Spectrum::black();
        let object = intersection.object();
		let ray = intersection.ray();
		let intersection_point = intersection.point();
		let normal = object.surface_normal(intersection_point);
		let num_light_samples = self.config.light_samples;

		for light in self.scene.lights() {
			let mut color = Spectrum::black();
			for _ in 0..num_light_samples {
				let wo = ray.direction;
				let sample = light.sample_l(intersection_point);
				let (pdf, wi) = (sample.pdf, sample.wi);
				let bounced_ray = Ray::new(intersection_point, wi);
				let other_emittance = self.cast_ray(bounced_ray, 0);

				if !other_emittance.is_black() {
					let reflected = object.bsdf(wi, wo);
					let cos_theta = f64::abs(wi.dot(normal));
					color +=
						other_emittance * reflected * cos_theta * pdf;
				}
			}
			l += color * (1.0 / num_light_samples as f64);
		}
		l += self.zero_bounce_radiance(intersection);
		l
	}

	/// Global illumination
	fn global_illumination(&self, intersection: &RayIntersection, bounces_left: u32) -> Spectrum {
		let object = intersection.object();
		let intersection_point = intersection.point();
		let normal = object.surface_normal(intersection_point);
		let ray = intersection.ray();

		let mut l = self.one_bounce_radiance_importance(intersection);

		// russian roulette for "infinite bounces"
		let sample = fastrand::f32();
		if sample > RUSSIAN_ROULETTE_PROBABILITY {
			return l
		}

		let wo = ray.direction;
		let sample = object.sample_bsdf(wo, normal);
		let (wi, pdf, reflected) = (sample.wi, sample.pdf, sample.reflected);

		let bounced_ray = Ray::new(intersection_point, wi);
		let mut color = self.cast_ray(bounced_ray, bounces_left - 1);

		if !color.is_black() {
			let cos_theta = f64::abs(wi.dot(normal));
			color =
				color * reflected * cos_theta * pdf;
		}
		l = l + color;
		l
	}

	/// Where the magic happens.
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
                    self.global_illumination(&ray_intersection, bounces_left)
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
		if let Some(ri) = self.scene.intersect(ray) {
			Spectrum::white() * (1.0 / f64::powf(2.0, ri.distance() / 10.0))
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
		if !self.config.debug {
			self.render();
		} else {
			self.debug_render();
		}
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
