use crate::primitives::{Point, Vector, Ray};

pub trait Object {
    fn intersect(&self, ray: &Ray) -> Option<f64>;
    fn surface_normal(&self, point: Point) -> Vector;
    // fn bsdf(&self, wi: Vector, wo: Vector) -> Spectrum;
}

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
}

pub struct Plane {
    pub point: Point,
    pub normal: Vector,
}

impl Sphere {
    pub fn new(center: Point, radius: f64) -> Sphere {
	Sphere {
	    center,
	    radius,
	}
    }
}

impl Object for Sphere {
    // sphere intersection from bheisler
    // returns intersection distance
    fn intersect(&self, ray: &Ray) -> Option<f64> {
	let l: Vector = self.center - ray.origin;
	let adj = l.dot(ray.direction);
	let d2 = l.dot(l) - (adj * adj);
	let radius2 = self.radius * self.radius;
	if d2 > radius2 {
            return None;
	}
	let thc = (radius2 - d2).sqrt();
	let t0 = adj - thc;
	let t1 = adj + thc;

	if t0 < 0.0 && t1 < 0.0 {
            return None;
	}

	let distance = if t0 < t1 { t0 } else { t1 };
	Some(distance)
    }

    fn surface_normal(&self, point: Point) -> Vector {
	(point - self.center).normalized()
    }
}

impl Plane {
    pub fn new(point: Point, normal: Vector) -> Plane {
	Plane {
	    point,
	    normal,
	}
    }
}

impl Object for Plane {
    fn intersect(&self, ray: &Ray) -> Option<f64> {
	let d = (self.point - ray.origin).dot(self.normal) /
	    ray.direction.dot(self.normal);
	if d > 0.0 {
	    Some(d)
	}
	else {
	    None
	}
    }
    fn surface_normal(&self, _point: Point) -> Vector {
	self.normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sphere_intersection() {
	let origin = Point::new(0.0, 0.0, 0.0);
	let direction = Vector::new_normalized(0.0, 0.0, -1.0);
	let ray = Ray::new(&origin, &direction);
	let sphere = Sphere::new(Point::new(0.0, 0.0, -4.0), 2.0);
	assert!(sphere.intersect(&ray) != None);

	let vector_that_misses = Vector::new_normalized(3.0, 0.0, -4.0);
	let ray_that_misses = Ray::new(&origin, &vector_that_misses);
	assert!(sphere.intersect(&ray_that_misses) == None);
    }
}
