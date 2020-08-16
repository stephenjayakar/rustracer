extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod primitives;
mod objects;

use primitives::{Point, Ray, Vector};
use objects::{Object, PointLight, Sphere};

const DEFAULT_SCREEN_WIDTH: u32 = 400;
const DEFAULT_SCREEN_HEIGHT: u32 = 400;

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
    lights: Vec<Box<PointLight>>,
}

impl Scene {
    fn new() -> Scene {
	let sphere = Sphere::new(Point::new(0.0, 0.0, -10.0), 2.0);
	let boxed_sphere = Box::new(sphere);
	
	let light = PointLight::new(Point::new(0.0, 4.0, -8.0));
	let boxed_light = Box::new(light);
	    
	let mut objects = Vec::<Box<dyn Object>>::new();
	objects.push(boxed_sphere);
	let mut lights = Vec::new();
	lights.push(boxed_light);
	Scene {
	    objects: objects,
	    lights: lights,
	}
    }
}

struct MyCanvas {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

impl MyCanvas {
    fn draw_pixel(&mut self, x: u32, y: u32, c: Color) {
	self.canvas.set_draw_color(c);
	self.canvas.fill_rect(Rect::new(x as i32, y as i32, 2, 2));
	self.canvas.present();
    }
}

fn init_with_config(config: &Config) -> (MyCanvas, sdl2::EventPump) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rustracer", config.screen_width / 2, config.screen_height / 2)
	.allow_highdpi()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.fill_rect(Rect::new(10, 10, config.screen_width - 10, config.screen_height - 10));
    // canvas.clear();
    canvas.present();
    canvas.set_draw_color(Color::RGB(255, 0, 0));
    let event_pump = sdl_context.event_pump().unwrap();
    (MyCanvas { canvas: canvas }, event_pump)
}

fn main() {
    let config = Config::new(DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT, 90.0);

    let (mut my_canvas, mut event_pump) = init_with_config(&config);
    let scene = Scene::new();

    'running: loop {
    	for event in event_pump.poll_iter() {
            match event {
		Event::KeyDown { keycode: Some(Keycode::R), .. } => {
		    // start rendering
		    // ray casting algorithm
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
			    let vector = Vector::new_normalized(x_component, y_component, -1.0);
			    let ray = Ray::new(&config.origin, &vector);
			    for object in scene.objects.iter() {
				// if camera intersects with object...
				if let Some(d) = object.intersect(&ray) {
				    let intersection_point = ray.get_intersection_point(d);
				    for point_light in scene.lights.iter() {
					let light_direction = point_light.position.sub_point(&intersection_point);
					let light_ray = Ray::new(&intersection_point, &light_direction);
					let mut should_draw = true;
					for obj2 in scene.objects.iter() {
					    if let Some(_) = obj2.intersect(&light_ray) {
						should_draw = false;
						break
					    }
					}
					if should_draw {
					    my_canvas.draw_pixel(i, j, Color::RGB(255, 0, 0));
					}
				    }
				}
			    }
			}
		    }
		}
    		Event::Quit {..} |
    		Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
    		},
    		_ => {}
            }
    	}
    }
}
