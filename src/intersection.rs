use crate::config::EPSILON;
use crate::linalg::Vector;
use crate::ray::Ray;
use crate::shape::Shape;

use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct State<'a> {
    pub t: f64,
    pub shape: &'a Shape,
    pub point: Vector,
    pub over_point: Vector,
    pub under_point: Vector,
    pub eye: Vector,
    pub normal: Vector,
    pub reflect: Vector,
    pub inside: bool,
    pub n1: f64,
    pub n2: f64,
    pub reflectance: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Intersection<'a> {
    pub t: f64,
    pub shape: &'a Shape,
    pub u: Option<f64>,
    pub v: Option<f64>,
}

impl<'a> Intersection<'a> {
    pub fn prepare_state(self, ray: Ray, intersections: &Vec<Intersection>) -> State<'a> {
        let t = self.t;
        let shape = self.shape;

        let point = ray.position(t);
        let eye = -ray.direction;

        let mut normal = shape.normal(point, self.u, self.v);
        let mut inside = false;

        if normal.dot(eye) < 0.0 {
            normal = -normal;
            inside = true;
        }

        let over_point = point + (normal * EPSILON);
        let under_point = point - (normal * EPSILON);

        let reflect = ray.direction.reflect(normal);

        let mut shapes: Vec<&Shape> = vec![];
        let mut set: HashSet<&Shape> = HashSet::new();
        let mut n1 = 1.0;
        let mut n2 = 1.0;

        for intersection in intersections {
            // `self` assumed to be the hit of `intersections`
            if self == *intersection {
                n1 = shapes
                    .last()
                    .map(|s| s.material.refractive_index)
                    .unwrap_or(1.0);
            }

            if set.contains(intersection.shape) {
                shapes = shapes
                    .into_iter()
                    .filter(|shape| *shape != intersection.shape)
                    .collect();

                set.remove(intersection.shape);
            } else {
                shapes.push(intersection.shape);
                set.insert(intersection.shape);
            }

            // `self` assumed to be the hit of `intersections`
            if self == *intersection {
                n2 = shapes
                    .last()
                    .map(|s| s.material.refractive_index)
                    .unwrap_or(1.0);
            }
        }

        let reflectance = Intersection::schlick(eye, normal, n1, n2);

        State {
            t,
            shape,
            point,
            over_point,
            under_point,
            eye,
            normal,
            reflect,
            inside,
            n1,
            n2,
            reflectance,
        }
    }

    fn schlick(eye: Vector, normal: Vector, n1: f64, n2: f64) -> f64 {
        let mut cos = eye.dot(normal);

        if n1 > n2 {
            let n = n1 / n2;
            let sin2_t = n.powi(2) * (1.0 - cos.powi(2));
            if sin2_t > 1.0 {
                return 1.0;
            }
            cos = (1.0 - sin2_t).sqrt();
        }

        let r0 = ((n1 - n2) / (n1 + n2)).powi(2);

        r0 + (1.0 - r0) * (1.0 - cos).powi(5)
    }

    pub fn sort(intersections: &mut Vec<Intersection>) {
        intersections.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap())
    }

    pub fn hit(intersections: &Vec<Intersection<'a>>) -> Option<Intersection<'a>> {
        intersections
            .iter()
            .find(|intersection| intersection.t >= 0.0)
            .map(|intersection| *intersection)
    }
}

impl PartialEq for Intersection<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && self.shape == other.shape
    }
}

impl Eq for Intersection<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;
    use crate::linalg::Matrix;
    use crate::material::consts::transparency::GLASS;
    use crate::material::Material;
    use crate::shape::ShapeArgs;

    #[test]
    fn aggregating_intersections() {
        let sphere = Shape::sphere(ShapeArgs::default());

        let i1 = Intersection {
            t: 1.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: 2.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let mut is = vec![i2, i1];
        Intersection::sort(&mut is);

        assert!(
            is.len() == 2
                && is[0].t == 1.0
                && is[0].shape == &sphere
                && is[1].t == 2.0
                && is[1].shape == &sphere
        )
    }

    #[test]
    fn hit_all_positive_t() {
        let sphere = Shape::sphere(ShapeArgs::default());

        let i1 = Intersection {
            t: 1.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: 2.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let mut is = vec![i2, i1];
        Intersection::sort(&mut is);

        let hit = Intersection::hit(&is);

        assert!(hit.is_some() && hit.unwrap().t == 1.0 && hit.unwrap().shape == &sphere)
    }

    #[test]
    fn hit_some_negative_t() {
        let sphere = Shape::sphere(ShapeArgs::default());

        let i1 = Intersection {
            t: -1.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: 1.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let is = vec![i2, i1];
        let hit = Intersection::hit(&is);

        assert!(hit.is_some() && hit.unwrap().t == 1.0 && hit.unwrap().shape == &sphere)
    }

    #[test]
    fn hit_lowest_nonnegative() {
        let sphere = Shape::sphere(ShapeArgs::default());

        let i1 = Intersection {
            t: 5.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: 7.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let i3 = Intersection {
            t: -3.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let i4 = Intersection {
            t: 2.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let mut is = vec![i1, i2, i3, i4];
        Intersection::sort(&mut is);

        let hit = Intersection::hit(&is);

        assert!(hit.is_some() && hit.unwrap().t == 2.0 && hit.unwrap().shape == &sphere)
    }

    #[test]
    fn hit_all_negative_t() {
        let sphere = Shape::sphere(ShapeArgs::default());

        let i1 = Intersection {
            t: -2.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: -1.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let is = vec![i2, i1];

        let hit = Intersection::hit(&is);

        assert!(hit.is_none())
    }

    #[test]
    fn precompute_state_intersection() {
        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let sphere = Shape::sphere(ShapeArgs::default());

        let i = Intersection {
            t: 4.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let state = i.prepare_state(ray, &vec![]);

        assert!(
            state.t.approx(&i.t)
                && state.shape == i.shape
                && state.point.approx(&Vector::point(0.0, 0.0, -1.0))
                && state.eye.approx(&Vector::vector(0.0, 0.0, -1.0))
                && state.normal.approx(&Vector::vector(0.0, 0.0, -1.0))
        )
    }

    #[test]
    fn hit_intersection_outside() {
        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let sphere = Shape::sphere(ShapeArgs::default());

        let i = Intersection {
            t: 4.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let state = i.prepare_state(ray, &vec![]);

        assert!(!state.inside)
    }

    #[test]
    fn hit_intersection_inside() {
        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 0.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let sphere = Shape::sphere(ShapeArgs::default());

        let i = Intersection {
            t: 1.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let state = i.prepare_state(ray, &vec![]);

        assert!(
            state.point.approx(&Vector::point(0.0, 0.0, 1.0))
                && state.eye.approx(&Vector::vector(0.0, 0.0, -1.0))
                && state.normal.approx(&Vector::vector(0.0, 0.0, -1.0))
                && state.inside
        )
    }

    #[test]
    fn hit_offset_over_point() {
        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let sphere = Shape::sphere(ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, 1.0),
            ..ShapeArgs::default()
        });

        let intersection = Intersection {
            t: 5.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let state = intersection.prepare_state(ray, &vec![]);

        assert!(state.over_point.z < -EPSILON / 2.0 && state.over_point.z < state.point.z)
    }

    #[test]
    fn precomputing_reflection_vector() {
        let shape = Shape::plane(ShapeArgs::default());

        let ray = Ray {
            origin: Vector::point(0.0, 1.0, -1.0),
            direction: Vector::vector(0.0, 2.0f64.sqrt() / -2.0, 2.0f64.sqrt() / 2.0),
        };

        let intersection = Intersection {
            t: 2.0f64.sqrt(),
            shape: &shape,
            u: None,
            v: None,
        };

        let state = intersection.prepare_state(ray, &vec![]);

        assert!(state.reflect.approx(&Vector::vector(
            0.0,
            2.0f64.sqrt() / 2.0,
            2.0f64.sqrt() / 2.0,
        )))
    }

    #[test]
    fn finding_n1_and_n2_at_intersections() {
        let a = Shape::sphere(ShapeArgs {
            transform: Matrix::scaling(2.0, 2.0, 2.0),
            material: Material {
                transparency: GLASS,
                refractive_index: 1.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let b = Shape::sphere(ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, -0.25),
            material: Material {
                transparency: GLASS,
                refractive_index: 2.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let c = Shape::sphere(ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, 0.25),
            material: Material {
                transparency: GLASS,
                refractive_index: 2.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -4.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let i1 = Intersection {
            t: 2.0,
            shape: &a,
            u: None,
            v: None,
        };
        let i2 = Intersection {
            t: 2.75,
            shape: &b,
            u: None,
            v: None,
        };
        let i3 = Intersection {
            t: 3.25,
            shape: &c,
            u: None,
            v: None,
        };
        let i4 = Intersection {
            t: 4.75,
            shape: &b,
            u: None,
            v: None,
        };
        let i5 = Intersection {
            t: 5.25,
            shape: &c,
            u: None,
            v: None,
        };
        let i6 = Intersection {
            t: 6.0,
            shape: &a,
            u: None,
            v: None,
        };

        let is = vec![i1, i2, i3, i4, i5, i6];

        let state1 = i1.prepare_state(ray, &is.clone());
        let state2 = i2.prepare_state(ray, &is.clone());
        let state3 = i3.prepare_state(ray, &is.clone());
        let state4 = i4.prepare_state(ray, &is.clone());
        let state5 = i5.prepare_state(ray, &is.clone());
        let state6 = i6.prepare_state(ray, &is.clone());

        assert!(
            state1.n1.approx(&1.0)
                && state1.n2.approx(&1.5)
                && state2.n1.approx(&1.5)
                && state2.n2.approx(&2.0)
                && state3.n1.approx(&2.0)
                && state3.n2.approx(&2.5)
                && state4.n1.approx(&2.5)
                && state4.n2.approx(&2.5)
                && state5.n1.approx(&2.5)
                && state5.n2.approx(&1.5)
                && state6.n1.approx(&1.5)
                && state6.n2.approx(&1.0)
        )
    }

    #[test]
    fn under_point_offset_below_surface() {
        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let sphere = Shape::sphere(ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, 1.0),
            ..ShapeArgs::default()
        });

        let intersection = Intersection {
            t: 5.0,
            shape: &sphere,
            u: None,
            v: None,
        };

        let state = intersection.prepare_state(ray, &vec![]);

        assert!(state.under_point.z > EPSILON / 2.0 && state.point.z < state.under_point.z)
    }

    #[test]
    fn schlick_approximation_under_total_internal_reflection() {
        let shape = Shape::sphere(ShapeArgs {
            transform: Matrix::id(),
            material: Material {
                transparency: 1.0,
                refractive_index: 1.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 2.0f64.sqrt() / 2.0),
            direction: Vector::vector(0.0, 1.0, 0.0),
        };

        let i1 = Intersection {
            t: 2.0f64.sqrt() / -2.0,
            shape: &shape,
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: 2.0f64.sqrt() / 2.0,
            shape: &shape,
            u: None,
            v: None,
        };

        let is = vec![i1, i2];

        let state = i2.prepare_state(ray, &is);

        let reflectance = state.reflectance;

        assert!(reflectance.approx(&1.0))
    }

    #[test]
    fn schlick_approximation_with_perpendicular_viewing_angle() {
        let shape = Shape::sphere(ShapeArgs {
            material: Material {
                transparency: 1.0,
                refractive_index: 1.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 0.0),
            direction: Vector::vector(0.0, 1.0, 0.0),
        };

        let i1 = Intersection {
            t: -1.0,
            shape: &shape,
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: 1.0,
            shape: &shape,
            u: None,
            v: None,
        };

        let is = vec![i1, i2];

        let state = i2.prepare_state(ray, &is);

        let reflectance = state.reflectance;

        assert!(reflectance.approx(&0.04))
    }

    #[test]
    fn schlick_approximation_with_small_angle_and_n2_gt_n1() {
        let shape = Shape::sphere(ShapeArgs {
            material: Material {
                transparency: 1.0,
                refractive_index: 1.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let ray = Ray {
            origin: Vector::point(0.0, 0.99, -2.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let i1 = Intersection {
            t: 1.8589,
            shape: &shape,
            u: None,
            v: None,
        };

        let is = vec![i1];

        let state = i1.prepare_state(ray, &is);

        let reflectance = state.reflectance;

        assert!(reflectance.approx(&0.48873))
    }
}
