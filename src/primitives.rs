#[derive(Debug)]
pub struct Ray<'a> {
    pub origin: &'a Point,
    pub direction: &'a Vector,
}

impl<'a> Ray<'a> {
    pub fn new(origin: &'a Point, direction: &'a Vector) -> Ray<'a> {
	Ray {
	    origin: origin,
	    direction: direction,
	}
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
	Point {
	    x: x,
	    y: y,
	    z: z,
	}
    }
}

pub type Vector = Point;

impl Vector {
    pub fn points_to_vector(p1: &Point, p2: &Point) -> Vector {
	Vector {
	    x: p1.x - p2.x,
	    y: p1.y - p2.y,
	    z: p1.z - p2.z
	}
    }

    pub fn dot(&self, other_vector: &Vector) -> f64 {
	self.x * other_vector.x +
	    self.y * other_vector.y +
	    self.z * other_vector.z
    }

    pub fn add_vector(&self, other_vector: &Vector) -> Vector {
	Vector {
	    x: self.x + other_vector.x,
	    y: self.y + other_vector.y,
	    z: self.z + other_vector.z,
	}
    }

    pub fn sub_vector(&self, other_vector: &Vector) -> Vector {
	Vector {
	    x: self.x - other_vector.x,
	    y: self.y - other_vector.y,
	    z: self.z - other_vector.z,
	}
    }

}
