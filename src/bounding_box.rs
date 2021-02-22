use crate::approx::Approx;
use crate::linalg::{Matrix, Vector};
use crate::ray::Ray;
use crate::shape::Geometry;

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vector,
    pub max: Vector,
}

impl Approx<BoundingBox> for BoundingBox {
    fn approx(&self, other: &BoundingBox) -> bool {
        self.min.approx(&other.min) && self.max.approx(&other.max)
    }
}

impl BoundingBox {
    pub fn empty() -> BoundingBox {
        BoundingBox {
            min: Vector::point(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Vector::point(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    pub fn new(min: Vector, max: Vector) -> BoundingBox {
        BoundingBox { min, max }
    }

    pub fn insert(&self, point: Vector) -> BoundingBox {
        let min_x = self.min.x.min(point.x);
        let min_y = self.min.y.min(point.y);
        let min_z = self.min.z.min(point.z);

        let max_x = self.max.x.max(point.x);
        let max_y = self.max.y.max(point.y);
        let max_z = self.max.z.max(point.z);

        BoundingBox {
            min: Vector::point(min_x, min_y, min_z),
            max: Vector::point(max_x, max_y, max_z),
        }
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        self.insert(other.min).insert(other.max)
    }

    pub fn contains(&self, point: Vector) -> bool {
        self.min.x <= point.x
            && point.x <= self.max.x
            && self.min.y <= point.y
            && point.y <= self.max.y
            && self.min.z <= point.z
            && point.z <= self.max.z
    }

    pub fn encloses(&self, other: &BoundingBox) -> bool {
        self.contains(other.min) && self.contains(other.max)
    }

    pub fn transform(&self, matrix: Matrix) -> BoundingBox {
        let p1 = self.min;
        let p2 = Vector::point(self.min.x, self.min.y, self.max.z);
        let p3 = Vector::point(self.min.x, self.max.y, self.min.z);
        let p4 = Vector::point(self.min.x, self.max.y, self.max.z);
        let p5 = Vector::point(self.max.x, self.min.y, self.min.z);
        let p6 = Vector::point(self.max.x, self.min.y, self.max.z);
        let p7 = Vector::point(self.max.x, self.max.y, self.min.z);
        let p8 = self.max;

        let mut bbox = BoundingBox::empty();
        for p in vec![p1, p2, p3, p4, p5, p6, p7, p8] {
            bbox = bbox.insert(matrix * p);
        }

        bbox
    }

    pub fn intersects(&self, ray: Ray) -> bool {
        let (x_t_min, x_t_max) =
            Geometry::intersect_cube_axis(ray.origin.x, ray.direction.x, self.min.x, self.max.x);
        let (y_t_min, y_t_max) =
            Geometry::intersect_cube_axis(ray.origin.y, ray.direction.y, self.min.y, self.max.y);
        let (z_t_min, z_t_max) =
            Geometry::intersect_cube_axis(ray.origin.z, ray.direction.z, self.min.z, self.max.z);

        let t_min = x_t_min.max(y_t_min).max(z_t_min);
        let t_max = x_t_max.min(y_t_max).min(z_t_max);

        t_min <= t_max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;

    use test_case::test_case;

    #[test]
    fn insert() {
        let p1 = Vector::point(-5.0, 2.0, 0.0);
        let p2 = Vector::point(7.0, 0.0, -3.0);

        let bbox = BoundingBox::empty().insert(p1).insert(p2);

        let expected =
            BoundingBox::new(Vector::point(-5.0, 0.0, -3.0), Vector::point(7.0, 2.0, 0.0));

        assert!(bbox.approx(&expected))
    }

    #[test]
    fn union() {
        let bbox1 = BoundingBox::new(Vector::point(-5.0, -2.0, 0.0), Vector::point(7.0, 4.0, 4.0));
        let bbox2 = BoundingBox::new(
            Vector::point(8.0, -7.0, -2.0),
            Vector::point(14.0, 2.0, 8.0),
        );
        let bbox3 = bbox1.union(&bbox2);

        let expected = BoundingBox::new(
            Vector::point(-5.0, -7.0, -2.0),
            Vector::point(14.0, 4.0, 8.0),
        );

        assert!(bbox3.approx(&expected))
    }

    #[test_case(Vector::point( 5.0, -2.0,  0.0), true  ; "example 1")]
    #[test_case(Vector::point(11.0,  4.0,  7.0), true  ; "example 2")]
    #[test_case(Vector::point( 8.0,  1.0,  3.0), true  ; "example 3")]
    #[test_case(Vector::point( 3.0,  0.0,  3.0), false ; "example 4")]
    #[test_case(Vector::point( 8.0, -4.0,  3.0), false ; "example 5")]
    #[test_case(Vector::point( 8.0,  1.0, -1.0), false ; "example 6")]
    #[test_case(Vector::point(13.0,  1.0,  3.0), false ; "example 7")]
    #[test_case(Vector::point( 8.0,  5.0,  3.0), false ; "example 8")]
    #[test_case(Vector::point( 8.0,  1.0,  8.0), false ; "example 9")]
    fn contains(point: Vector, expected: bool) {
        let bbox = BoundingBox::new(Vector::point(5.0, -2.0, 0.0), Vector::point(11.0, 4.0, 7.0));

        assert_eq!(bbox.contains(point), expected)
    }

    #[test_case(BoundingBox::new(Vector::point(5.0, -2.0,  0.0), Vector::point(11.0, 4.0, 7.0)), true  ; "example 1")]
    #[test_case(BoundingBox::new(Vector::point(6.0, -1.0,  1.0), Vector::point(10.0, 3.0, 6.0)), true  ; "example 2")]
    #[test_case(BoundingBox::new(Vector::point(4.0, -3.0, -1.0), Vector::point(10.0, 3.0, 6.0)), false ; "example 3")]
    #[test_case(BoundingBox::new(Vector::point(6.0, -1.0,  1.0), Vector::point(12.0, 5.0, 8.0)), false ; "example 4")]
    fn encloses(inner: BoundingBox, expected: bool) {
        let outer = BoundingBox::new(Vector::point(5.0, -2.0, 0.0), Vector::point(11.0, 4.0, 7.0));

        assert_eq!(outer.encloses(&inner), expected)
    }

    #[test]
    fn transform() {
        let bbox1 = BoundingBox::new(
            Vector::point(-1.0, -1.0, -1.0),
            Vector::point(1.0, 1.0, 1.0),
        );
        let transform = Matrix::rotation_x(std::f64::consts::PI / 4.0)
            * Matrix::rotation_y(std::f64::consts::PI / 4.0);
        let bbox2 = bbox1.transform(transform);

        let expected = BoundingBox::new(
            Vector::point(-1.414213, -1.707106, -1.707106),
            Vector::point(1.414213, 1.707106, 1.707106),
        );

        assert!(bbox2.approx(&expected))
    }

    #[test_case(Vector::point( 5.0,  0.5,  0.0), Vector::vector(-1.0,  0.0,  0.0), true  ; "example 1" )]
    #[test_case(Vector::point(-5.0,  0.5,  0.0), Vector::vector( 1.0,  0.0,  0.0), true  ; "example 2" )]
    #[test_case(Vector::point( 0.5,  5.0,  0.0), Vector::vector( 0.0, -1.0,  0.0), true  ; "example 3" )]
    #[test_case(Vector::point( 0.5, -5.0,  0.0), Vector::vector( 0.0,  1.0,  0.0), true  ; "example 4" )]
    #[test_case(Vector::point( 0.5,  0.0,  5.0), Vector::vector( 0.0,  0.0, -1.0), true  ; "example 5" )]
    #[test_case(Vector::point( 0.5,  0.0, -5.0), Vector::vector( 0.0,  0.0,  1.0), true  ; "example 6" )]
    #[test_case(Vector::point( 0.0,  0.5,  0.0), Vector::vector( 0.0,  0.0,  1.0), true  ; "example 7" )]
    #[test_case(Vector::point(-2.0,  0.0,  0.0), Vector::vector( 2.0,  4.0,  6.0), false ; "example 8" )]
    #[test_case(Vector::point( 0.0, -2.0,  0.0), Vector::vector( 6.0,  2.0,  4.0), false ; "example 9" )]
    #[test_case(Vector::point( 0.0,  0.0, -2.0), Vector::vector( 4.0,  6.0,  2.0), false ; "example 10")]
    #[test_case(Vector::point( 2.0,  0.0,  2.0), Vector::vector( 0.0,  0.0, -1.0), false ; "example 11")]
    #[test_case(Vector::point( 0.0,  2.0,  2.0), Vector::vector( 0.0, -1.0,  0.0), false ; "example 12")]
    #[test_case(Vector::point( 2.0,  2.0,  0.0), Vector::vector(-1.0,  0.0,  0.0), false ; "example 13")]
    fn intersects_cubic(origin: Vector, direction: Vector, expected: bool) {
        let bbox = BoundingBox::new(
            Vector::point(-1.0, -1.0, -1.0),
            Vector::point(1.0, 1.0, 1.0),
        );
        let ray = Ray {
            origin,
            direction: direction.normalize(),
        };
        let intersects = bbox.intersects(ray);

        assert_eq!(intersects, expected)
    }

    #[test_case(Vector::point( 15.0,  1.0,   2.0), Vector::vector(-1.0,  0.0,  0.0), true  ; "example 1" )]
    #[test_case(Vector::point(- 5.0, -1.0,   4.0), Vector::vector( 1.0,  0.0,  0.0), true  ; "example 2" )]
    #[test_case(Vector::point(  7.0,  6.0,   5.0), Vector::vector( 0.0, -1.0,  0.0), true  ; "example 3" )]
    #[test_case(Vector::point(  9.0, -5.0,   6.0), Vector::vector( 0.0,  1.0,  0.0), true  ; "example 4" )]
    #[test_case(Vector::point(  8.0,  2.0,  12.0), Vector::vector( 0.0,  0.0, -1.0), true  ; "example 5" )]
    #[test_case(Vector::point(  6.0,  0.0, - 5.0), Vector::vector( 0.0,  0.0,  1.0), true  ; "example 6" )]
    #[test_case(Vector::point(  8.0,  1.0,   3.5), Vector::vector( 0.0,  0.0,  1.0), true  ; "example 7" )]
    #[test_case(Vector::point(  9.0, -1.0, - 8.0), Vector::vector( 2.0,  4.0,  6.0), false ; "example 8" )]
    #[test_case(Vector::point(  8.0,  3.0, - 4.0), Vector::vector( 6.0,  2.0,  4.0), false ; "example 9" )]
    #[test_case(Vector::point(  9.0, -1.0, - 2.0), Vector::vector( 4.0,  6.0,  2.0), false ; "example 10")]
    #[test_case(Vector::point(  4.0,  0.0,   9.0), Vector::vector( 0.0,  0.0, -1.0), false ; "example 11")]
    #[test_case(Vector::point(  8.0,  6.0, - 1.0), Vector::vector( 0.0, -1.0,  0.0), false ; "example 12")]
    #[test_case(Vector::point( 12.0,  5.0,   4.0), Vector::vector(-1.0,  0.0,  0.0), false ; "example 13")]
    fn intersects_non_cubic(origin: Vector, direction: Vector, expected: bool) {
        let bbox = BoundingBox::new(Vector::point(5.0, -2.0, 0.0), Vector::point(11.0, 4.0, 7.0));
        let ray = Ray {
            origin,
            direction: direction.normalize(),
        };
        let intersects = bbox.intersects(ray);

        assert_eq!(intersects, expected)
    }
}
