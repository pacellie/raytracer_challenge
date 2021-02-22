use crate::color::Color;
use crate::linalg::Vector;

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub intensity: Color,
    pub origin: Vector,
}
