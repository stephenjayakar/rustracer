use crate::canvas::Canvas;
use crate::common::{weighted_coin_flip, Spectrum};
use crate::scene::{Point, Ray, RayIntersection, Scene, Vector};
use crate::Config;
use rayon::prelude::*;

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

const RUSSIAN_ROULETTE_PROBABILITY: f32 = 0.7;

pub struct Raytracer {
    pub inner: Arc<RaytracerInner>,
}

// Camera movement speed
const CAMERA_SPEED: f32 = 2.0;

/// Shared pixel buffer that render threads write to directly.
/// Each pixel is 4 bytes (RGBA). Uses AtomicU8 for lock-free writes.
pub struct SharedPixelBuffer {
    pub data: Vec<AtomicU8>,
    pub width: u32,
    pub height: u32,
}

impl SharedPixelBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(AtomicU8::new(0));
        }
        SharedPixelBuffer {
            data,
            width,
            height,
        }
    }

    #[inline(always)]
    pub fn set_pixel(&self, x: u32, y: u32, s: Spectrum) {
        let index = ((y * self.width + x) * 4) as usize;
        if index + 3 < self.data.len() {
            self.data[index].store(s.r(), Ordering::Relaxed);
            self.data[index + 1].store(s.g(), Ordering::Relaxed);
            self.data[index + 2].store(s.b(), Ordering::Relaxed);
            self.data[index + 3].store(255, Ordering::Relaxed);
        }
    }

    /// Copy atomic buffer into a plain Vec<u8> for texture upload.
    /// Uses relaxed ordering since we only need a consistent snapshot for display.
    pub fn snapshot(&self, dst: &mut Vec<u8>) {
        debug_assert_eq!(dst.len(), self.data.len());
        for (i, atom) in self.data.iter().enumerate() {
            dst[i] = atom.load(Ordering::Relaxed);
        }
    }

    pub fn clear(&self) {
        for (i, atom) in self.data.iter().enumerate() {
            atom.store(if i % 4 == 3 { 255 } else { 0 }, Ordering::Relaxed);
        }
    }
}

pub struct RaytracerInner {
    config: RwLock<RenderConfig>,
    pub canvas: Canvas,
    scene: RwLock<Scene>,
    pub camera_position: Mutex<Point>,
    pub rendering_mode: Mutex<RenderingMode>,
    // Flag to interrupt rendering
    interrupt: AtomicBool,
    // Rendering state
    pub is_rendering: AtomicBool,
    pub render_progress: AtomicU32, // 0-100
    // Shared pixel buffer - render threads write here, GUI reads
    pub pixel_buffer: Arc<SharedPixelBuffer>,
    // Reusable rayon thread pool
    thread_pool: rayon::ThreadPool,
}

/// Dynamic render configuration that can be changed at runtime
#[derive(Clone)]
pub struct RenderConfig {
    pub screen_width: u32,
    pub screen_height: u32,
    pub fov: f32,
    pub samples_per_pixel: u32,
    pub light_samples: u32,
    pub bounces: u32,
    pub single_threaded: bool,
}

/// Precomputed values for screen_to_world that only depend on screen size and FOV
#[derive(Clone)]
struct ScreenParams {
    inv_w: f32,
    inv_h: f32,
    start: f32,
    total: f32,
    aspect_ratio: f32,
    z: f32,
}

impl ScreenParams {
    fn from_config(config: &RenderConfig) -> Self {
        let w = config.screen_width as f32;
        let h = config.screen_height as f32;
        let half_fov = config.fov * 0.5;
        let start = f32::sin(-half_fov);
        let total = -2.0 * start;
        ScreenParams {
            inv_w: 1.0 / w,
            inv_h: 1.0 / h,
            start,
            total,
            aspect_ratio: w / h,
            z: 1.7,
        }
    }

    #[inline(always)]
    fn screen_to_world(&self, i: u32, j: u32) -> Vector {
        let iw = (i as f32 + 0.5) * self.inv_w;
        let jh = (j as f32 + 0.5) * self.inv_h;
        let xi = (self.start + iw * self.total) * self.aspect_ratio;
        let yi = -self.start - jh * self.total;
        Vector::new_normalized(xi, yi, -self.z)
    }
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
    /// `Spectrum` value to the shared pixel buffer.
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
        let screen_params = ScreenParams::from_config(&config);

        let total_rows = config.screen_width;

        if config.single_threaded {
            'outer: for i in 0..config.screen_width {
                for j in 0..config.screen_height {
                    if self.interrupt.load(Ordering::Relaxed) {
                        break 'outer;
                    }
                    let color =
                        self.render_helper(i, j, camera_pos, &config, &scene, &screen_params);
                    self.pixel_buffer.set_pixel(i, j, color);
                }
                let progress = ((i + 1) as f32 / total_rows as f32 * 100.0) as u32;
                self.render_progress.store(progress, Ordering::Relaxed);
            }
        } else {
            let completed_rows = AtomicU32::new(0);

            self.thread_pool.install(|| {
                (0..config.screen_width).into_par_iter().for_each(|i| {
                    if self.interrupt.load(Ordering::Relaxed) {
                        return;
                    }

                    for j in 0..config.screen_height {
                        if self.interrupt.load(Ordering::Relaxed) {
                            break;
                        }
                        let color =
                            self.render_helper(i, j, camera_pos, &config, &scene, &screen_params);
                        self.pixel_buffer.set_pixel(i, j, color);
                    }

                    let done = completed_rows.fetch_add(1, Ordering::Relaxed) + 1;
                    let progress = (done as f32 / total_rows as f32 * 100.0) as u32;
                    self.render_progress.store(progress, Ordering::Relaxed);
                });
            });
        }

        self.is_rendering.store(false, Ordering::SeqCst);
        self.render_progress.store(100, Ordering::SeqCst);
    }

    #[inline(always)]
    fn render_helper(
        &self,
        i: u32,
        j: u32,
        camera_pos: Point,
        config: &RenderConfig,
        scene: &Scene,
        screen_params: &ScreenParams,
    ) -> Spectrum {
        let vector = screen_params.screen_to_world(i, j);
        let ray = Ray::new_prenormalized(camera_pos, vector); // already normalized by screen_to_world
        let mut color = Spectrum::black();
        for _ in 0..config.samples_per_pixel {
            color += self.cast_ray(ray, config.bounces, config, scene);
        }
        color = color * (1.0 / config.samples_per_pixel as f32);
        color
    }

    /// Radiance from immediate scene intersections.  Should only paint lights.
    #[inline(always)]
    fn zero_bounce_radiance(&self, intersection: &RayIntersection) -> Spectrum {
        intersection.object().material().emittance
    }

    /// One bounce radiance using light-source importance sampling.
    #[inline(always)]
    fn one_bounce_radiance_importance(
        &self,
        intersection: &RayIntersection,
        intersection_point: Point,
        normal: Vector,
        config: &RenderConfig,
        scene: &Scene,
    ) -> Spectrum {
        let mut l = Spectrum::black();
        let object = intersection.object();
        let wo = intersection.ray().direction;
        let num_light_samples = config.light_samples;
        let inv_light_samples = 1.0 / num_light_samples as f32;

        // Iterate lights without allocating a Vec
        for &light_idx in scene.light_indexes() {
            let light = scene.get_object(light_idx);
            let light_emittance = &light.material().emittance;
            let mut color = Spectrum::black();
            for _ in 0..num_light_samples {
                let sample = light.sample_l(intersection_point);
                let (pdf, wi) = (sample.pdf, sample.wi);

                // Shadow ray: check if path to light sample point is blocked
                let shadow_ray = Ray::new_prenormalized(intersection_point, wi);
                if !scene.is_occluded(&shadow_ray, sample.distance) {
                    let reflected = object.bsdf(wi, wo);
                    let cos_theta = f32::abs(wi.dot(normal));
                    color += *light_emittance * reflected * cos_theta * pdf;
                }
            }
            l += color * inv_light_samples;
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
        let normal = intersection.normal();

        let mut l = self.one_bounce_radiance_importance(
            intersection,
            intersection_point,
            normal,
            config,
            scene,
        );

        // russian roulette for "infinite bounces"
        if !weighted_coin_flip(RUSSIAN_ROULETTE_PROBABILITY) {
            return l;
        }

        let wo = intersection.ray().direction;
        let sample = object.sample_bsdf(wo, normal);
        let (wi, pdf, reflected) = (sample.wi, sample.pdf, sample.reflected);

        let bounced_ray = Ray::new(intersection_point, wi);
        let mut color = self.cast_ray(bounced_ray, bounces_left - 1, config, scene);

        if !color.is_black() {
            let cos_theta = f32::abs(wi.dot(normal));
            color = color * reflected * cos_theta * pdf;
        }
        l = l + color;
        l
    }

    /// Where the magic happens.
    #[inline(always)]
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
                1 => {
                    let pt = ray_intersection.point();
                    let n = ray_intersection.normal();
                    self.one_bounce_radiance_importance(&ray_intersection, pt, n, config, scene)
                }
                _ => self.global_illumination(&ray_intersection, bounces_left, config, scene),
            }
        } else {
            Spectrum::black()
        }
    }

    /// Renderer that paints grey for intersections, and black otherwise
    pub fn debug_render(&self) {
        self.interrupt.store(false, Ordering::SeqCst);
        self.is_rendering.store(true, Ordering::SeqCst);

        let camera_pos = *self.camera_position.lock().unwrap();
        let config = self.config.read().unwrap().clone();
        let scene = self.scene.read().unwrap();
        let screen_params = ScreenParams::from_config(&config);

        if config.single_threaded {
            'outer: for i in 0..config.screen_width {
                for j in 0..config.screen_height {
                    if self.interrupt.load(Ordering::Relaxed) {
                        break 'outer;
                    }
                    let color = self.debug_render_helper(i, j, camera_pos, &scene, &screen_params);
                    self.pixel_buffer.set_pixel(i, j, color);
                }
            }
        } else {
            self.thread_pool.install(|| {
                (0..config.screen_width).into_par_iter().for_each(|i| {
                    if self.interrupt.load(Ordering::Relaxed) {
                        return;
                    }
                    for j in 0..config.screen_height {
                        if self.interrupt.load(Ordering::Relaxed) {
                            break;
                        }
                        let color =
                            self.debug_render_helper(i, j, camera_pos, &scene, &screen_params);
                        self.pixel_buffer.set_pixel(i, j, color);
                    }
                });
            });
        }

        self.is_rendering.store(false, Ordering::SeqCst);
    }

    #[inline(always)]
    fn debug_render_helper(
        &self,
        i: u32,
        j: u32,
        camera_pos: Point,
        scene: &Scene,
        screen_params: &ScreenParams,
    ) -> Spectrum {
        let vector = screen_params.screen_to_world(i, j);
        let ray = Ray::new_prenormalized(camera_pos, vector); // already normalized

        if let Some(ri) = scene.intersect(ray) {
            let max_distance: f32 = 100.0;
            let distance_factor = 1.0 - (ri.distance().min(max_distance) / max_distance);
            Spectrum::new_f(
                0.7 * distance_factor,
                0.7 * distance_factor,
                0.7 * distance_factor,
            )
        } else {
            Spectrum::black()
        }
    }

    /// Helpful function to test a pixel's behavior.
    pub fn test(&self, i: u32, j: u32) {
        let camera_pos = *self.camera_position.lock().unwrap();
        let config = self.config.read().unwrap().clone();
        let scene = self.scene.read().unwrap();
        let screen_params = ScreenParams::from_config(&config);
        println!(
            "{:?}",
            self.debug_render_helper(i, j, camera_pos, &scene, &screen_params)
        );
    }
}

impl Raytracer {
    pub fn new(config: Config, scene: Scene) -> Raytracer {
        let pixel_buffer = Arc::new(SharedPixelBuffer::new(
            config.screen_width,
            config.screen_height,
        ));

        let canvas = Canvas::new(
            config.screen_width,
            config.screen_height,
            config.high_dpi,
            config.image_mode,
        );

        let rendering_mode = RenderingMode::Debug;
        let render_config = RenderConfig::from(&config);

        // Build the thread pool once, reuse for all renders
        let thread_pool = rayon::ThreadPoolBuilder::new().build().unwrap();

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
                pixel_buffer,
                thread_pool,
            }),
        }
    }

    pub fn start(&self) {
        // Only do initial background debug render in GUI mode (skip for image mode)
        if !self.inner.canvas.image_mode {
            self.render(false);
        }

        // Start the GUI (or image mode)
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
        self.interrupt_render();
        let mut mode = self.inner.rendering_mode.lock().unwrap();
        *mode = match *mode {
            RenderingMode::Debug => RenderingMode::Full,
            RenderingMode::Full => RenderingMode::Debug,
        };
    }

    pub fn interrupt_render(&self) {
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
    pub fn render(&self, wait_for_completion: bool) {
        let local_self = self.inner.clone();

        if wait_for_completion {
            local_self.render();
        } else {
            thread::spawn(move || {
                local_self.render();
            });
        }
    }
}
