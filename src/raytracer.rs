use crate::canvas::Canvas;
use crate::common::{weighted_coin_flip, Spectrum};
use crate::scene::{Point, Ray, RayIntersection, Scene, Vector};
use crate::Config;
use rayon::prelude::*;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, RwLock};
use std::{f64::consts::PI, sync::Arc, thread};

const RUSSIAN_ROULETTE_PROBABILITY: f32 = 0.7;

pub struct Raytracer {
    pub inner: Arc<RaytracerInner>,
}

// Camera movement speed
const CAMERA_SPEED: f64 = 2.0;

pub struct RaytracerInner {
    config: RwLock<RenderConfig>,
    canvas: Canvas,
    scene: RwLock<Scene>,
    pub camera_position: Mutex<Point>,
    pub rendering_mode: Mutex<RenderingMode>,
    // Flag to interrupt rendering
    interrupt: AtomicBool,
    // Rendering state
    pub is_rendering: AtomicBool,
    pub render_progress: AtomicU32, // 0-100
}

/// Dynamic render configuration that can be changed at runtime
#[derive(Clone)]
pub struct RenderConfig {
    pub screen_width: u32,
    pub screen_height: u32,
    pub fov: f64,
    pub samples_per_pixel: u32,
    pub light_samples: u32,
    pub bounces: u32,
    pub single_threaded: bool,
}

impl From<&Config> for RenderConfig {
    fn from(config: &Config) -> Self {
        RenderConfig {
            screen_width: config.screen_width,
            screen_height: config.screen_height,
            fov: config.fov,
            samples_per_pixel: config.samples_per_pixel,
            light_samples: config.light_samples,
            bounces: config.bounces,
            single_threaded: config.single_threaded,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum RenderingMode {
    Debug,
    Full,
}

impl RaytracerInner {
    /// For each pixel of the output image, casts ray(s) into the `Scene` and writes the according
    /// `Spectrum` value to the `Canvas`.
    pub fn render(&self) {
        let rendering_mode = *self.rendering_mode.lock().unwrap();
        match rendering_mode {
            RenderingMode::Full => self.do_render(),
            RenderingMode::Debug => self.debug_render(),
        }
    }

    fn do_render(&self) {
        // Reset interrupt flag and set rendering state
        self.interrupt.store(false, Ordering::SeqCst);
        self.is_rendering.store(true, Ordering::SeqCst);
        self.render_progress.store(0, Ordering::SeqCst);

        // Get camera position and config once at the beginning
        let camera_pos = *self.camera_position.lock().unwrap();
        let config = self.config.read().unwrap().clone();
        let scene = self.scene.read().unwrap();

        let total_rows = config.screen_width;

        if config.single_threaded {
            'outer: for i in 0..config.screen_width {
                for j in 0..config.screen_height {
                    // Check for interrupt
                    if self.interrupt.load(Ordering::SeqCst) {
                        break 'outer;
                    }

                    let color = self.render_helper(i, j, camera_pos, &config, &scene);
                    self.canvas.draw_pixel(i, j, color);
                }
                // Update progress
                let progress = ((i + 1) as f32 / total_rows as f32 * 100.0) as u32;
                self.render_progress.store(progress, Ordering::SeqCst);
            }
        } else {
            // For parallel rendering with progress tracking
            let completed_rows = AtomicU32::new(0);

            let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

            pool.install(|| {
                let rows: Vec<u32> = (0..config.screen_width).collect();

                rows.into_par_iter().for_each(|i| {
                    // Early exit if rendering was cancelled
                    if self.interrupt.load(Ordering::SeqCst) {
                        return;
                    }

                    for j in 0..config.screen_height {
                        // Skip if rendering was cancelled
                        if self.interrupt.load(Ordering::SeqCst) {
                            break;
                        }

                        let color = self.render_helper(i, j, camera_pos, &config, &scene);
                        self.canvas.draw_pixel(i, j, color);
                    }

                    // Update progress
                    let done = completed_rows.fetch_add(1, Ordering::SeqCst) + 1;
                    let progress = (done as f32 / total_rows as f32 * 100.0) as u32;
                    self.render_progress.store(progress, Ordering::SeqCst);
                });
            });
        }

        self.is_rendering.store(false, Ordering::SeqCst);
        self.render_progress.store(100, Ordering::SeqCst);
    }

    fn render_helper(
        &self,
        i: u32,
        j: u32,
        camera_pos: Point,
        config: &RenderConfig,
        scene: &Scene,
    ) -> Spectrum {
        let vector = self.screen_to_world(i, j, config);
        let ray = Ray::new(camera_pos, vector);
        let mut color = Spectrum::black();
        for _ in 0..config.samples_per_pixel {
            color += self.cast_ray(ray, config.bounces, config, scene);
        }
        color = color * (1.0 / config.samples_per_pixel as f64);
        color
    }

    /// Algorithm to covert pixel positions in screen-space to a 3D Vector in world-space.
    /// Assumes the camera is pointing in -z at the origin.
    fn screen_to_world(&self, i: u32, j: u32, config: &RenderConfig) -> Vector {
        let w = config.screen_width as f64;
        let h = config.screen_height as f64;
        let aspect_ratio = w / h;
        let z = 1.7;
        let (iw, jh) = ((i as f64 + 0.5) / w, (j as f64 + 0.5) / h);
        let fov = config.fov;
        let half_fov = fov * 0.5;

        let start = f64::sin(-half_fov);
        let total = -2.0 * start;
        let xi = (start + iw * total) * aspect_ratio;
        let yi = -start - jh * total;

        let direction = Vector::new_normalized(xi, yi, -z);
        direction
    }

    /// Radiance from immediate scene intersections.  Should only paint lights.
    fn zero_bounce_radiance(&self, intersection: &RayIntersection) -> Spectrum {
        intersection.object().material().emittance
    }

    /// Simulating one bounce radiance by using hemisphere sampling for the bounce direction.
    fn one_bounce_radiance_hemisphere(
        &self,
        intersection: &RayIntersection,
        config: &RenderConfig,
        scene: &Scene,
    ) -> Spectrum {
        let object = intersection.object();
        let ray = intersection.ray();
        let intersection_point: Point = intersection.point();
        let normal: Vector = object.surface_normal(intersection_point);

        let num_samples = config.light_samples;
        let mut l = Spectrum::black();
        for _ in 0..num_samples {
            // direct lighting
            let wo = ray.direction;
            let wi = Vector::random_hemisphere().to_coord_space(normal);
            let bounced_ray = Ray::new(intersection_point, wi);
            let other_emittance = self.cast_ray(bounced_ray, 0, config, scene);

            if !other_emittance.is_black() {
                let reflected = object.bsdf(wi, wo);
                let cos_theta = f64::abs(wi.dot(normal));
                let color = other_emittance * reflected * cos_theta * 2.0 * PI;
                l += color;
            }
        }
        l = l * (1.0 / num_samples as f64) + self.zero_bounce_radiance(intersection);
        l
    }

    /// One bounce radiance where we prioritize rays that go towards light sources.
    fn one_bounce_radiance_importance(
        &self,
        intersection: &RayIntersection,
        config: &RenderConfig,
        scene: &Scene,
    ) -> Spectrum {
        let mut l = Spectrum::black();
        let object = intersection.object();
        let ray = intersection.ray();
        let intersection_point = intersection.point();
        let normal = object.surface_normal(intersection_point);
        let num_light_samples = config.light_samples;

        for light in scene.lights() {
            let mut color = Spectrum::black();
            for _ in 0..num_light_samples {
                let wo = ray.direction;
                let sample = light.sample_l(intersection_point);
                let (pdf, wi) = (sample.pdf, sample.wi);
                let bounced_ray = Ray::new(intersection_point, wi);
                let other_emittance = self.cast_ray(bounced_ray, 0, config, scene);

                if !other_emittance.is_black() {
                    let reflected = object.bsdf(wi, wo);
                    let cos_theta = f64::abs(wi.dot(normal));
                    color += other_emittance * reflected * cos_theta * pdf;
                }
            }
            l += color * (1.0 / num_light_samples as f64);
        }
        l += self.zero_bounce_radiance(intersection);
        l
    }

    /// Global illumination
    fn global_illumination(
        &self,
        intersection: &RayIntersection,
        bounces_left: u32,
        config: &RenderConfig,
        scene: &Scene,
    ) -> Spectrum {
        let object = intersection.object();
        let intersection_point = intersection.point();
        let normal = object.surface_normal(intersection_point);
        let ray = intersection.ray();

        let mut l = self.one_bounce_radiance_importance(intersection, config, scene);

        // russian roulette for "infinite bounces"
        if !weighted_coin_flip(RUSSIAN_ROULETTE_PROBABILITY) {
            return l;
        }

        let wo = ray.direction;
        let sample = object.sample_bsdf(wo, normal);
        let (wi, pdf, reflected) = (sample.wi, sample.pdf, sample.reflected);

        let bounced_ray = Ray::new(intersection_point, wi);
        let mut color = self.cast_ray(bounced_ray, bounces_left - 1, config, scene);

        if !color.is_black() {
            let cos_theta = f64::abs(wi.dot(normal));
            color = color * reflected * cos_theta * pdf;
        }
        l = l + color;
        l
    }

    /// Where the magic happens.
    fn cast_ray(
        &self,
        ray: Ray,
        bounces_left: u32,
        config: &RenderConfig,
        scene: &Scene,
    ) -> Spectrum {
        if let Some(ray_intersection) = scene.intersect(ray) {
            match bounces_left {
                0 => self.zero_bounce_radiance(&ray_intersection),
                1 => self.one_bounce_radiance_importance(&ray_intersection, config, scene),
                _ => self.global_illumination(&ray_intersection, bounces_left, config, scene),
            }
        } else {
            Spectrum::black()
        }
    }

    /// Renderer that paints grey for intersections, and black otherwise
    pub fn debug_render(&self) {
        // Reset interrupt flag
        self.interrupt.store(false, Ordering::SeqCst);
        self.is_rendering.store(true, Ordering::SeqCst);

        // Get camera position and config once at the beginning
        let camera_pos = *self.camera_position.lock().unwrap();
        let config = self.config.read().unwrap().clone();
        let scene = self.scene.read().unwrap();

        // Use parallel rendering for better performance in debug mode
        if config.single_threaded {
            'outer: for i in 0..config.screen_width {
                for j in 0..config.screen_height {
                    // Check for interrupt
                    if self.interrupt.load(Ordering::SeqCst) {
                        break 'outer;
                    }

                    let color = self.debug_render_helper(i, j, camera_pos, &config, &scene);
                    self.canvas.draw_pixel(i, j, color);
                }
            }
        } else {
            // Parallel implementation for debug mode
            let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

            pool.install(|| {
                let rows: Vec<u32> = (0..config.screen_width).collect();

                rows.into_par_iter().for_each(|i| {
                    if self.interrupt.load(Ordering::SeqCst) {
                        return;
                    }

                    for j in 0..config.screen_height {
                        if self.interrupt.load(Ordering::SeqCst) {
                            break;
                        }

                        let color = self.debug_render_helper(i, j, camera_pos, &config, &scene);
                        self.canvas.draw_pixel(i, j, color);
                    }
                });
            });
        }

        self.is_rendering.store(false, Ordering::SeqCst);
    }

    fn debug_render_helper(
        &self,
        i: u32,
        j: u32,
        camera_pos: Point,
        config: &RenderConfig,
        scene: &Scene,
    ) -> Spectrum {
        let vector = self.screen_to_world(i, j, config);
        let ray = Ray::new(camera_pos, vector);

        // Extremely simplified rendering for debug mode
        // Just show if there's an intersection or not, with minimal calculation
        if let Some(ri) = scene.intersect(ray) {
            // Use a simple distance-based coloring with minimal math
            let max_distance = 100.0;
            let distance_factor = 1.0 - (ri.distance().min(max_distance) / max_distance);

            // Return a gray color without additional calculations
            Spectrum::new_f(
                0.7 * distance_factor,
                0.7 * distance_factor,
                0.7 * distance_factor,
            )
        } else {
            Spectrum::black()
        }
    }

    /// Helpful function to test a pixel's behavior.  Use this in combination
    /// with the mouse_down pixel print implemented
    pub fn test(&self, i: u32, j: u32) {
        let camera_pos = *self.camera_position.lock().unwrap();
        let config = self.config.read().unwrap().clone();
        let scene = self.scene.read().unwrap();
        println!(
            "{:?}",
            self.debug_render_helper(i, j, camera_pos, &config, &scene)
        );
    }
}

impl Raytracer {
    pub fn new(config: Config, scene: Scene) -> Raytracer {
        let canvas = Canvas::new(
            config.screen_width,
            config.screen_height,
            config.high_dpi,
            config.image_mode,
        );

        // Always start in debug mode for navigation
        let rendering_mode = RenderingMode::Debug;
        let render_config = RenderConfig::from(&config);

        Raytracer {
            inner: Arc::new(RaytracerInner {
                config: RwLock::new(render_config),
                canvas,
                scene: RwLock::new(scene),
                camera_position: Mutex::new(Point::new(0.0, 0.0, 0.0)),
                rendering_mode: Mutex::new(rendering_mode),
                interrupt: AtomicBool::new(false),
                is_rendering: AtomicBool::new(false),
                render_progress: AtomicU32::new(0),
            }),
        }
    }

    pub fn start(&self) {
        // Initial render in a background thread
        self.render(false);

        // Start the GUI
        self.inner.canvas.start(self.inner.clone());
    }

    pub fn move_camera(&self, direction: Vector) {
        let mut camera_pos = self.inner.camera_position.lock().unwrap();
        *camera_pos = *camera_pos + direction * CAMERA_SPEED;
    }

    pub fn reset_camera(&self) {
        let mut camera_pos = self.inner.camera_position.lock().unwrap();
        *camera_pos = Point::new(0.0, 0.0, 0.0);
    }

    pub fn toggle_rendering_mode(&self) {
        // First interrupt any ongoing render
        self.interrupt_render();

        // Then toggle the mode
        let mut mode = self.inner.rendering_mode.lock().unwrap();
        *mode = match *mode {
            RenderingMode::Debug => RenderingMode::Full,
            RenderingMode::Full => RenderingMode::Debug,
        };
    }

    pub fn interrupt_render(&self) {
        // Set the interrupt flag to true
        self.inner.interrupt.store(true, Ordering::SeqCst);
    }

    /// Update render settings dynamically
    pub fn update_render_settings(
        &self,
        samples_per_pixel: u32,
        light_samples: u32,
        max_bounces: u32,
    ) {
        let mut config = self.inner.config.write().unwrap();
        config.samples_per_pixel = samples_per_pixel;
        config.light_samples = light_samples;
        config.bounces = max_bounces;
    }

    /// Set a new scene
    pub fn set_scene(&self, scene: Scene) {
        self.interrupt_render();
        let mut current_scene = self.inner.scene.write().unwrap();
        *current_scene = scene;
    }

    /// Triggers a render, with option to wait for completion
    /// Passing wait_for_completion=true will block until rendering is done,
    /// which is useful for full renders that shouldn't be interrupted
    pub fn render(&self, wait_for_completion: bool) {
        let local_self = self.inner.clone();

        if wait_for_completion {
            // Run directly in this thread and wait for completion
            local_self.render();
        } else {
            // Run in a background thread
            thread::spawn(move || {
                local_self.render();
            });
        }
    }
}
