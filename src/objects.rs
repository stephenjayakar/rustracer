use crate::primitives::{Point, Vector, Ray};

pub trait Object {
    fn intersect(&self, ray: &Ray) -> bool;
}

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
}

impl Sphere {
    fn new(center: Point, radius: f64) -> Sphere {
	Sphere {
	    center: center,
	    radius: radius,
	}
    }
}

impl Object for Sphere {
    // sphere intersection from bheisler
    fn intersect(&self, ray: &Ray) -> bool {
	//Create a line segment between the ray origin and the center of the sphere
        let l: Vector = self.center.sub_vector(ray.origin);
        //Use l as a hypotenuse and find the length of the adjacent side
        let adj2 = l.dot(&ray.direction);
        //Find the length-squared of the opposite side
        //This is equivalent to (but faster than) (l.length() * l.length()) - (adj2 * adj2)
        let d2 = l.dot(&l) - (adj2 * adj2);
        //If that length-squared is less than radius squared, the ray intersects the sphere
        d2 < (self.radius * self.radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sphere_intersection() {
	let origin = Point::new(0.0, 0.0, 0.0);
	let direction = Vector::new(0.0, 0.0, -1.0);
	let ray = Ray::new(&origin, &direction);
	let sphere = Sphere::new(Point::new(0.0, -4.0, 0.0), 2.0);
	assert!(sphere.intersect(&ray));
    }
}
