extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

extern crate crossbeam_channel;
use crossbeam_channel::{unbounded, Sender};

use std::thread;
use std::time::Duration;

use crate::common::Spectrum;

// Essentially just a wrapper around SDL2's wrapper.
// - hopefully should make it easy to change out the internal implementation.
pub struct Canvas {
    sender: Sender<DrawPixelMessage>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        // TODO: make this bounded for performance reasons
        let (s, r) = unbounded::<DrawPixelMessage>();
        thread::spawn(move || {
            let sdl_context = sdl2::init().unwrap();
            let mut event_pump = sdl_context.event_pump().unwrap();
            let video_subsystem = sdl_context.video().unwrap();
            let window = video_subsystem
                .window("rustracer", width / 2, height / 2)
                .allow_highdpi()
                .build()
                .unwrap();

            // canvas initialization
            let mut canvas = window.into_canvas().build().unwrap();
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.clear();
            canvas.present();

            // canvas loop
            'running: loop {
                // process events
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'running,
                        _ => {}
                    }
                }
                // process draw pixel messages
                for draw_pixel_message in r.try_iter() {
                    let (x, y, s) = (
                        draw_pixel_message.x,
                        draw_pixel_message.y,
                        draw_pixel_message.s,
                    );
                    canvas.set_draw_color(s.to_sdl2_color());
                    canvas
                        .fill_rect(Rect::new(x as i32, y as i32, 2, 2))
                        .expect("failed to draw rectangle");
                }
                canvas.present();
                thread::sleep(Duration::from_secs(1));
            }
        });
        Canvas { sender: s }
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, s: Spectrum) {
        self.sender.send(DrawPixelMessage { x, y, s }).unwrap();
    }
}

struct DrawPixelMessage {
    x: u32,
    y: u32,
    s: Spectrum,
}
