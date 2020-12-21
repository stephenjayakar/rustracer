extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use crate::common::Spectrum;

// Essentially just a wrapper around SDL2's wrapper.
// - hopefully should make it easy to change out the internal implementation.
pub struct Canvas {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> (Canvas, sdl2::EventPump) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rustracer", width / 2, height / 2)
            .allow_highdpi()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        canvas.present();

        let event_pump = sdl_context.event_pump().unwrap();
        return (Canvas { canvas }, event_pump);
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, s: Spectrum) {
        self.canvas.set_draw_color(s.to_sdl2_color());
        self.canvas
            .fill_rect(Rect::new(x as i32, y as i32, 2, 2))
            .expect("failed to draw rectangle");
        // TODO: make this happen on on a regular interval instead of after drawing every pixel.
        self.canvas.present();
    }
}
