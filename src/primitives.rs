#[derive(Debug)]
pub struct Ray<'a> {
    pub origin: &'a Point,
    pub direction: &'a Vector,
}

impl<'a> Ray<'a> {
    pub fn new(origin: &'a Point, direction: &'a Vector) -> Ray<'a> {
	Ray {
	    origin,
	    direction,
	}
    }

    pub fn get_intersection_point(&self, scalar: f64) -> Point {
	let scaled_vector = self.direction.scale(scalar);
	self.origin.add_vector(&scaled_vector) // add EPS to prevent floating point errors?
    }
}

#[derive(Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Point {
	Point { x, y, z }
    }

    pub fn add_vector(&self, vector: &Vector) -> Point {
	Point::new(
	    self.x + vector.x,
	    self.y + vector.y,
	    self.z + vector.z,
	)
    }

    pub fn sub_point(&self, other: &Point) -> Vector {
	Vector::new(
	    self.x - other.x,
	    self.y - other.y,
	    self.z - other.z,
	)
    }
}

#[derive(Debug)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
	Vector { x, y, z }
    }


    pub fn new_normalized(x: f64, y: f64, z: f64) -> Vector {
	let norm = f64::sqrt(f64::powi(x, 2) + f64::powi(y, 2) + f64::powi(z, 2));
	Vector::new(x / norm, y / norm, z / norm)
    }

    pub fn points_to_vector(p1: &Point, p2: &Point) -> Vector {
	Vector::new(
	    p1.x - p2.x,
	    p1.y - p2.y,
	    p1.z - p2.z,
	)
    }

    pub fn dot(&self, other_vector: &Vector) -> f64 {
	self.x * other_vector.x +
	    self.y * other_vector.y +
	    self.z * other_vector.z
    }

    pub fn add_vector(&self, other_vector: &Vector) -> Vector {
	Vector::new(
	    self.x + other_vector.x,
	    self.y + other_vector.y,
	    self.z + other_vector.z,
	)
    }

    pub fn sub_vector(&self, other_vector: &Vector) -> Vector {
	Vector::new(
	    self.x - other_vector.x,
	    self.y - other_vector.y,
	    self.z - other_vector.z,
	)
    }

    pub fn scale(&self, scalar: f64) -> Vector {
	Vector::new(
	    scalar * self.x,
	    scalar * self.y,
	    scalar * self.z,
	)
    }
}
