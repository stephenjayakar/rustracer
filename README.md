# rustracer

Screenshot | Table?
----------------------------------------:|:----------------------------
![dragon](/screenshots/dragon.png?raw=true "dragon") | ![specular](/screenshots/specular.png?raw=true "specular")
![global illumination](/screenshots/global_illumination.png?raw=true "global illumination") | ![direct lighting importance](/screenshots/direct_lighting_importance.png?raw=true "direct lighting importance")
![Direct lighting with hemisphere sampling](/screenshots/direct_lighting_hemisphere.png?raw=true "Direct lighting with hemisphere sampling") | ![Lambertian Sphere on top of plane](/screenshots/sphere_on_top_of_plane.png?raw=true "Lambertian Sphere on Plane")

Not a racing program written in rust!

A raytracer that has reached the global illumination stage.

General inspiration was that I could no longer find the latest version of my CS-184 project.

# setup

make sure you have `rust` :Z

## Running with GUI

```sh
cargo run --release -- --high-dpi
```

## Mac OSX

```sh
brew install sdl2
cargo run
```

## Windows

I got it to work by following the directions on the rust sdl2 bindings.

however, I got significantly lower performance on my system that is about 2x faster than my macbook.

plus the high-dpi system doesn't work the same as mac osx so things break.

let me know if you get it :)

# dependencies

* sdl2
* rust

# inspiration

* cs-184 at Berkeley (carries some similar function names / formulas!)
* scratchapixel
* pbrt
* my brain
* my hopes and dreams
