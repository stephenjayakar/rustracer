extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

extern crate png;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::gui::{GuiAction, GuiState, SceneType};
use crate::scene::Scene;

use egui_sdl2_gl::egui;

const REFRESH_RATE: u64 = 1000 / 60; // 60 FPS for smooth GUI

/// Mostly contains concurrency primitives to properly wrap
/// around SDL2 context.
pub struct Canvas {
    width: u32,
    height: u32,
    high_dpi: bool,
    pub image_mode: bool,
}

impl Canvas {
    pub fn new(width: u32, height: u32, high_dpi: bool, image_mode: bool) -> Canvas {
        Canvas {
            width,
            height,
            high_dpi,
            image_mode,
        }
    }

    fn new_png_writer(&self) -> png::Writer<BufWriter<File>> {
        std::fs::create_dir_all("./dump").ok();

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let path_string = format!("./dump/{}.png", timestamp.as_secs().to_string());
        println!("Saving with filename {}", path_string);
        let path = Path::new(&path_string);
        let file = File::create(path).unwrap();
        let w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width, self.height);
        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header().unwrap()
    }

    fn save_canvas(&self, pixel_buffer_rgb: &[u8]) {
        let mut writer = self.new_png_writer();
        writer
            .write_image_data(pixel_buffer_rgb)
            .expect("Failed to write canvas to png");
        println!("Image saved successfully!");
    }

    /// Extract RGB data from RGBA pixel buffer for PNG saving
    fn rgba_to_rgb(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
        let pixel_count = (width * height) as usize;
        let mut rgb = Vec::with_capacity(pixel_count * 3);
        for i in 0..pixel_count {
            rgb.push(rgba[i * 4]);
            rgb.push(rgba[i * 4 + 1]);
            rgb.push(rgba[i * 4 + 2]);
        }
        rgb
    }

    pub fn start(&self, raytracer_inner: Arc<crate::raytracer::RaytracerInner>) {
        if self.image_mode {
            self.start_image_mode(raytracer_inner);
        } else {
            self.start_gui(raytracer_inner);
        }
    }

    /// Headless mode: render to image and save to PNG without opening a window
    fn start_image_mode(&self, raytracer_inner: Arc<crate::raytracer::RaytracerInner>) {
        let raytracer = crate::raytracer::Raytracer {
            inner: raytracer_inner,
        };

        {
            let mut mode = raytracer.inner.rendering_mode.lock().unwrap();
            *mode = crate::raytracer::RenderingMode::Full;
        }

        println!("Rendering image...");
        let start = std::time::Instant::now();
        raytracer.render(true);
        let elapsed = start.elapsed();
        println!("Render time: {:.3}s", elapsed.as_secs_f64());

        // Snapshot the shared pixel buffer and save
        let pixel_count = (self.width * self.height) as usize;
        let mut rgba = vec![0u8; pixel_count * 4];
        raytracer.inner.pixel_buffer.snapshot(&mut rgba);
        let rgb = Self::rgba_to_rgb(&rgba, self.width, self.height);
        self.save_canvas(&rgb);
    }

    /// Starts a new canvas context that takes over the main thread.
    pub fn start_gui(&self, raytracer_inner: Arc<crate::raytracer::RaytracerInner>) {
        let raytracer = crate::raytracer::Raytracer {
            inner: raytracer_inner,
        };

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let divider = if self.high_dpi { 2 } else { 1 };
        let win_width = self.width / divider;
        let win_height = self.height / divider;

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_double_buffer(true);
        gl_attr.set_multisample_samples(4);

        let window = video_subsystem
            .window("Rustracer - Path Tracer", win_width, win_height)
            .opengl()
            .allow_highdpi()
            .resizable()
            .build()
            .unwrap();

        let _gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

        // Initialize egui with correct Retina DPI scaling.
        let shader_ver = egui_sdl2_gl::ShaderVersion::Default;
        let (drawable_w, drawable_h) = window.drawable_size();
        let dpi_scale = drawable_w as f32 / win_width as f32;
        let (mut egui_painter, mut egui_state) = egui_sdl2_gl::with_sdl2(
            &window,
            shader_ver,
            egui_sdl2_gl::DpiScaling::Custom(dpi_scale),
        );
        let egui_ctx = egui::Context::default();

        egui_painter.update_screen_rect((drawable_w, drawable_h));
        egui_state.input.screen_rect = Some(egui_painter.screen_rect);

        egui_ctx.set_visuals(egui::Visuals::dark());

        let mut gui_state = GuiState::new();

        // Double-buffered pixel snapshot to avoid cloning the entire buffer every frame
        let pixel_count = (self.width * self.height * 4) as usize;
        let mut pixel_snapshot: Vec<u8> = vec![0; pixel_count];

        // Register the render texture with egui's painter
        let render_texture_id = egui_painter.new_user_texture_rgba8(
            (self.width as usize, self.height as usize),
            pixel_snapshot.clone(),
            false,
        );

        let mut needs_render = false;

        let mut event_pump = sdl_context.event_pump().unwrap();

        // canvas loop
        'running: loop {
            // Snapshot shared pixel buffer into our local copy (no channel, no clone)
            // This is a fast memcpy-like operation reading from AtomicU8s
            raytracer.inner.pixel_buffer.snapshot(&mut pixel_snapshot);

            // Update the egui texture with the snapshot data
            // This avoids cloning - we pass the data directly
            egui_painter.update_user_texture_rgba8_data(
                render_texture_id,
                pixel_snapshot.clone(), // TODO: egui_sdl2_gl API requires owned Vec, unavoidable
            );

            // Re-render in debug mode when flagged and not already rendering
            let is_currently_rendering = raytracer
                .inner
                .is_rendering
                .load(std::sync::atomic::Ordering::Relaxed);
            if needs_render && !is_currently_rendering {
                let current_mode = *raytracer.inner.rendering_mode.lock().unwrap();
                if current_mode == crate::raytracer::RenderingMode::Debug {
                    raytracer.render(false);
                }
                needs_render = false;
            }

            // Update GUI state from raytracer
            {
                let camera_pos = *raytracer.inner.camera_position.lock().unwrap();
                gui_state.update_camera(camera_pos.x(), camera_pos.y(), camera_pos.z());
                gui_state.is_debug_mode = *raytracer.inner.rendering_mode.lock().unwrap()
                    == crate::raytracer::RenderingMode::Debug;
                gui_state.is_rendering = raytracer
                    .inner
                    .is_rendering
                    .load(std::sync::atomic::Ordering::Relaxed);
                gui_state.render_progress = raytracer
                    .inner
                    .render_progress
                    .load(std::sync::atomic::Ordering::Relaxed)
                    as f32
                    / 100.0;
            }

            egui_state.input.time = Some(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            );

            let events: Vec<Event> = event_pump.poll_iter().collect();
            let egui_wants_keyboard = egui_ctx.wants_keyboard_input();

            for event in events {
                match event {
                    Event::Quit { .. } => break 'running,

                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,

                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } if !egui_wants_keyboard => match keycode {
                        Keycode::W => {
                            raytracer.interrupt_render();
                            raytracer.move_camera(crate::scene::Vector::new(0.0, 0.0, -1.0));
                            needs_render = true;
                        }
                        Keycode::S => {
                            raytracer.interrupt_render();
                            raytracer.move_camera(crate::scene::Vector::new(0.0, 0.0, 1.0));
                            needs_render = true;
                        }
                        Keycode::A => {
                            raytracer.interrupt_render();
                            raytracer.move_camera(crate::scene::Vector::new(-1.0, 0.0, 0.0));
                            needs_render = true;
                        }
                        Keycode::D => {
                            raytracer.interrupt_render();
                            raytracer.move_camera(crate::scene::Vector::new(1.0, 0.0, 0.0));
                            needs_render = true;
                        }
                        Keycode::Q => {
                            raytracer.interrupt_render();
                            raytracer.move_camera(crate::scene::Vector::new(0.0, 1.0, 0.0));
                            needs_render = true;
                        }
                        Keycode::E => {
                            raytracer.interrupt_render();
                            raytracer.move_camera(crate::scene::Vector::new(0.0, -1.0, 0.0));
                            needs_render = true;
                        }

                        Keycode::R => {
                            raytracer.toggle_rendering_mode();
                            needs_render = true;
                        }

                        Keycode::F => {
                            {
                                let mut mode = raytracer.inner.rendering_mode.lock().unwrap();
                                *mode = crate::raytracer::RenderingMode::Full;
                            }
                            raytracer.update_render_settings(
                                gui_state.effective_samples_per_pixel(),
                                gui_state.effective_light_samples(),
                                gui_state.custom_max_bounces,
                            );
                            raytracer.render(false);
                        }

                        Keycode::C => {
                            gui_state.continuous_rendering = !gui_state.continuous_rendering;
                            println!(
                                "Continuous rendering: {}",
                                if gui_state.continuous_rendering {
                                    "ON"
                                } else {
                                    "OFF"
                                }
                            );
                        }

                        _ => {
                            egui_state.process_input(
                                &window,
                                Event::KeyDown {
                                    timestamp: 0,
                                    window_id: window.id(),
                                    keycode: Some(keycode),
                                    scancode: None,
                                    keymod: sdl2::keyboard::Mod::NOMOD,
                                    repeat: false,
                                },
                                &mut egui_painter,
                            );
                        }
                    },

                    // Handle mouse events ourselves to fix Retina coordinate scaling.
                    Event::MouseMotion { x, y, .. } => {
                        egui_state.pointer_pos = egui::pos2(x as f32, y as f32);
                        egui_state
                            .input
                            .events
                            .push(egui::Event::PointerMoved(egui_state.pointer_pos));
                    }
                    Event::MouseButtonDown { mouse_btn, .. } => {
                        let btn = match mouse_btn {
                            sdl2::mouse::MouseButton::Left => Some(egui::PointerButton::Primary),
                            sdl2::mouse::MouseButton::Middle => Some(egui::PointerButton::Middle),
                            sdl2::mouse::MouseButton::Right => Some(egui::PointerButton::Secondary),
                            _ => None,
                        };
                        if let Some(btn) = btn {
                            egui_state.input.events.push(egui::Event::PointerButton {
                                pos: egui_state.pointer_pos,
                                button: btn,
                                pressed: true,
                                modifiers: egui_state.modifiers,
                            });
                        }
                    }
                    Event::MouseButtonUp { mouse_btn, .. } => {
                        let btn = match mouse_btn {
                            sdl2::mouse::MouseButton::Left => Some(egui::PointerButton::Primary),
                            sdl2::mouse::MouseButton::Middle => Some(egui::PointerButton::Middle),
                            sdl2::mouse::MouseButton::Right => Some(egui::PointerButton::Secondary),
                            _ => None,
                        };
                        if let Some(btn) = btn {
                            egui_state.input.events.push(egui::Event::PointerButton {
                                pos: egui_state.pointer_pos,
                                button: btn,
                                pressed: false,
                                modifiers: egui_state.modifiers,
                            });
                        }
                    }

                    _ => {
                        egui_state.process_input(&window, event, &mut egui_painter);
                    }
                }
            }

            // Single egui frame
            egui_ctx.begin_frame(egui_state.input.take());

            let gui_action = gui_state.render(&egui_ctx);

            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(egui::Color32::BLACK))
                .show(&egui_ctx, |ui| {
                    let available = ui.available_size();
                    ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                        render_texture_id,
                        available,
                    )));
                });

            let egui::FullOutput {
                platform_output,
                textures_delta,
                shapes,
                pixels_per_point,
                viewport_output: _,
            } = egui_ctx.end_frame();

            egui_state.process_output(&window, &platform_output);

            // Handle GUI actions
            match gui_action {
                GuiAction::ChangeScene(scene_type) => {
                    raytracer.interrupt_render();
                    let new_scene = match scene_type {
                        SceneType::Dragon => Scene::new_dragon(),
                        SceneType::Teapot => Scene::new_teapot(),
                        SceneType::Specular => Scene::new_specular(),
                        SceneType::Diffuse => Scene::new_diffuse(),
                        SceneType::Triangle => Scene::new_triangle(),
                    };
                    raytracer.set_scene(new_scene);
                    raytracer.inner.pixel_buffer.clear();
                    needs_render = true;
                }
                GuiAction::StartFullRender => {
                    {
                        let mut mode = raytracer.inner.rendering_mode.lock().unwrap();
                        *mode = crate::raytracer::RenderingMode::Full;
                    }
                    raytracer.update_render_settings(
                        gui_state.effective_samples_per_pixel(),
                        gui_state.effective_light_samples(),
                        gui_state.custom_max_bounces,
                    );
                    raytracer.render(false);
                }
                GuiAction::CancelRender => {
                    raytracer.interrupt_render();
                }
                GuiAction::ToggleDebugMode => {
                    raytracer.toggle_rendering_mode();
                    needs_render = true;
                }
                GuiAction::SaveImage => {
                    let rgb_data = Self::rgba_to_rgb(&pixel_snapshot, self.width, self.height);
                    self.save_canvas(&rgb_data);
                }
                GuiAction::ResetCamera => {
                    raytracer.reset_camera();
                    needs_render = true;
                }
                GuiAction::UpdateRenderSettings {
                    samples_per_pixel,
                    light_samples,
                    max_bounces,
                } => {
                    raytracer.update_render_settings(samples_per_pixel, light_samples, max_bounces);
                }
                GuiAction::None => {}
            }

            let paint_jobs = egui_ctx.tessellate(shapes, pixels_per_point);
            egui_painter.paint_jobs(None, textures_delta, paint_jobs);

            window.gl_swap_window();
            thread::sleep(Duration::from_millis(REFRESH_RATE));
        }
    }
}
