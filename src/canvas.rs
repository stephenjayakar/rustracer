extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

extern crate crossbeam_channel;
use crossbeam_channel::{unbounded, Receiver, Sender};

extern crate png;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::common::Spectrum;

const REFRESH_RATE: u64 = 1000 / 10;

/// Mostly contains concurrency primitives to properly wrap
/// around SDL2 context.
pub struct Canvas {
    receiver: Receiver<DrawPixelMessage>,
    sender: Sender<DrawPixelMessage>,
    width: u32,
    height: u32,
    high_dpi: bool,
    image_mode: bool,
}

impl Canvas {
    /// Initializes the canvas with concurrency constructs.
    pub fn new(width: u32, height: u32, high_dpi: bool, image_mode: bool) -> Canvas {
        let (s, r) = unbounded::<DrawPixelMessage>();
        Canvas {
            sender: s,
            receiver: r,
            width,
            height,
            high_dpi,
            image_mode,
        }
    }

    fn new_png_writer(&self) -> png::Writer<BufWriter<File>> {
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
        let writer = encoder.write_header().unwrap();
        writer
    }

    /// Saves the canvas to a png file.  The filename is the '{current UNIX timestamp}.png'.
    fn save_canvas_gui(&self, canvas: &sdl2::render::Canvas<sdl2::video::Window>) {
        let pixels = canvas
            .read_pixels(None, sdl2::pixels::PixelFormatEnum::RGB24)
            .expect("Failed to read pixels from canvas");
        let mut writer = self.new_png_writer();
        writer
            .write_image_data(&pixels)
            .expect("Failed to write canvas to png");
    }

    fn save_canvas(&self) {
        let (w, h) = (self.width as usize, self.height as usize);
        let buffer_size = w * h * 3;
        let mut pixels: Vec<u8> = vec![0; buffer_size];
        for dpm in self.receiver.try_iter() {
            let (x, y, s) = (dpm.x as usize, dpm.y as usize, dpm.s);
            let index = ((y * w) + x) * 3;
            pixels[index] = s.r();
            pixels[index + 1] = s.g();
            pixels[index + 2] = s.b();
        }
        let mut writer = self.new_png_writer();
        writer
            .write_image_data(&pixels)
            .expect("Failed to write canvas to png");
    }

    pub fn start(&self, raytracer_inner: Arc<crate::raytracer::RaytracerInner>) {
        self.start_gui(raytracer_inner);
    }

    /// Starts a new canvas context that takes over the main thread.
    pub fn start_gui(&self, raytracer_inner: Arc<crate::raytracer::RaytracerInner>) {
        // Create a Raytracer from the inner reference
        let raytracer = crate::raytracer::Raytracer { inner: raytracer_inner };
        
        let sdl_context = sdl2::init().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let divider = if self.high_dpi { 2 } else { 1 };
        let width = self.width / divider;
        let height = self.height / divider;
        let window = video_subsystem
            .window("rustracer", width, height)
            .allow_highdpi()
            .build()
            .unwrap();

        // canvas initialization
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        let mut dpmCache = Vec::<DrawPixelMessage>::new();

        // canvas loop
        'running: loop {
            let mut canvasUpdated = false;
            // process draw pixel messages
            for draw_pixel_message in self.receiver.try_iter() {
                dpmCache.push(draw_pixel_message);
                canvasUpdated = true;
            }

            if canvasUpdated {
                for draw_pixel_message in dpmCache.iter() {
                    let (x, y, s) = (
                        draw_pixel_message.x,
                        draw_pixel_message.y,
                        draw_pixel_message.s,
                    );
                    canvas.set_draw_color(s.to_sdl2_color());
                    let square_size = if self.high_dpi { 2 } else { 1 };
                    canvas
                        .fill_rect(Rect::new(x as i32, y as i32, square_size, square_size))
                        .expect("failed to draw rectangle");
                }
                canvas.present();
            }
            
            // process events
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    
                    // Camera movement with WASD
                    Event::KeyDown {
                        keycode: Some(Keycode::W),
                        ..
                    } => {
                        // Interrupt any ongoing render
                        raytracer.interrupt_render();
                        // Move forward (negative Z)
                        raytracer.move_camera(crate::scene::Vector::new(0.0, 0.0, -1.0));
                        raytracer.render();
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::S),
                        ..
                    } => {
                        // Interrupt any ongoing render
                        raytracer.interrupt_render();
                        // Move backward (positive Z)
                        raytracer.move_camera(crate::scene::Vector::new(0.0, 0.0, 1.0));
                        raytracer.render();
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::A),
                        ..
                    } => {
                        // Interrupt any ongoing render
                        raytracer.interrupt_render();
                        // Move left (negative X)
                        raytracer.move_camera(crate::scene::Vector::new(-1.0, 0.0, 0.0));
                        raytracer.render();
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::D),
                        ..
                    } => {
                        // Interrupt any ongoing render
                        raytracer.interrupt_render();
                        // Move right (positive X)
                        raytracer.move_camera(crate::scene::Vector::new(1.0, 0.0, 0.0));
                        raytracer.render();
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => {
                        // Interrupt any ongoing render
                        raytracer.interrupt_render();
                        // Move up (positive Y)
                        raytracer.move_camera(crate::scene::Vector::new(0.0, 1.0, 0.0));
                        raytracer.render();
                    },
                    Event::KeyDown {
                        keycode: Some(Keycode::E),
                        ..
                    } => {
                        // Interrupt any ongoing render
                        raytracer.interrupt_render();
                        // Move down (negative Y)
                        raytracer.move_camera(crate::scene::Vector::new(0.0, -1.0, 0.0));
                        raytracer.render();
                    },
                    
                    // Toggle rendering mode with R
                    Event::KeyDown {
                        keycode: Some(Keycode::R),
                        ..
                    } => {
                        // This will interrupt any current render and toggle the mode
                        raytracer.toggle_rendering_mode();
                        // Start a new render with the new mode
                        raytracer.render();
                    },
                    
                    Event::KeyDown {
                        keycode: Some(Keycode::P),
                        ..
                    } => {
                        canvas.present();
                    },
                    Event::MouseButtonDown { x, y, .. } => println!(
                        "Mouse button down at coordinates ({}, {})",
                        x * divider as i32,
                        y * divider as i32
                    ),
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(REFRESH_RATE));
        }
    }

    pub fn draw_pixel(&self, x: u32, y: u32, s: Spectrum) -> bool {
        // maybe only do this check if debug?
        if x >= self.width || y >= self.height {
            return false;
        }
        self.sender.send(DrawPixelMessage { x, y, s }).unwrap();
        return true;
    }
}

struct DrawPixelMessage {
    x: u32,
    y: u32,
    s: Spectrum,
}
