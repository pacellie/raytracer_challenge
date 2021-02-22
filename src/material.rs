use crate::approx::Approx;
use crate::color::Color;
use crate::linalg::{Matrix, Vector};
use crate::noise::Noise;

use std::default::Default;

pub mod consts {
    pub mod transparency {
        pub const VACUUM: f64 = 1.0;
        pub const AIR: f64 = 1.00029;
        pub const WATER: f64 = 1.333;
        pub const GLASS: f64 = 1.52;
        pub const DIAMOND: f64 = 2.417;
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    pub pattern: Pattern,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflective: f64,
    pub transparency: f64,
    pub refractive_index: f64,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            pattern: Pattern::plain(Color::white()),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
            reflective: 0.0,
            transparency: 0.0,
            refractive_index: 1.0,
        }
    }
}

impl Approx<Material> for Material {
    fn approx(&self, other: &Material) -> bool {
        self.pattern.approx(&other.pattern)
            && self.ambient.approx(&other.ambient)
            && self.diffuse.approx(&other.diffuse)
            && self.specular.approx(&other.specular)
            && self.shininess.approx(&other.shininess)
            && self.reflective.approx(&other.reflective)
            && self.transparency.approx(&other.transparency)
            && self.refractive_index.approx(&other.refractive_index)
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub enum Pattern {
    Debug,
    Plain   { color: Color },
    Jitter  { kind: JitterKind, noise: Noise, pattern: Box<Pattern> },
    Mixture { kind: MixtureKind, transform_inv: Matrix, left: Box<Pattern>, right: Box<Pattern> },
}

impl Approx<Pattern> for Pattern {
    fn approx(&self, other: &Pattern) -> bool {
        match (self, other) {
            (Pattern::Debug, Pattern::Debug) => true,
            (Pattern::Plain { color: scolor }, Pattern::Plain { color: ocolor }) => {
                scolor.approx(ocolor)
            }
            (
                Pattern::Jitter {
                    kind: skind,
                    noise: snoise,
                    pattern: spattern,
                },
                Pattern::Jitter {
                    kind: okind,
                    noise: onoise,
                    pattern: opattern,
                },
            ) => skind.approx(okind) && snoise.approx(onoise) && spattern.approx(opattern),
            (
                Pattern::Mixture {
                    kind: skind,
                    transform_inv: stransform_inv,
                    left: sleft,
                    right: sright,
                },
                Pattern::Mixture {
                    kind: okind,
                    transform_inv: otransform_inv,
                    left: oleft,
                    right: oright,
                },
            ) => {
                skind.approx(okind)
                    && stransform_inv.approx(otransform_inv)
                    && sleft.approx(oleft)
                    && sright.approx(oright)
            }
            (_, _) => false,
        }
    }
}

impl Pattern {
    pub fn plain(color: Color) -> Pattern {
        Pattern::Plain { color }
    }

    fn new_jitter(kind: JitterKind, noise: Noise, pattern: Pattern) -> Pattern {
        Pattern::Jitter {
            kind,
            noise,
            pattern: Box::new(pattern),
        }
    }

    pub fn color_jitter(noise: Noise, pattern: Pattern) -> Pattern {
        Pattern::new_jitter(JitterKind::Color, noise, pattern)
    }

    pub fn point_jitter(noise: Noise, pattern: Pattern) -> Pattern {
        Pattern::new_jitter(JitterKind::Point, noise, pattern)
    }

    fn new_mixture(kind: MixtureKind, transform: Matrix, left: Pattern, right: Pattern) -> Pattern {
        Pattern::Mixture {
            kind,
            transform_inv: transform.inverse(),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn blend(transform: Matrix, left: Pattern, right: Pattern) -> Pattern {
        Pattern::new_mixture(MixtureKind::Blend, transform, left, right)
    }

    pub fn checkers(transform: Matrix, left: Pattern, right: Pattern) -> Pattern {
        Pattern::new_mixture(MixtureKind::Checkers, transform, left, right)
    }

    pub fn ring_gradient(transform: Matrix, left: Pattern, right: Pattern) -> Pattern {
        Pattern::new_mixture(MixtureKind::RingGradient, transform, left, right)
    }

    pub fn ring(transform: Matrix, left: Pattern, right: Pattern) -> Pattern {
        Pattern::new_mixture(MixtureKind::Ring, transform, left, right)
    }

    pub fn gradient(transform: Matrix, left: Pattern, right: Pattern) -> Pattern {
        Pattern::new_mixture(MixtureKind::Gradient, transform, left, right)
    }

    pub fn stripes(transform: Matrix, left: Pattern, right: Pattern) -> Pattern {
        Pattern::new_mixture(MixtureKind::Stripes, transform, left, right)
    }

    pub fn color_at(&self, point: Vector) -> Color {
        match self {
            Pattern::Debug => Color {
                r: point.x,
                g: point.y,
                b: point.z,
            },
            Pattern::Plain { color } => *color,
            Pattern::Jitter {
                kind,
                noise,
                pattern,
            } => kind.color_at(point, *noise, pattern),
            Pattern::Mixture {
                kind,
                transform_inv,
                left,
                right,
            } => {
                let point = *transform_inv * point;
                kind.color_at(point, left, right)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JitterKind {
    Color,
    Point,
}

impl Approx<JitterKind> for JitterKind {
    fn approx(&self, other: &JitterKind) -> bool {
        match (self, other) {
            (JitterKind::Color, JitterKind::Color) => true,
            (JitterKind::Point, JitterKind::Point) => true,
            (_, _) => false,
        }
    }
}

impl JitterKind {
    fn color_at(&self, point: Vector, noise: Noise, pattern: &Pattern) -> Color {
        match self {
            JitterKind::Color => {
                let color = pattern.color_at(point);
                let (nr, ng, nb) = noise.jitter_3d(color.r, color.g, color.b);
                Color {
                    r: nr,
                    g: ng,
                    b: nb,
                }
            }
            JitterKind::Point => {
                let (nx, ny, nz) = noise.jitter_3d(point.x, point.y, point.z);
                pattern.color_at(Vector::point(nx, ny, nz))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MixtureKind {
    Blend,
    Checkers,
    RingGradient,
    Ring,
    Gradient,
    Stripes,
}

impl Approx<MixtureKind> for MixtureKind {
    fn approx(&self, other: &MixtureKind) -> bool {
        match (self, other) {
            (MixtureKind::Blend, MixtureKind::Blend) => true,
            (MixtureKind::Checkers, MixtureKind::Checkers) => true,
            (MixtureKind::RingGradient, MixtureKind::RingGradient) => true,
            (MixtureKind::Ring, MixtureKind::Ring) => true,
            (MixtureKind::Gradient, MixtureKind::Gradient) => true,
            (MixtureKind::Stripes, MixtureKind::Stripes) => true,
            (_, _) => false,
        }
    }
}

impl MixtureKind {
    fn color_at(&self, point: Vector, left: &Pattern, right: &Pattern) -> Color {
        match self {
            MixtureKind::Blend => {
                let left = left.color_at(point);
                let right = right.color_at(point);
                left.avg(right)
            }
            MixtureKind::Checkers => {
                let x = point.x.floor() as i32;
                let y = point.y.floor() as i32;
                let z = point.z.floor() as i32;

                if (x + y + z) % 2 == 0 {
                    left.color_at(point)
                } else {
                    right.color_at(point)
                }
            }
            MixtureKind::RingGradient => {
                let distance = (point - Vector::point(0.0, 0.0, 0.0)).magnitude();
                let fraction = distance - distance.floor();

                let left = left.color_at(point);
                let right = right.color_at(point);

                left + ((right - left) * fraction)
            }
            MixtureKind::Ring => {
                if (point.x.powi(2) + point.z.powi(2)).sqrt().floor() as i32 % 2 == 0 {
                    left.color_at(point)
                } else {
                    right.color_at(point)
                }
            }
            MixtureKind::Gradient => {
                let fraction = point.x - point.x.floor();

                let left = left.color_at(point);
                let right = right.color_at(point);

                left + ((right - left) * fraction)
            }
            MixtureKind::Stripes => {
                if point.x.floor() as i32 % 2 == 0 {
                    left.color_at(point)
                } else {
                    right.color_at(point)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;

    use test_case::test_case;

    #[test_case(Matrix::id(), Vector::point( 0.0, 1.0, 0.0), Color::white() ; "constant y example 1" )]
    #[test_case(Matrix::id(), Vector::point( 0.0, 2.0, 0.0), Color::white() ; "constant y example 2" )]
    #[test_case(Matrix::id(), Vector::point( 0.0, 0.0, 1.0), Color::white() ; "constant z example 1" )]
    #[test_case(Matrix::id(), Vector::point( 0.0, 0.0, 2.0), Color::white() ; "constant z example 2" )]
    #[test_case(Matrix::id(), Vector::point( 0.9, 0.0, 0.0), Color::white() ; "alternate x example 1")]
    #[test_case(Matrix::id(), Vector::point( 1.0, 0.0, 0.0), Color::black() ; "alternate x example 2")]
    #[test_case(Matrix::id(), Vector::point(-0.1, 0.0, 0.0), Color::black() ; "alternate x example 3")]
    #[test_case(Matrix::id(), Vector::point(-1.0, 0.0, 0.0), Color::black() ; "alternate x example 4")]
    #[test_case(Matrix::id(), Vector::point(-1.1, 0.0, 0.0), Color::white() ; "alternate x example 5")]
    #[test_case(
        Matrix::id(),
        Matrix::scaling(2.0, 2.0, 2.0).inverse() * Vector::point(1.5, 0.0, 0.0),
        Color::white() ;
        "shape transform"
    )]
    #[test_case(
        Matrix::scaling(2.0, 2.0, 2.0),
        Vector::point(1.5, 0.0, 0.0),
        Color::white() ;
        "pattern transform"
    )]
    #[test_case(
        Matrix::scaling(2.0, 2.0, 2.0),
        Matrix::scaling(2.0, 2.0, 2.0).inverse() * Vector::point(2.5, 0.0, 0.0),
        Color::white() ;
        "shape and pattern transform"
    )]
    fn stripes(transform: Matrix, point: Vector, expected: Color) {
        let pattern = Pattern::stripes(
            transform,
            Pattern::plain(Color::white()),
            Pattern::plain(Color::black()),
        );
        let color = pattern.color_at(point);

        assert!(color.approx(&expected))
    }

    #[test_case(Vector::point(0.0 , 0.0, 0.0), Color::white()               ; "example 1")]
    #[test_case(Vector::point(0.25, 0.0, 0.0), Color::new(0.75, 0.75, 0.75) ; "example 2")]
    #[test_case(Vector::point(0.5 , 0.0, 0.0), Color::new(0.5 , 0.5 , 0.5 ) ; "example 3")]
    #[test_case(Vector::point(0.75, 0.0, 0.0), Color::new(0.25, 0.25, 0.25) ; "example 4")]
    fn gradient(point: Vector, expected: Color) {
        let pattern = Pattern::gradient(
            Matrix::id(),
            Pattern::plain(Color::white()),
            Pattern::plain(Color::black()),
        );
        let color = pattern.color_at(point);

        assert!(color.approx(&expected))
    }

    #[test_case(Vector::point(0.0  , 0.0, 0.0  ), Color::white() ; "example 1")]
    #[test_case(Vector::point(1.0  , 0.0, 0.0  ), Color::black() ; "example 2")]
    #[test_case(Vector::point(0.0  , 0.0, 1.0  ), Color::black() ; "example 3")]
    #[test_case(Vector::point(0.708, 0.0, 0.708), Color::black() ; "example 4")]
    fn ring(point: Vector, expected: Color) {
        let pattern = Pattern::ring(
            Matrix::id(),
            Pattern::plain(Color::white()),
            Pattern::plain(Color::black()),
        );

        let color = pattern.color_at(point);

        assert!(color.approx(&expected))
    }

    #[test_case(Vector::point(0.99, 0.0 , 0.0 ), Color::white() ; "repeat x example 1")]
    #[test_case(Vector::point(1.01, 0.0 , 0.0 ), Color::black() ; "repeat x example 2")]
    #[test_case(Vector::point(0.0 , 0.99, 0.0 ), Color::white() ; "repeat y example 1")]
    #[test_case(Vector::point(0.0 , 1.01, 0.0 ), Color::black() ; "repeat y example 2")]
    #[test_case(Vector::point(0.0 , 0.0 , 0.99), Color::white() ; "repeat z example 1")]
    #[test_case(Vector::point(0.0 , 0.0 , 1.01), Color::black() ; "repeat z example 2")]
    fn checkers(point: Vector, expected: Color) {
        let pattern = Pattern::checkers(
            Matrix::id(),
            Pattern::plain(Color::white()),
            Pattern::plain(Color::black()),
        );
        let color = pattern.color_at(point);

        assert!(color.approx(&expected))
    }
}
