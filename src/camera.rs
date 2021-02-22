use crate::linalg::{Matrix, Vector};
use crate::ray::Ray;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub hsize: usize,
    pub vsize: usize,
    pub field_of_view: f64,
    transform_inv: Matrix,
    pixel_size: f64,
    half_width: f64,
    half_height: f64,
}

impl Camera {
    pub fn new(hsize: usize, vsize: usize, field_of_view: f64, transform: Matrix) -> Camera {
        let half_view = (field_of_view / 2.0).tan();
        let aspect = hsize as f64 / vsize as f64;

        let (half_width, half_height) = if aspect >= 1.0 {
            (half_view, half_view / aspect)
        } else {
            (half_view * aspect, half_view)
        };

        let pixel_size = (half_width * 2.0) / hsize as f64;

        Camera {
            hsize,
            vsize,
            field_of_view,
            transform_inv: transform.inverse(),
            pixel_size,
            half_width,
            half_height,
        }
    }

    pub fn ray_at_pixel(self, x: usize, y: usize) -> Ray {
        let xoffset = (x as f64 + 0.5) * self.pixel_size;
        let yoffset = (y as f64 + 0.5) * self.pixel_size;

        let world_x = self.half_width - xoffset;
        let world_y = self.half_height - yoffset;

        let pixel = self.transform_inv * Vector::point(world_x, world_y, -1.0);
        let origin = Vector::point(
            self.transform_inv[0][3],
            self.transform_inv[1][3],
            self.transform_inv[2][3],
        ); // self.transform_inv * Vector::point(0.0, 0.0, 0.0)
        let direction = (pixel - origin).normalize();

        Ray { origin, direction }
    }

    pub fn transform(from: Vector, to: Vector, up: Vector) -> Matrix {
        let forward = (to - from).normalize();

        let up = up.normalize();
        let left = forward.cross(up);
        let up = left.cross(forward);

        #[rustfmt::skip]
        let orientation = Matrix::new([
            [     left.x,     left.y,     left.z, 0.0 ],
            [       up.x,       up.y,       up.z, 0.0 ],
            [ -forward.x, -forward.y, -forward.z, 0.0 ],
            [        0.0,        0.0,        0.0, 1.0 ],
        ]);

        orientation * Matrix::translation(-from.x, -from.y, -from.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;

    use std::f64::consts::PI;
    use test_case::test_case;

    #[test_case(
        Vector::point(0.0, 0.0, 0.0),
        Vector::point(0.0, 0.0, -1.0),
        Vector::vector(0.0, 1.0, 0.0),
        Matrix::id() ;
        "default orientation"
    )]
    #[test_case(
        Vector::point(0.0, 0.0, 0.0),
        Vector::point(0.0, 0.0, 1.0),
        Vector::vector(0.0, 1.0, 0.0),
        Matrix::scaling(-1.0, 1.0, -1.0) ;
        "positive z"
    )]
    #[test_case(
        Vector::point(0.0, 0.0, 8.0),
        Vector::point(0.0, 0.0, 0.0),
        Vector::vector(0.0, 1.0, 0.0),
        Matrix::translation(0.0, 0.0, -8.0) ;
        "moves world"
    )]
    #[test_case(
        Vector::point(1.0, 3.0, 2.0),
        Vector::point(4.0, -2.0, 8.0),
        Vector::vector(1.0, 1.0, 0.0),
        Matrix::new([
            [ -0.50709, 0.50709,  0.67612, -2.36643 ],
            [  0.76772, 0.60609,  0.12122, -2.82843 ],
            [ -0.35857, 0.59761, -0.71714,  0.00000 ],
            [  0.00000, 0.00000,  0.00000,  1.00000 ],
        ]) ;
        "arbitrary"
    )]
    fn view_transform(from: Vector, to: Vector, up: Vector, expected: Matrix) {
        let view_transform = Camera::transform(from, to, up);

        assert!(view_transform.approx(&expected))
    }

    #[test_case(200, 125, 0.01 ; "horizontal image")]
    #[test_case(125, 200, 0.01 ; "vertical image"  )]
    fn pixel_size(hsize: usize, vsize: usize, size: f64) {
        let camera = Camera::new(hsize, vsize, PI / 2.0, Matrix::id());

        assert!(camera.pixel_size.approx(&size))
    }

    #[test_case(
        Matrix::id(),
        100,
        50,
        Vector::point(0.0, 0.0, 0.0),
        Vector::vector(0.0, 0.0, -1.0) ;
        "center"
    )]
    #[test_case(
        Matrix::id(),
        0,
        0,
        Vector::point(0.0, 0.0, 0.0),
        Vector::vector(0.66519, 0.33259, -0.66851) ;
        "corner"
    )]
    #[test_case(
        Matrix::rotation_y(std::f64::consts::PI / 4.0) * Matrix::translation(0.0, -2.0, 5.0),
        100,
        50,
        Vector::point(0.0, 2.0, -5.0),
        Vector::vector(2.0f64.sqrt() / 2.0, 0.0, 2.0f64.sqrt() / -2.0) ;
        "transformed"
    )]
    fn ray_at_pixel(transform: Matrix, x: usize, y: usize, origin: Vector, direction: Vector) {
        let camera = Camera::new(201, 101, PI / 2.0, transform);
        let ray = camera.ray_at_pixel(x, y);

        assert!(ray.origin.approx(&origin) && ray.direction.approx(&direction))
    }
}
