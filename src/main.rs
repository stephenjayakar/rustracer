extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::Duration;
use std::vec::Vec;

struct Config {
    screen_width: u32,
    screen_height: u32,
}

struct Scene {
    objects: Vec<Box<Object>>,
}

struct Camera {
    // for now, direction is assumed to be -z
    // direction: Vector,
    fov: f64,
    position: Point,
}

trait Object {
    fn intersect(&self, ray: Ray) -> bool;
}

struct Ray {
    origin: Point,
    direction: Vector,
}

struct Sphere {
    center: Point,
    color: Color,
    radius: f64,
}

impl Object for Sphere {
    // sphere intersection from scratchapixel
    fn intersect(&self, ray: Ray) -> bool {
	let radius2 = self.radius.powi(2);
	let (mut t0, mut t1) = (0.0, 0.0);
	let L = Vector::points_to_vector(&ray.origin, &self.center);
	let tca = L.dot(&ray.direction);
        let d2: f64 = L.dot(&L) - tca * tca;
        if d2 > radius2 {
	    return false;
	}
        let thc: f64 = f64::sqrt(radius2 - d2);
        t0 = tca - thc;
        t1 = tca + thc;
        if t0 > t1 {
	    let temp: f64 = t0;
	    t0 = t1;
	    t1 = temp;
	}

        if t0 < 0.0 {
            t0 = t1; // if t0 is negative, let's use t1 instead
            if t0 < 0.0 {
		return false; // both t0 and t1 are negative
	    }
        }

	// TODO: use this to actually calculate distance to sphere
        // t = t0;

        return true;
    }
}

struct Point {
    x: f64,
    y: f64,
    z: f64,
}

type Vector = Point;

impl Vector {
    fn points_to_vector(p1: &Point, p2: &Point) -> Vector {
	Vector {
	    x: p1.x - p2.x,
	    y: p1.y - p2.y,
	    z: p1.z - p2.z
	}
    }

    fn dot(&self, other_vector: &Vector) -> f64 {
	self.x * other_vector.x +
	    self.y * other_vector.y +
	    self.z * other_vector.z
    }
}

// convert pixel coordinates to direction vector
fn px_coords_to_direction(x: u32,
			  y: u32,
			  config: &Config,
			  camera: &Camera) {
    let z_vec = -1.0;
    
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rustracer", 800, 800)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
    	for event in event_pump.poll_iter() {
            match event {
    		Event::Quit {..} |
    		Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
    		},
    		_ => {}
            }
    	}
    }
}
