extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod primitives;
mod objects;

use primitives::{Point, Ray, Vector};
use objects::{Object, Sphere};

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

struct Scene {
    objects: Vec<Box<dyn Object>>,
}

impl Scene {
    // TODO: don't hardcode scene in here
    fn new() -> Scene {
	let sphere = Sphere {
	    center: Point::new(0.0, 0.0, 10.0),
	    radius: 1.0,
	};
	let boxed_sphere = Box::new(sphere);
	let mut objects = Vec::<Box<dyn Object>>::new();
	objects.push(boxed_sphere);
	Scene {
	    objects: objects
	}
    }
}

fn cast_rays(config: &Config, scene: &Scene) {
    let x_width = 2.0 * f64::tan(config.fov / 2.0);
    let y_width = 2.0 * f64::tan(config.fov / 2.0);

    let x_step = x_width / (config.screen_width as f64);
    let x_start = -x_width / 2.0;
    let y_step = y_width / (config.screen_height as f64);
    let y_start = -y_width / 2.0;

    for i in 0..config.screen_width {
	for j in 0..config.screen_height {
	    let x_component = x_start + x_step * (i as f64);
	    let y_component = y_start + y_step * (j as f64);
	    let vector = Vector::new(x_component, y_component, -1.0);
	    let ray = Ray::new(&config.origin, &vector);
	    for object in scene.objects.iter() {
		if object.intersect(&ray) {
		    println!("{:#?}", ray);
		}
	    }
	}
    }
}

fn init_with_config(config: &Config) -> (sdl2::render::Canvas<sdl2::video::Window>, sdl2::EventPump) {
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

    let event_pump = sdl_context.event_pump().unwrap();
    (canvas, event_pump)
}

fn main() {
    let config = Config::new(800, 800, 90.0);

    let (window, mut event_pump) = init_with_config(&config);
    let scene = Scene::new();

    cast_rays(&config, &scene);

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
