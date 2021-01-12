#![allow(dead_code)]
#![feature(const_fn)]

use clap::{Arg, App};

mod canvas;
mod common;
mod raytracer;
mod scene;

use raytracer::Raytracer;
use scene::{Point, Scene};

const DEFAULT_SCREEN_WIDTH: u32 = 600;
const DEFAULT_SCREEN_HEIGHT: u32 = 600;
const DEFAULT_FOV_DEGREES: f64 = 90.0;
const DEFAULT_SAMPLES_PER_PIXEL: u32 = 4;
const DEFAULT_LIGHT_SAMPLES: u32 = 4;
const DEFAULT_MAX_BOUNCES: u32 = 50;

pub struct Config {
    screen_width: u32,
    screen_height: u32,
    fov: f64,
    origin: Point,
	samples_per_pixel: u32,
	light_samples: u32,
	bounces: u32,
	debug: bool,
	high_dpi: bool,
	image_mode: bool,
	single_threaded: bool,
}

impl Config {
	fn from_args() -> Config {
		let matches = App::new("rustracer")
			.arg(Arg::with_name("s")
				 .short("s")
				 .takes_value(true)
				 .help("Sets how many samples per pixel to do"))
			.arg(Arg::with_name("l")
				 .short("l")
				 .takes_value(true)
				 .help("Sets how many light samples to do for one-bounce radiance"))
			.arg(Arg::with_name("b")
				 .short("b")
				 .takes_value(true)
				 .help("Sets the max amount of bounces to simulate"))
			.arg(Arg::with_name("w")
				 .short("w")
				 .takes_value(true)
				 .help("Screen width"))
			.arg(Arg::with_name("h")
				 .short("h")
				 .takes_value(true)
				 .help("Screen height"))
			.arg(Arg::with_name("debug")
				 .short("d")
				 .help("Debug mode, where only intersections are shown"))
			.arg(Arg::with_name("high_dpi")
				 .long("high-dpi")
				 .help("Basically for handling that OSX does display scaling weird"))
			.arg(Arg::with_name("image_mode")
				 .long("image-mode")
				 .help("Mode that doesn't display anything to screen, but rather just dumps it to a png")
				 .short("i"))
			.arg(Arg::with_name("single_threaded")
				 .long("single-threaded")
				 .help("Mode that runs without parallelization"))
			.get_matches();

		let light_samples = matches.value_of("l")
			.map_or(DEFAULT_LIGHT_SAMPLES, |arg| arg.parse().unwrap());
		let samples_per_pixel = matches.value_of("s")
			.map_or(DEFAULT_SAMPLES_PER_PIXEL, |arg| arg.parse().unwrap());
		let bounces = matches.value_of("b")
			.map_or(DEFAULT_MAX_BOUNCES, |arg| arg.parse().unwrap());
		let screen_width = matches.value_of("w")
			.map_or(DEFAULT_SCREEN_WIDTH, |arg| arg.parse().unwrap());
		let screen_height = matches.value_of("h")
			.map_or(DEFAULT_SCREEN_HEIGHT, |arg| arg.parse().unwrap());
		let debug = matches.is_present("debug");
		let high_dpi = matches.is_present("high_dpi");
		let image_mode = matches.is_present("image_mode");
		let single_threaded = matches.is_present("single_threaded");

		Config {
			screen_width,
			screen_height,
			fov: f64::to_radians(DEFAULT_FOV_DEGREES),
			origin: Point::new(0.0, 0.0, 0.0),
			samples_per_pixel,
			light_samples,
			bounces,
			debug,
			high_dpi,
			image_mode,
			single_threaded,
		}
	}
}

fn main() {
    // parse args
	let config = Config::from_args();

    let raytracer = Raytracer::new(config, Scene::new_teapot());
	// raytracer.test(28, 483);
	raytracer.start();
}
