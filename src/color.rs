use crate::approx::Approx;

use std::fmt;
use std::ops;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Color {
        Color { r, g, b }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }

    pub fn white() -> Color {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }

    pub fn black() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }

    pub fn clamp(self) -> (u8, u8, u8) {
        let clamp = |x: f64| (x.min(1.0).max(0.0) * 255.0).round() as u8;

        (clamp(self.r), clamp(self.g), clamp(self.b))
    }

    pub fn avg(self, other: Color) -> Color {
        (self + other) * 0.5
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{},{})", self.r, self.g, self.b)
    }
}

impl Approx<Color> for Color {
    fn approx(&self, c: &Color) -> bool {
        self.r.approx(&c.r) && self.g.approx(&c.g) && self.b.approx(&c.b)
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl ops::AddAssign for Color {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl ops::Sub<Color> for Color {
    type Output = Color;

    fn sub(self, other: Color) -> Color {
        Color {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
        }
    }
}

impl ops::Mul<f64> for Color {
    type Output = Color;

    fn mul(self, other: f64) -> Color {
        Color {
            r: self.r * other,
            g: self.g * other,
            b: self.b * other,
        }
    }
}

impl ops::Mul<Color> for Color {
    type Output = Color;

    fn mul(self, other: Color) -> Color {
        Color {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;

    #[test]
    fn adding_colors() {
        let c0 = Color::new(0.9, 0.6, 0.75);
        let c1 = Color::new(0.7, 0.1, 0.25);
        let c2 = Color::new(1.6, 0.7, 1.0);

        assert!((c0 + c1).approx(&c2))
    }

    #[test]
    fn subtracting_colors() {
        let c0 = Color::new(0.9, 0.6, 0.75);
        let c1 = Color::new(0.7, 0.1, 0.25);
        let c2 = Color::new(0.2, 0.5, 0.5);

        assert!((c0 - c1).approx(&c2))
    }

    #[test]
    fn multiplying_color_scalar() {
        let c0 = Color::new(0.2, 0.3, 0.4);
        let c1 = Color::new(0.4, 0.6, 0.8);

        assert!((c0 * 2.0).approx(&c1))
    }

    #[test]
    fn multiplying_colors() {
        let c0 = Color::new(1.0, 0.2, 0.4);
        let c1 = Color::new(0.9, 1.0, 0.1);
        let c2 = Color::new(0.9, 0.2, 0.04);

        assert!((c0 * c1).approx(&c2))
    }
}
