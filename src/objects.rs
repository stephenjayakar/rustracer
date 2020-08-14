use crate::primitives::{Point, Vector, Ray};

pub trait Object {
    fn intersect(&self, ray: &Ray) -> bool;
}

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
}

impl Object for Sphere {
    // sphere intersection from scratchapixel
    fn intersect(&self, ray: &Ray) -> bool {
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
