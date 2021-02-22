use crate::approx::Approx;
use crate::linalg::Matrix;

use std::fmt;
use std::ops;

#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vector {
    pub fn point(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z, w: 1.0 }
    }

    pub fn vector(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z, w: 0.0 }
    }

    pub fn magnitude(self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn normalize(self) -> Vector {
        let magnitude = self.magnitude();

        Vector {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
            w: 0.0,
        }
    }

    pub fn dot(self, other: Vector) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Vector) -> Vector {
        Vector::vector(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn translate(self, x: f64, y: f64, z: f64) -> Vector {
        Matrix::translation(x, y, z) * self
    }

    pub fn scale(self, x: f64, y: f64, z: f64) -> Vector {
        Matrix::scaling(x, y, z) * self
    }

    pub fn rotate_x(self, r: f64) -> Vector {
        Matrix::rotation_x(r) * self
    }

    pub fn rotate_y(self, r: f64) -> Vector {
        Matrix::rotation_y(r) * self
    }

    pub fn rotate_z(self, r: f64) -> Vector {
        Matrix::rotation_z(r) * self
    }

    pub fn shear(self, x_y: f64, x_z: f64, y_x: f64, y_z: f64, z_x: f64, z_y: f64) -> Vector {
        Matrix::shearing(x_y, x_z, y_x, y_z, z_x, z_y) * self
    }

    pub fn reflect(self, normal: Vector) -> Vector {
        self - normal * (2.0 * self.dot(normal))
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} {} {} {})", self.x, self.y, self.z, self.w)
    }
}

impl Approx<Vector> for Vector {
    fn approx(&self, v: &Vector) -> bool {
        self.x.approx(&v.x) && self.y.approx(&v.y) && self.z.approx(&v.z) && self.w.approx(&v.w)
    }
}

impl ops::Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Vector {
        Vector {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl ops::Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl ops::Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        Vector {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl ops::Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, other: f64) -> Vector {
        Vector {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
            w: self.w * other,
        }
    }
}

impl ops::Div<f64> for Vector {
    type Output = Vector;

    fn div(self, other: f64) -> Vector {
        Vector {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
            w: self.w / other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;

    use test_case::test_case;

    #[test]
    fn addition_point_vector() {
        let v0 = Vector::point(3.0, -2.0, 5.0);
        let v1 = Vector::vector(-2.0, 3.0, 1.0);
        let v2 = Vector::point(1.0, 1.0, 6.0);

        assert!((v0 + v1).approx(&v2))
    }

    #[test_case(
        Vector::point(3.0, 2.0, 1.0),
        Vector::point(5.0, 6.0, 7.0),
        Vector::vector(-2.0, -4.0, -6.0) ;
        "point point"
    )]
    #[test_case(
        Vector::point(3.0, 2.0, 1.0),
        Vector::vector(5.0, 6.0, 7.0),
        Vector::point(-2.0, -4.0, -6.0) ;
        "point vector"
    )]
    #[test_case(
        Vector::vector(3.0, 2.0, 1.0),
        Vector::vector(5.0, 6.0, 7.0),
        Vector::vector(-2.0, -4.0, -6.0) ;
        "vector vector"
    )]
    #[test_case(
        Vector::vector(0.0, 0.0, 0.0),
        Vector::vector(1.0, -2.0, 3.0),
        Vector::vector(-1.0, 2.0, -3.0) ;
        "zero-vector vector"
    )]
    fn subtraction(v0: Vector, v1: Vector, expected: Vector) {
        assert!((v0 - v1).approx(&expected))
    }

    #[test]
    fn negation_vector() {
        let v0 = Vector::vector(1.0, -2.0, 3.0);
        let v1 = Vector::vector(-1.0, 2.0, -3.0);

        assert!((-v0).approx(&v1))
    }

    #[test]
    fn multiplication_vector_scalar() {
        let v0 = Vector::vector(1.0, -2.0, 3.0);
        let v1 = Vector::vector(3.5, -7.0, 10.5);

        assert!((v0 * 3.5).approx(&v1))
    }

    #[test]
    fn division_vector_scalar() {
        let v0 = Vector::vector(1.0, -2.0, 3.0);
        let v1 = Vector::vector(0.5, -1.0, 1.5);

        assert!((v0 / 2.0).approx(&v1))
    }

    #[test]
    fn magnitude_vector() {
        let v = Vector::vector(1.0, -2.0, 3.0);

        assert!(v.magnitude().approx(&14f64.sqrt()))
    }

    #[test]
    fn normalize_vector() {
        let v0 = Vector::vector(1.0, 2.0, 3.0);
        let v1 = Vector::vector(0.26726, 0.53452, 0.80178);

        assert!(v0.normalize().approx(&v1))
    }

    #[test]
    fn magnitude_normalized_vector() {
        let v = Vector::vector(1.0, 2.0, 3.0);

        assert!(v.normalize().magnitude().approx(&1.0))
    }

    #[test]
    fn dot_product() {
        let v0 = Vector::vector(1.0, 2.0, 3.0);
        let v1 = Vector::vector(2.0, 3.0, 4.0);

        assert!(v0.dot(v1).approx(&20.0))
    }

    #[test]
    fn cross_product() {
        let v0 = Vector::vector(1.0, 2.0, 3.0);
        let v1 = Vector::vector(2.0, 3.0, 4.0);
        let v2 = Vector::vector(-1.0, 2.0, -1.0);
        let v3 = Vector::vector(1.0, -2.0, 1.0);

        assert!(v0.cross(v1).approx(&v2) && v1.cross(v0).approx(&v3))
    }

    #[test]
    fn transform_chaining() {
        let p1 = Vector::point(1.0, 0.0, 1.0)
            .rotate_x(std::f64::consts::PI / 2.0)
            .scale(5.0, 5.0, 5.0)
            .translate(10.0, 5.0, 7.0);
        let p2 = Vector::point(15.0, 0.0, 7.0);

        assert!(p1.approx(&p2))
    }

    #[test_case(
        Vector::vector(1.0, -1.0, 0.0),
        Vector::vector(0.0, 1.0, 0.0),
        Vector::vector(1.0, 1.0, 0.0) ;
        "45 degree"
    )]
    #[test_case(
        Vector::vector(0.0, -1.0, 0.0),
        Vector::vector(2.0f64.sqrt() / 2.0, 2.0f64.sqrt() / 2.0, 0.0),
        Vector::vector(1.0, 0.0, 0.0) ;
        "slanted"
    )]
    fn reflection(vector: Vector, normal: Vector, expected: Vector) {
        let reflect = vector.reflect(normal);

        assert!(reflect.approx(&expected));
    }
}
