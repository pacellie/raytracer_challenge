use crate::linalg::{Matrix, Vector};

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vector,
    pub direction: Vector,
}

impl Ray {
    pub fn position(self, t: f64) -> Vector {
        self.origin + self.direction * t
    }

    pub fn transform(self, matrix: Matrix) -> Ray {
        Ray {
            origin: matrix * self.origin,
            direction: matrix * self.direction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;

    use test_case::test_case;

    #[test_case( 0.0, Vector::point(2.0, 3.0, 4.0) ; "example 1")]
    #[test_case( 1.0, Vector::point(3.0, 3.0, 4.0) ; "example 2")]
    #[test_case(-1.0, Vector::point(1.0, 3.0, 4.0) ; "example 3")]
    #[test_case( 2.5, Vector::point(4.5, 3.0, 4.0) ; "example 4")]
    fn position(t: f64, point: Vector) {
        let ray = Ray {
            origin: Vector::point(2.0, 3.0, 4.0),
            direction: Vector::vector(1.0, 0.0, 0.0),
        };

        assert!(ray.position(t).approx(&point))
    }

    #[test_case(Matrix::translation(3.0, 4.0, 5.0), Vector::point(4.0, 6.0,  8.0), Vector::vector(0.0, 1.0, 0.0) ; "translation")]
    #[test_case(Matrix::scaling(2.0, 3.0, 4.0)    , Vector::point(2.0, 6.0, 12.0), Vector::vector(0.0, 3.0, 0.0) ; "scaling"    )]
    fn transform(transform: Matrix, origin: Vector, direction: Vector) {
        let ray1 = Ray {
            origin: Vector::point(1.0, 2.0, 3.0),
            direction: Vector::vector(0.0, 1.0, 0.0),
        };
        let ray2 = ray1.transform(transform);

        assert!(ray2.origin.approx(&origin) && ray2.direction.approx(&direction))
    }
}
