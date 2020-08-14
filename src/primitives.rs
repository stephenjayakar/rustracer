pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

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
}
