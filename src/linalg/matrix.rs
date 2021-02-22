use crate::approx::Approx;
use crate::linalg::Vector;

use std::fmt;
use std::ops;

#[derive(Debug, Clone, Copy)]
pub struct Matrix {
    data: [[f64; 4]; 4],
}

const N: usize = 4;

impl Matrix {
    pub fn new(data: [[f64; N]; N]) -> Matrix {
        Matrix { data }
    }

    pub fn id() -> Matrix {
        #[rustfmt::skip]
        let data = [
            [ 1.0, 0.0, 0.0, 0.0 ],
            [ 0.0, 1.0, 0.0, 0.0 ],
            [ 0.0, 0.0, 1.0, 0.0 ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ];
        Matrix::new(data)
    }

    pub fn translation(x: f64, y: f64, z: f64) -> Matrix {
        #[rustfmt::skip]
        let data = [
            [ 1.0, 0.0, 0.0,   x ],
            [ 0.0, 1.0, 0.0,   y ],
            [ 0.0, 0.0, 1.0,   z ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ];

        Matrix::new(data)
    }

    pub fn scaling(x: f64, y: f64, z: f64) -> Matrix {
        #[rustfmt::skip]
        let data = [
            [   x, 0.0, 0.0, 0.0 ],
            [ 0.0,   y, 0.0, 0.0 ],
            [ 0.0, 0.0,   z, 0.0 ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ];

        Matrix::new(data)
    }

    pub fn rotation_x(r: f64) -> Matrix {
        #[rustfmt::skip]
        let data = [
            [ 1.0,     0.0,      0.0, 0.0 ],
            [ 0.0, r.cos(), -r.sin(), 0.0 ],
            [ 0.0, r.sin(),  r.cos(), 0.0 ],
            [ 0.0,     0.0,      0.0, 1.0 ],
        ];

        Matrix::new(data)
    }

    pub fn rotation_y(r: f64) -> Matrix {
        #[rustfmt::skip]
        let data = [
            [  r.cos(), 0.0,  r.sin(), 0.0 ],
            [      0.0, 1.0,      0.0, 0.0 ],
            [ -r.sin(), 0.0,  r.cos(), 0.0 ],
            [      0.0, 0.0,      0.0, 1.0 ],
        ];

        Matrix::new(data)
    }

    pub fn rotation_z(r: f64) -> Matrix {
        #[rustfmt::skip]
        let data = [
            [ r.cos(), -r.sin(), 0.0, 0.0 ],
            [ r.sin(),  r.cos(), 0.0, 0.0 ],
            [     0.0,      0.0, 1.0, 0.0 ],
            [     0.0,      0.0, 0.0, 1.0 ],
        ];

        Matrix::new(data)
    }

    pub fn shearing(x_y: f64, x_z: f64, y_x: f64, y_z: f64, z_x: f64, z_y: f64) -> Matrix {
        #[rustfmt::skip]
        let data = [
            [ 1.0, x_y, x_z, 0.0 ],
            [ y_x, 1.0, y_z, 0.0 ],
            [ z_x, z_y, 1.0, 0.0 ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ];

        Matrix::new(data)
    }

    pub fn translate(self, x: f64, y: f64, z: f64) -> Matrix {
        Matrix::translation(x, y, z) * self
    }

    pub fn scale(self, x: f64, y: f64, z: f64) -> Matrix {
        Matrix::scaling(x, y, z) * self
    }

    pub fn rotate_x(self, r: f64) -> Matrix {
        Matrix::rotation_x(r) * self
    }

    pub fn rotate_y(self, r: f64) -> Matrix {
        Matrix::rotation_y(r) * self
    }

    pub fn rotate_z(self, r: f64) -> Matrix {
        Matrix::rotation_z(r) * self
    }

    pub fn shear(self, x_y: f64, x_z: f64, y_x: f64, y_z: f64, z_x: f64, z_y: f64) -> Matrix {
        Matrix::shearing(x_y, x_z, y_x, y_z, z_x, z_y) * self
    }

    pub fn transpose(self) -> Matrix {
        let mut data = [[0.0f64; N]; N];

        for row in 0..N {
            for col in 0..N {
                data[col][row] = self.data[row][col];
            }
        }

        Matrix::new(data)
    }

    pub fn determinant(self) -> f64 {
        let m = &self.data;

        let s0 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
        let s1 = m[0][0] * m[1][2] - m[1][0] * m[0][2];
        let s2 = m[0][0] * m[1][3] - m[1][0] * m[0][3];
        let s3 = m[0][1] * m[1][2] - m[1][1] * m[0][2];
        let s4 = m[0][1] * m[1][3] - m[1][1] * m[0][3];
        let s5 = m[0][2] * m[1][3] - m[1][2] * m[0][3];

        let c5 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
        let c4 = m[2][1] * m[3][3] - m[3][1] * m[2][3];
        let c3 = m[2][1] * m[3][2] - m[3][1] * m[2][2];
        let c2 = m[2][0] * m[3][3] - m[3][0] * m[2][3];
        let c1 = m[2][0] * m[3][2] - m[3][0] * m[2][2];
        let c0 = m[2][0] * m[3][1] - m[3][0] * m[2][1];

        s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0
    }

    pub fn is_invertible(self) -> bool {
        !self.determinant().approx(&0.0)
    }

    pub fn inverse(self) -> Matrix {
        let m = &self.data;

        let s0 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
        let s1 = m[0][0] * m[1][2] - m[1][0] * m[0][2];
        let s2 = m[0][0] * m[1][3] - m[1][0] * m[0][3];
        let s3 = m[0][1] * m[1][2] - m[1][1] * m[0][2];
        let s4 = m[0][1] * m[1][3] - m[1][1] * m[0][3];
        let s5 = m[0][2] * m[1][3] - m[1][2] * m[0][3];

        let c5 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
        let c4 = m[2][1] * m[3][3] - m[3][1] * m[2][3];
        let c3 = m[2][1] * m[3][2] - m[3][1] * m[2][2];
        let c2 = m[2][0] * m[3][3] - m[3][0] * m[2][3];
        let c1 = m[2][0] * m[3][2] - m[3][0] * m[2][2];
        let c0 = m[2][0] * m[3][1] - m[3][0] * m[2][1];

        let det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;

        assert!(det != 0.0);

        let adj00 = (m[1][1] * c5 - m[1][2] * c4 + m[1][3] * c3) / det;
        let adj02 = (m[3][1] * s5 - m[3][2] * s4 + m[3][3] * s3) / det;
        let adj11 = (m[0][0] * c5 - m[0][2] * c2 + m[0][3] * c1) / det;
        let adj13 = (m[2][0] * s5 - m[2][2] * s2 + m[2][3] * s1) / det;
        let adj20 = (m[1][0] * c4 - m[1][1] * c2 + m[1][3] * c0) / det;
        let adj22 = (m[3][0] * s4 - m[3][1] * s2 + m[3][3] * s0) / det;
        let adj31 = (m[0][0] * c3 - m[0][1] * c1 + m[0][2] * c0) / det;
        let adj33 = (m[2][0] * s3 - m[2][1] * s1 + m[2][2] * s0) / det;

        let adj01 = (-m[0][1] * c5 + m[0][2] * c4 - m[0][3] * c3) / det;
        let adj03 = (-m[2][1] * s5 + m[2][2] * s4 - m[2][3] * s3) / det;
        let adj10 = (-m[1][0] * c5 + m[1][2] * c2 - m[1][3] * c1) / det;
        let adj12 = (-m[3][0] * s5 + m[3][2] * s2 - m[3][3] * s1) / det;
        let adj21 = (-m[0][0] * c4 + m[0][1] * c2 - m[0][3] * c0) / det;
        let adj23 = (-m[2][0] * s4 + m[2][1] * s2 - m[2][3] * s0) / det;
        let adj30 = (-m[1][0] * c3 + m[1][1] * c1 - m[1][2] * c0) / det;
        let adj32 = (-m[3][0] * s3 + m[3][1] * s1 - m[3][2] * s0) / det;

        Matrix::new([
            [adj00, adj01, adj02, adj03],
            [adj10, adj11, adj12, adj13],
            [adj20, adj21, adj22, adj23],
            [adj30, adj31, adj32, adj33],
        ])
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = "".to_string();

        for row in 0..N {
            for col in 0..N {
                result.push_str(&format!("{}", self.data[row][col]));
            }
            result.push('\n');
        }

        write!(f, "{}", result)
    }
}

impl ops::Index<usize> for Matrix {
    type Output = [f64; N];

    fn index(&self, row: usize) -> &[f64; N] {
        &self.data[row]
    }
}

impl ops::IndexMut<usize> for Matrix {
    fn index_mut(&mut self, row: usize) -> &mut Self::Output {
        &mut self.data[row]
    }
}

impl ops::Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        let mut data = [[0.0f64; 4]; 4];

        for row in 0..N {
            for col in 0..N {
                let mut value = 0.0;

                for i in 0..N {
                    value += self[row][i] * other[i][col];
                }

                data[row][col] = value;
            }
        }

        Matrix::new(data)
    }
}

impl ops::Mul<Vector> for Matrix {
    type Output = Vector;

    fn mul(self, other: Vector) -> Vector {
        Vector {
            x: self.data[0][0] * other.x
                + self.data[0][1] * other.y
                + self.data[0][2] * other.z
                + self.data[0][3] * other.w,
            y: self.data[1][0] * other.x
                + self.data[1][1] * other.y
                + self.data[1][2] * other.z
                + self.data[1][3] * other.w,
            z: self.data[2][0] * other.x
                + self.data[2][1] * other.y
                + self.data[2][2] * other.z
                + self.data[2][3] * other.w,
            w: self.data[3][0] * other.x
                + self.data[3][1] * other.y
                + self.data[3][2] * other.z
                + self.data[3][3] * other.w,
        }
    }
}

impl Approx<Matrix> for Matrix {
    fn approx(&self, other: &Matrix) -> bool {
        for row in 0..N {
            for col in 0..N {
                if !self[row][col].approx(&other[row][col]) {
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;

    use test_case::test_case;

    #[test]
    fn construction_indexing() {
        #[rustfmt::skip]
        let data = [
            [  1.0,  2.0,  3.0,  4.0 ],
            [  5.5,  6.5,  7.5,  8.5 ],
            [  9.0, 10.0, 11.0, 12.0 ],
            [ 13.5, 14.5, 15.5, 16.5 ],
        ];

        let m = Matrix::new(data);

        assert!(
            m[0][0].approx(&1.0)
                && m[0][1].approx(&2.0)
                && m[0][2].approx(&3.0)
                && m[0][3].approx(&4.0)
                && m[1][0].approx(&5.5)
                && m[1][1].approx(&6.5)
                && m[1][2].approx(&7.5)
                && m[1][3].approx(&8.5)
                && m[2][0].approx(&9.0)
                && m[2][1].approx(&10.0)
                && m[2][2].approx(&11.0)
                && m[2][3].approx(&12.0)
                && m[3][0].approx(&13.5)
                && m[3][1].approx(&14.5)
                && m[3][2].approx(&15.5)
                && m[3][3].approx(&16.5)
        )
    }

    #[test]
    fn equality() {
        #[rustfmt::skip]
        let data = [
            [  1.0,  2.0,  3.0,  4.0 ],
            [  5.5,  6.5,  7.5,  8.5 ],
            [  9.0, 10.0, 11.0, 12.0 ],
            [ 13.5, 14.5, 15.5, 16.5 ],
        ];

        let m1 = Matrix::new(data.clone());
        let m2 = Matrix::new(data);

        assert!(m1.approx(&m2))
    }

    #[test]
    fn inequality() {
        #[rustfmt::skip]
        let data1 = [
            [  1.0,  2.0,  3.0,  4.0 ],
            [  5.5,  6.5,  7.5,  8.5 ],
            [  9.0, 10.0, 11.0, 12.0 ],
            [ 13.5, 14.5, 15.5, 16.5 ],
        ];

        #[rustfmt::skip]
        let data2 = [
            [  1.0,  2.0,  3.0,  4.0 ],
            [  5.5,  6.5,  7.5,  8.5 ],
            [  9.0, 10.5, 11.0, 12.0 ],
            [ 13.5, 14.5, 15.5, 16.5 ],
        ];

        let m1 = Matrix::new(data1);
        let m2 = Matrix::new(data2);

        assert!(!m1.approx(&m2))
    }

    #[test]
    fn matrix_matrix_multiplication() {
        #[rustfmt::skip]
        let data1 = [
            [ 1.0, 2.0, 3.0, 4.0 ],
            [ 5.0, 6.0, 7.0, 8.0 ],
            [ 9.0, 8.0, 7.0, 6.0 ],
            [ 5.0, 4.0, 3.0, 2.0 ],
        ];

        #[rustfmt::skip]
        let data2 = [
            [ -2.0, 1.0, 2.0, 3.0 ],
            [  3.0, 2.0, 1.0,-1.0 ],
            [  4.0, 3.0, 6.0, 5.0 ],
            [  1.0, 2.0, 7.0, 8.0 ],
        ];

        #[rustfmt::skip]
        let data3 = [
            [ 20.0, 22.0,  50.0,  48.0 ],
            [ 44.0, 54.0, 114.0, 108.0 ],
            [ 40.0, 58.0, 110.0, 102.0 ],
            [ 16.0, 26.0,  46.0,  42.0 ],
        ];

        let m1 = Matrix::new(data1);
        let m2 = Matrix::new(data2);
        let m3 = Matrix::new(data3);

        assert!((m1 * m2).approx(&m3))
    }

    #[test]
    fn matrix_vector_multiplication() {
        #[rustfmt::skip]
        let data = [
            [ 1.0, 2.0, 3.0, 4.0 ],
            [ 2.0, 4.0, 4.0, 2.0 ],
            [ 8.0, 6.0, 4.0, 1.0 ],
            [ 0.0, 0.0, 0.0, 1.0 ],
        ];

        let m = Matrix::new(data);
        let v0 = Vector::point(1.0, 2.0, 3.0);
        let v1 = Vector::point(18.0, 24.0, 33.0);

        assert!((m * v0).approx(&v1))
    }

    #[test]
    fn matrix_identity_multiplication() {
        #[rustfmt::skip]
        let data = [
            [ 0.0, 1.0,  2.0,  4.0 ],
            [ 1.0, 2.0,  4.0,  8.0 ],
            [ 2.0, 4.0,  8.0, 16.0 ],
            [ 4.0, 8.0, 16.0, 32.0 ],
        ];

        let m = Matrix::new(data);

        assert!((m * Matrix::id()).approx(&m))
    }

    #[test]
    fn matrix_vector_identity_multiplication() {
        let v = Vector {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            w: 4.0,
        };

        assert!((Matrix::id() * v).approx(&v))
    }

    #[test]
    fn transpose() {
        #[rustfmt::skip]
        let data1 = [
            [ 0.0, 9.0, 3.0, 0.0 ],
            [ 9.0, 8.0, 0.0, 8.0 ],
            [ 1.0, 8.0, 5.0, 3.0 ],
            [ 0.0, 0.0, 5.0, 8.0 ],
        ];

        #[rustfmt::skip]
        let data2 = [
            [ 0.0, 9.0, 1.0, 0.0 ],
            [ 9.0, 8.0, 8.0, 0.0 ],
            [ 3.0, 0.0, 5.0, 5.0 ],
            [ 0.0, 8.0, 3.0, 8.0 ],
        ];

        let m1 = Matrix::new(data1);
        let m2 = Matrix::new(data2);

        assert!(m1.transpose().approx(&m2))
    }

    #[test]
    fn transpose_identity() {
        assert!(Matrix::id().transpose().approx(&Matrix::id()))
    }

    #[test]
    fn determinant() {
        #[rustfmt::skip]
        let data = [
            [ -2.0, -8.0,  3.0,  5.0 ],
            [ -3.0,  1.0,  7.0,  3.0 ],
            [  1.0,  2.0, -9.0,  6.0 ],
            [ -6.0,  7.0,  7.0, -9.0 ],
        ];

        let m = Matrix::new(data);

        assert!(m.determinant().approx(&-4071.0))
    }

    #[test]
    fn is_invertible() {
        #[rustfmt::skip]
        let data = [
            [ 6.0,  4.0, 4.0,  4.0 ],
            [ 5.0,  5.0, 7.0,  6.0 ],
            [ 4.0, -9.0, 3.0, -7.0 ],
            [ 9.0,  1.0, 7.0, -6.0 ],
        ];

        let m = Matrix::new(data);

        assert!(m.determinant().approx(&-2120.0) && m.is_invertible())
    }

    #[test]
    fn is_not_invertible() {
        #[rustfmt::skip]
        let data = [
            [ -4.0,  2.0, -2.0, -3.0 ],
            [  9.0,  6.0,  2.0,  6.0 ],
            [  0.0, -5.0,  1.0, -5.0 ],
            [  0.0,  0.0,  0.0,  0.0 ],
        ];

        let m = Matrix::new(data);

        assert!(m.determinant().approx(&0.0) && !m.is_invertible())
    }

    #[test_case(
        Matrix::new([
            [ -5.0,  2.0,  6.0, -8.0 ],
            [  1.0, -5.0,  1.0,  8.0 ],
            [  7.0,  7.0, -6.0, -7.0 ],
            [  1.0, -3.0,  7.0,  4.0 ],
        ]),
        Matrix::new([
            [  0.21805,  0.45113,  0.24060, -0.04511 ],
            [ -0.80827, -1.45677, -0.44361,  0.52068 ],
            [ -0.07895, -0.22368, -0.05263,  0.19737 ],
            [ -0.52256, -0.81391, -0.30075,  0.30639 ],
        ]) ;
        "example 1"
    )]
    #[test_case(
        Matrix::new([
            [  8.0, -5.0,  9.0,  2.0 ],
            [  7.0,  5.0,  6.0,  1.0 ],
            [ -6.0,  0.0,  9.0,  6.0 ],
            [ -3.0,  0.0, -9.0, -4.0 ],
        ]),
        Matrix::new([
            [ -0.15385, -0.15385, -0.28205, -0.53846 ],
            [ -0.07692,  0.12308,  0.02564,  0.03077 ],
            [  0.35897,  0.35897,  0.43590,  0.92308 ],
            [ -0.69231, -0.69231, -0.76923, -1.92308 ],
        ]) ;
        "example 2"
    )]
    #[test_case(
        Matrix::new([
            [  9.0,  3.0,  0.0,  9.0 ],
            [ -5.0, -2.0, -6.0, -3.0 ],
            [ -4.0,  9.0,  6.0,  4.0 ],
            [ -7.0,  6.0,  6.0,  2.0 ],
        ]),
        Matrix::new([
            [ -0.04074, -0.07778,  0.14444, -0.22222 ],
            [ -0.07778,  0.03333,  0.36667, -0.33333 ],
            [ -0.02901, -0.14630, -0.10926,  0.12963 ],
            [  0.17778,  0.06667, -0.26667,  0.33333 ],
        ]) ;
        "example 3"
    )]
    fn inverse(matrix: Matrix, expected: Matrix) {
        assert!(matrix.inverse().approx(&expected))
    }

    #[test]
    fn multiplication_inverse_identity() {
        #[rustfmt::skip]
        let data = [
            [  3.0, -9.0,  7.0,  3.0 ],
            [  3.0, -8.0,  2.0, -9.0 ],
            [ -4.0,  4.0,  4.0,  1.0 ],
            [ -6.0,  5.0, -1.0,  1.0 ],
        ];

        let m1 = Matrix::new(data);
        let m2 = m1.inverse();

        assert!((m1 * m2).approx(&Matrix::id()))
    }

    #[test_case(
        Matrix::translation(5.0, -3.0, 2.0),
        Vector::point(-3.0, 4.0, 5.0),
        Vector::point(2.0, 1.0, 7.0) ;
        "translation point"
    )]
    #[test_case(
        Matrix::translation(5.0, -3.0, 2.0).inverse(),
        Vector::point(-3.0, 4.0, 5.0),
        Vector::point(-8.0, 7.0, 3.0) ;
        "inverse translation point"
    )]
    #[test_case(
        Matrix::translation(5.0, -3.0, 2.0),
        Vector::vector(-3.0, 4.0, 5.0),
        Vector::vector(-3.0, 4.0, 5.0) ;
        "inverse translation vector"
    )]
    #[test_case(
        Matrix::scaling(2.0, 3.0, 4.0),
        Vector::point(-4.0, 6.0, 8.0),
        Vector::point(-8.0, 18.0, 32.0) ;
        "scaling point"
    )]
    #[test_case(
        Matrix::scaling(2.0, 3.0, 4.0),
        Vector::vector(-4.0, 6.0, 8.0),
        Vector::vector(-8.0, 18.0, 32.0) ;
        "scaling vector"
    )]
    #[test_case(
        Matrix::scaling(2.0, 3.0, 4.0).inverse(),
        Vector::vector(-4.0, 6.0, 8.0),
        Vector::vector(-2.0, 2.0, 2.0) ;
        "inverse scaling vector"
    )]
    #[test_case(
        Matrix::scaling(-1.0, 1.0, 1.0),
        Vector::point(2.0, 3.0, 4.0),
        Vector::point(-2.0, 3.0, 4.0) ;
        "scaling reflection vector"
    )]
    #[test_case(
        Matrix::rotation_x(std::f64::consts::PI / 4.0),
        Vector::point(0.0, 1.0, 0.0),
        Vector::point(0.0, 2.0f64.sqrt() / 2.0, 2.0f64.sqrt() / 2.0) ;
        "rotation x-axis example 1"
    )]
    #[test_case(
        Matrix::rotation_x(std::f64::consts::PI / 2.0),
        Vector::point(0.0, 1.0, 0.0),
        Vector::point(0.0, 0.0, 1.0) ;
        "rotation x-axis example 2"
    )]
    #[test_case(
        Matrix::rotation_x(std::f64::consts::PI / 4.0).inverse(),
        Vector::point(0.0, 1.0, 0.0),
        Vector::point(0.0, 2.0f64.sqrt() / 2.0, 2.0f64.sqrt() / -2.0) ;
        "inverse rotation x-axis"
    )]
    #[test_case(
        Matrix::rotation_y(std::f64::consts::PI / 4.0),
        Vector::point(0.0, 0.0, 1.0),
        Vector::point(2.0f64.sqrt() / 2.0, 0.0, 2.0f64.sqrt() / 2.0) ;
        "rotation y-axis example 1"
    )]
    #[test_case(
        Matrix::rotation_y(std::f64::consts::PI / 2.0),
        Vector::point(0.0, 0.0, 1.0),
        Vector::point(1.0, 0.0, 0.0) ;
        "rotation y-axis example 2"
    )]
    #[test_case(
        Matrix::rotation_z(std::f64::consts::PI / 4.0),
        Vector::point(0.0, 1.0, 0.0),
        Vector::point(2.0f64.sqrt() / -2.0, 2.0f64.sqrt() / 2.0, 0.0) ;
        "rotation z-axis example 1"
    )]
    #[test_case(
        Matrix::rotation_z(std::f64::consts::PI / 2.0),
        Vector::point(0.0, 1.0, 0.0),
        Vector::point(-1.0, 0.0, 0.0) ;
        "rotation z-axis example 2"
    )]
    #[test_case(
        Matrix::shearing(1.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        Vector::point(2.0, 3.0, 4.0),
        Vector::point(5.0, 3.0, 4.0) ;
        "shearing x y"
    )]
    #[test_case(
        Matrix::shearing(0.0, 1.0, 0.0, 0.0, 0.0, 0.0),
        Vector::point(2.0, 3.0, 4.0),
        Vector::point(6.0, 3.0, 4.0) ;
        "shearing x z"
    )]
    #[test_case(
        Matrix::shearing(0.0, 0.0, 1.0, 0.0, 0.0, 0.0),
        Vector::point(2.0, 3.0, 4.0),
        Vector::point(2.0, 5.0, 4.0) ;
        "shearing y x"
    )]
    #[test_case(
        Matrix::shearing(0.0, 0.0, 0.0, 1.0, 0.0, 0.0),
        Vector::point(2.0, 3.0, 4.0),
        Vector::point(2.0, 7.0, 4.0) ;
        "shearing y z"
    )]
    #[test_case(
        Matrix::shearing(0.0, 0.0, 0.0, 0.0, 1.0, 0.0),
        Vector::point(2.0, 3.0, 4.0),
        Vector::point(2.0, 3.0, 6.0) ;
        "shearing z x"
    )]
    #[test_case(
        Matrix::shearing(0.0, 0.0, 0.0, 0.0, 0.0, 1.0),
        Vector::point(2.0, 3.0, 4.0),
        Vector::point(2.0, 3.0, 7.0) ;
        "shearing z y"
    )]
    fn transform(transform: Matrix, vector: Vector, expected: Vector) {
        assert!((transform * vector).approx(&expected))
    }

    #[test]
    fn transform_sequence() {
        let rotation = Matrix::rotation_x(std::f64::consts::PI / 2.0);
        let scaling = Matrix::scaling(5.0, 5.0, 5.0);
        let translation = Matrix::translation(10.0, 5.0, 7.0);

        let p1 = Vector::point(1.0, 0.0, 1.0);
        let p2 = Vector::point(1.0, -1.0, 0.0);
        let p3 = Vector::point(5.0, -5.0, 0.0);
        let p4 = Vector::point(15.0, 0.0, 7.0);

        assert!(
            (rotation * p1).approx(&p2)
                && (scaling * p2).approx(&p3)
                && (translation * p3).approx(&p4)
        )
    }

    #[test]
    fn transform_chaining() {
        let transform = Matrix::id()
            .rotate_x(std::f64::consts::PI / 2.0)
            .scale(5.0, 5.0, 5.0)
            .translate(10.0, 5.0, 7.0);

        let p1 = Vector::point(1.0, 0.0, 1.0);
        let p2 = Vector::point(15.0, 0.0, 7.0);

        assert!((transform * p1).approx(&p2))
    }
}
