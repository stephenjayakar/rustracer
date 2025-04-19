use crate::canvas::Canvas;
use crate::common::{weighted_coin_flip, Spectrum};
use crate::scene::{Point, Ray, RayIntersection, Scene, Vector};
use crate::Config;
use rayon::prelude::*;

use std::{f64::consts::PI, sync::Arc, thread};

const RUSSIAN_ROULETTE_PROBABILITY: f32 = 0.7;

pub struct Raytracer {
    pub inner: Arc<RaytracerInner>,
}

// Camera movement speed
const CAMERA_SPEED: f64 = 2.0;

pub struct RaytracerInner {
    pub config: Config,
    canvas: Canvas,
    scene: Scene,
    camera_position: std::sync::Mutex<Point>,
    pub rendering_mode: std::sync::Mutex<RenderingMode>,
    // Flag to interrupt rendering
    interrupt: std::sync::atomic::AtomicBool,
    // Cache for screen-to-world transformations
    screen_to_world_cache: std::sync::Mutex<Option<Vec<Vector>>>,
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
        // Reset interrupt flag
        self.interrupt
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Get camera position once at the beginning
        let camera_pos = *self.camera_position.lock().unwrap();

        if self.config.single_threaded {
            'outer: for i in 0..self.config.screen_width {
                for j in 0..self.config.screen_height {
                    // Check for interrupt
                    if self.interrupt.load(std::sync::atomic::Ordering::SeqCst) {
                        break 'outer;
                    }

                    let color = self.render_helper(i, j, camera_pos);
                    self.canvas.draw_pixel(i, j, color);
                }
            }
        } else {
            // For parallel rendering, we use tile-based parallelism for better cache locality
            let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

            pool.install(|| {
                // Use tile-based rendering instead of row-based for better cache coherence and load balancing
                let tile_size = 32; // Experiment with different sizes
                let num_tiles_x = (self.config.screen_width + tile_size - 1) / tile_size;
                let num_tiles_y = (self.config.screen_height + tile_size - 1) / tile_size;
                let total_tiles = num_tiles_x * num_tiles_y;
                
                // Process tiles in a deterministic order
                let tiles: Vec<u32> = (0..total_tiles).collect();
                tiles.into_par_iter().for_each(|tile_idx| {
                    // Early exit if rendering was cancelled
                    if self.interrupt.load(std::sync::atomic::Ordering::SeqCst) {
                        return;
                    }
                    
                    let tile_x = (tile_idx % num_tiles_x) * tile_size;
                    let tile_y = (tile_idx / num_tiles_x) * tile_size;
                    
                    // Render each tile in scanline order for better cache locality and fewer artifacts
                    let x_end = std::cmp::min(tile_x + tile_size, self.config.screen_width);
                    let y_end = std::cmp::min(tile_y + tile_size, self.config.screen_height);
                    
                    // Buffer pixels for each tile to reduce thread contention in the canvas
                    let mut tile_pixels = Vec::with_capacity((tile_size * tile_size) as usize);
                    
                    for j in tile_y..y_end {
                        for i in tile_x..x_end {
                            // Check for cancellation less frequently
                            if i == tile_x && j % 16 == 0 && 
                               self.interrupt.load(std::sync::atomic::Ordering::Relaxed) {
                                return;
                            }

                            let color = self.render_helper(i, j, camera_pos);
                            tile_pixels.push((i, j, color));
                        }
                    }
                    
                    // Draw all pixels from this tile at once
                    for (i, j, color) in tile_pixels {
                        self.canvas.draw_pixel(i, j, color);
                    }
                });
            });
        }
    }

    fn render_helper(&self, i: u32, j: u32, camera_pos: Point) -> Spectrum {
        let vector = self.screen_to_world(i, j);
        let ray = Ray::new(camera_pos, vector);
        let mut color = Spectrum::black();
        
        if self.config.samples_per_pixel > 1 {
            // For first few samples, check if we can early terminate for dark areas
            for s in 0..std::cmp::min(2, self.config.samples_per_pixel) {
                let sample = self.cast_ray(ray, self.config.bounces);
                color += sample;
                
                // If color is completely black after initial samples, skip remaining samples
                if s == 1 && color.is_black() {
                    return Spectrum::black();
                }
            }
            
            // Process remaining samples
            if self.config.samples_per_pixel > 2 {
                for _ in 2..self.config.samples_per_pixel {
                    color += self.cast_ray(ray, self.config.bounces);
                }
            }
            
            color = color * (1.0 / self.config.samples_per_pixel as f64);
        } else {
            color = self.cast_ray(ray, self.config.bounces);
        }
        
        color
    }

    /// Algorithm to covert pixel positions in screen-space to a 3D Vector in world-space.
    /// Assumes the camera is pointing in -z at the origin.
    fn screen_to_world(&self, i: u32, j: u32) -> Vector {
        let w = self.config.screen_width as f64;
        let h = self.config.screen_height as f64;
        let aspect_ratio = w / h;
        let z = 1.7;
        let (iw, jh) = ((i as f64 + 0.5) / w, (j as f64 + 0.5) / h);
        let fov = self.config.fov;
        let half_fov = fov * 0.5;

        let start = f64::sin(-half_fov);
        let total = -2.0 * start;
        let xi = (start + iw * total) * aspect_ratio;
        let yi = -start - jh * total;

        let direction = Vector::new_normalized(xi, yi, -z);
        direction
    }
    
    /// Updates the screen-to-world transformation cache
    fn update_screen_to_world_cache(&self) {
        let mut cache_lock = self.screen_to_world_cache.lock().unwrap();
        
        // Only update if the cache is None or dimensions have changed
        if cache_lock.is_none() || 
           cache_lock.as_ref().unwrap().len() != (self.config.screen_width * self.config.screen_height) as usize {
            let width = self.config.screen_width;
            let height = self.config.screen_height;
            let mut cache = Vec::with_capacity((width * height) as usize);
            
            // Prebake values common to all transformations
            let w = width as f64;
            let h = height as f64;
            let aspect_ratio = w / h;
            let z = 1.7;
            let fov = self.config.fov;
            let half_fov = fov * 0.5;
            let start = f64::sin(-half_fov);
            let total = -2.0 * start;
            
            // Precompute in a single pass with optimized math
            // Process in row-major order for better cache locality
            for j in 0..height {
                let jh = (j as f64 + 0.5) / h;
                let yi = -start - jh * total;
                
                for i in 0..width {
                    let iw = (i as f64 + 0.5) / w;
                    let xi = (start + iw * total) * aspect_ratio;
                    
                    // Ensure all vectors are properly normalized
                    let mut direction = Vector::new(xi, yi, -z);
                    direction = direction.normalized();
                    cache.push(direction);
                }
            }
            
            *cache_lock = Some(cache);
        }
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
                let color = other_emittance * reflected * cos_theta * 2.0 * PI;
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
                    color += other_emittance * reflected * cos_theta * pdf;
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
        if !weighted_coin_flip(RUSSIAN_ROULETTE_PROBABILITY) {
            return l;
        }

        let wo = ray.direction;
        let sample = object.sample_bsdf(wo, normal);
        let (wi, pdf, reflected) = (sample.wi, sample.pdf, sample.reflected);

        let bounced_ray = Ray::new(intersection_point, wi);
        let mut color = self.cast_ray(bounced_ray, bounces_left - 1);

        if !color.is_black() {
            let cos_theta = f64::abs(wi.dot(normal));
            color = color * reflected * cos_theta * pdf;
        }
        l = l + color;
        l
    }

    /// Where the magic happens.
    fn cast_ray(&self, ray: Ray, bounces_left: u32) -> Spectrum {
        if let Some(ray_intersection) = self.scene.intersect(ray) {
            match bounces_left {
                0 => self.zero_bounce_radiance(&ray_intersection),
                1 => self.one_bounce_radiance_importance(&ray_intersection),
                _ => self.global_illumination(&ray_intersection, bounces_left),
            }
        } else {
            Spectrum::black()
        }
    }

    /// Renderer that paints grey for intersections, and black otherwise
    pub fn debug_render(&self) {
        // Reset interrupt flag
        self.interrupt
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Get camera position once at the beginning
        let camera_pos = *self.camera_position.lock().unwrap();
        
        // Precompute screen-to-world transformations for the entire frame
        // This is a significant optimization for debug mode since we only need basic ray tests
        let screen_width = self.config.screen_width;
        let screen_height = self.config.screen_height;
        
        // Update the screen-to-world cache for better performance
        self.update_screen_to_world_cache();

        // Use parallel rendering for better performance in debug mode
        if self.config.single_threaded {
            'outer: for i in 0..screen_width {
                for j in 0..screen_height {
                    // Check for interrupt
                    if self.interrupt.load(std::sync::atomic::Ordering::SeqCst) {
                        break 'outer;
                    }

                    let color = self.debug_render_helper(i, j, camera_pos);
                    self.canvas.draw_pixel(i, j, color);
                }
            }
        } else {
            // Use persistent thread pool
            let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

            pool.install(|| {
                // Use tile-based rendering for better cache coherence
                let tile_size = 32;
                let num_tiles_x = (screen_width + tile_size - 1) / tile_size;
                let num_tiles_y = (screen_height + tile_size - 1) / tile_size;
                let total_tiles = num_tiles_x * num_tiles_y;
                
                // Process tiles in a deterministic order to prevent artifacts
                let tiles: Vec<u32> = (0..total_tiles).collect();
                tiles.into_par_iter().for_each(|tile_idx| {
                    if self.interrupt.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    
                    let tile_x = (tile_idx % num_tiles_x) * tile_size;
                    let tile_y = (tile_idx / num_tiles_x) * tile_size;
                    
                    // Define tile boundaries
                    let y_end = std::cmp::min(tile_y + tile_size, screen_height);
                    let x_end = std::cmp::min(tile_x + tile_size, screen_width);
                    
                    // Buffer pixels for each tile to reduce thread contention
                    let mut tile_pixels = Vec::with_capacity((tile_size * tile_size) as usize);
                    
                    for j in tile_y..y_end {
                        for i in tile_x..x_end {
                            // Check interrupt flag less frequently for better performance
                            if i == tile_x && j % 16 == 0 && 
                               self.interrupt.load(std::sync::atomic::Ordering::Relaxed) {
                                return;
                            }

                            let color = self.debug_render_helper(i, j, camera_pos);
                            tile_pixels.push((i, j, color));
                        }
                    }
                    
                    // Draw all pixels from this tile at once
                    for (i, j, color) in tile_pixels {
                        self.canvas.draw_pixel(i, j, color);
                    }
                });
            });
        }
    }

    fn debug_render_helper(&self, i: u32, j: u32, camera_pos: Point) -> Spectrum {
        // Use cached screen-to-world transformation if available
        let vector = if let Some(cache) = &*self.screen_to_world_cache.lock().unwrap() {
            cache[(j * self.config.screen_width + i) as usize]
        } else {
            self.screen_to_world(i, j)
        };
        
        let ray = Ray::new(camera_pos, vector);

        // Use the optimized fast intersection test for debug mode
        if let Some(ri) = self.scene.intersect(ray) {
            // Use a simplified distance-based coloring with minimal math
            let max_distance = 100.0;
            let distance_factor = 1.0 - (ri.distance().min(max_distance) / max_distance);
            
            // Add some color variation based on normal direction for better visualization
            let normal = ri.object().surface_normal(ri.point());
            let normal_factor = 0.5 + 0.5 * normal.dot(Vector::new(0.5, 0.5, 0.5).normalized());
            
            // Return a color based on distance and normal, making debug view more informative
            Spectrum::new_f(
                0.7 * distance_factor * normal_factor,
                0.7 * distance_factor,
                0.7 * distance_factor * (1.0 - 0.5 * normal_factor),
            )
        } else {
            Spectrum::black()
        }
    }

    /// Helpful function to test a pixel's behavior.  Use this in combination
    /// with the mouse_down pixel print implemented
    pub fn test(&self, i: u32, j: u32) {
        let camera_pos = *self.camera_position.lock().unwrap();
        if !self.config.debug {
            println!("{:?}", self.render_helper(i, j, camera_pos));
        } else {
            println!("{:?}", self.debug_render_helper(i, j, camera_pos));
        }
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

        Raytracer {
            inner: Arc::new(RaytracerInner {
                config,
                canvas,
                scene,
                camera_position: std::sync::Mutex::new(Point::new(0.0, 0.0, 0.0)),
                rendering_mode: std::sync::Mutex::new(rendering_mode),
                interrupt: std::sync::atomic::AtomicBool::new(false),
                screen_to_world_cache: std::sync::Mutex::new(None),
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
        
        // Invalidate the screen-to-world cache when camera moves
        let mut cache = self.inner.screen_to_world_cache.lock().unwrap();
        *cache = None;
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
        self.inner
            .interrupt
            .store(true, std::sync::atomic::Ordering::SeqCst);
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
