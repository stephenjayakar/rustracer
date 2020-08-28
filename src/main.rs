use std::env;

extern crate sdl2;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod primitives;
mod objects;

use primitives::{Point, Ray, Vector};
use objects::{Object, Plane, PointLight, Sphere};

const DEFAULT_SCREEN_WIDTH: u32 = 1200;
const DEFAULT_SCREEN_HEIGHT: u32 = 1200;
const EPS: f64 = 0.0000001;
const MOVEMENT_DELTA: f64 = 2.0;

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
    lights: Vec<PointLight>,
}

struct RayIntersection {
    distance: f64,
    object_index: usize,
}

impl Scene {
    fn new() -> Scene {
	let sphere = Sphere::new(Point::new(0.0, 0.0, -5.0), 2.0);
	let boxed_sphere = Box::new(sphere);
  	let mut objects = Vec::<Box<dyn Object>>::new();
	objects.push(boxed_sphere);
	let boxed_plane = Box::new(
	    Plane::new(
		Point::new(0.0, 0.0, -19.0),
		Vector::new(0.0, 0.0, 1.0),
	    )
	);
	objects.push(boxed_plane);

	let light = PointLight::new(Point::new(2.0, 2.0, 2.0));

	let mut lights = Vec::new();
	lights.push(light);
	Scene {
	    objects,
	    lights,
	}
    }

    // returns the closest intersected objects' distance
    fn ray_intersection(&self, ray: &Ray) -> Option<RayIntersection> {
	let mut min_dist = f64::INFINITY;
	let mut min_index = None;
	for (i, object) in self.objects.iter().enumerate() {
	    if let Some(d) = object.intersect(ray) {
		if d < min_dist {
		    min_dist = d;
		    min_index = Some(i);
		}
	    }
	}
	match min_index {
	    Some(i) => Some(RayIntersection { distance: min_dist, object_index: i }),
	    None => None,
	}
    }
}

struct MyCanvas {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

impl MyCanvas {
    fn draw_pixel(&mut self, x: u32, y: u32, c: Color) {
	self.canvas.set_draw_color(c);
	self.canvas.fill_rect(Rect::new(x as i32, y as i32, 2, 2)).expect("failed to draw rectangle");
    }

    fn present(&mut self) {
	self.canvas.present();
    }
}

struct Raytracer {
    config: Config,
    my_canvas: MyCanvas,
    scene: Scene,
}

impl Raytracer {
    fn new(config: Config, scene: Scene) -> (Raytracer, sdl2::EventPump) {
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();

	let window = video_subsystem.window("rustracer", config.screen_width / 2, config.screen_height / 2)
	    .allow_highdpi()
            .build()
            .unwrap();

	let mut canvas = window.into_canvas().build().unwrap();

	canvas.set_draw_color(Color::RGB(255, 255, 255));
	canvas.clear();
	canvas.present();

	let event_pump = sdl_context.event_pump().unwrap();
	let my_canvas = MyCanvas { canvas };
	(Raytracer {
	    config,
	    my_canvas,
	    scene,
	}, event_pump)
    }

    fn render (&mut self) {
	// start rendering
	// ray casting algorithm
	let x_width = 2.0 * f64::tan(self.config.fov / 2.0);
	let y_width = 2.0 * f64::tan(self.config.fov / 2.0);

	let x_step = x_width / (self.config.screen_width as f64);
	let x_start = -x_width / 2.0;
	let y_step = y_width / (self.config.screen_height as f64);
	let y_start = -y_width / 2.0;

	for i in 0..self.config.screen_width {
    	    for j in 0..self.config.screen_height {
    		let x_component = x_start + x_step * (i as f64);
    		let y_component = y_start + y_step * (j as f64);
    		let vector = Vector::new(x_component, y_component, -1.0);
    		let ray = Ray::new(self.config.origin, vector);
    		if let Some(ray_intersection) = self.scene.ray_intersection(&ray) {
    		    let mut intersection_point = ray.get_intersection_point(ray_intersection.distance);
		    // bumping the point a little out of the object to prevent self-collision
		    let surface_normal = self.scene.objects.get(ray_intersection.object_index).unwrap().surface_normal(intersection_point);
		    intersection_point = intersection_point + surface_normal.scale(EPS);

    		    for point_light in self.scene.lights.iter() {
    			let light_direction = (point_light.position - intersection_point).normalized();
    			let light_ray = Ray::new(intersection_point, light_direction);

    			if self.scene.ray_intersection(&light_ray).is_none() {
			    // lambertian code
			    let intensity = f64::abs(light_direction.dot(surface_normal));
			    let color = (intensity * 200.0) as u8;
    			    self.my_canvas.draw_pixel(i, j, Color::RGB(color, color, color));
    			} else {
    			    self.my_canvas.draw_pixel(i, j, Color::RGB(0, 0, 0));
    			}
    		    }
    		}
    	    }
	}
	self.my_canvas.present();
    }
}

fn parse_args(args: Vec<String>) -> Option<(u32, u32)> {
    match args.len() {
	3 => {
	    let width = args.get(1).unwrap().parse().expect("passed in invalid width");
	    let height = args.get(1).unwrap().parse().expect("passed in invalid width");
	    Some((width, height))
	},
	_ => {
	    None
	}
    }
}

fn main() {
    // parse args
    let args: Vec<String> = env::args().collect();
    let (screen_width, screen_height) = match parse_args(args) {
	None => (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT),
	Some((width, height)) => (width, height),
    };

    // set up raytracer
    let config = Config::new(screen_width, screen_height, 90.0);
    let (mut raytracer, mut event_pump) = Raytracer::new(config, Scene::new());

    // event loop
    'running: loop {
    	for event in event_pump.poll_iter() {
            match event {
		Event::KeyDown { keycode: Some(Keycode::D), .. } => {
		    raytracer.scene.lights.get_mut(0).unwrap().position.y += MOVEMENT_DELTA;
		}
		Event::KeyDown { keycode: Some(Keycode::A), .. } => {
		    raytracer.scene.lights.get_mut(0).unwrap().position.y -= MOVEMENT_DELTA;
		}
		Event::KeyDown { keycode: Some(Keycode::S), .. } => {
		    raytracer.scene.lights.get_mut(0).unwrap().position.x += MOVEMENT_DELTA;
		}
		Event::KeyDown { keycode: Some(Keycode::W), .. } => {
		    raytracer.scene.lights.get_mut(0).unwrap().position.x -= MOVEMENT_DELTA;
		}
		Event::KeyUp { keycode: Some(_), .. } |
		    Event::KeyDown { keycode: Some(Keycode::R), .. } => {
		    raytracer.render()
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
