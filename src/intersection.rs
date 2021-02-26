use crate::config::EPSILON;
use crate::linalg::Vector;
use crate::ray::Ray;
use crate::shape::{Group, Shape};

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

#[derive(Debug, Clone, Copy)]
pub struct Intersection<'a> {
    pub t: f64,
    pub shape: &'a Shape,
    pub u: Option<f64>,
    pub v: Option<f64>,
}

impl<'a> Intersection<'a> {
    pub fn prepare_state(self, ray: Ray, intersections: &Intersections) -> State<'a> {
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

        for intersection in &intersections.intersections {
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

        let reflectance = schlick(eye, normal, n1, n2);

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
}

impl PartialEq for Intersection<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && self.shape == other.shape
    }
}

impl Eq for Intersection<'_> {}

#[derive(Debug, Clone)]
pub struct Intersections<'a> {
    intersections: Vec<Intersection<'a>>,
}

impl<'a> IntoIterator for Intersections<'a> {
    type Item = Intersection<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.intersections.into_iter()
    }
}

impl<'a> Intersections<'a> {
    pub fn new() -> Intersections<'a> {
        Intersections {
            intersections: vec![],
        }
    }

    pub fn insert(&mut self, intersection: Intersection<'a>) {
        self.intersections.push(intersection);
    }

    pub fn clear(&mut self) {
        self.intersections.clear();
    }

    pub fn sort(&mut self) {
        self.intersections
            .sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap())
    }

    pub fn filter_by_group(&mut self, group: &Group) {
        let mut in_left = false;
        let mut in_right = false;

        let mut result = vec![];
        for intersection in &self.intersections {
            if !group.includes(intersection.shape) {
                result.push(*intersection);
                continue;
            }

            let left_hit = group.children[0].includes(intersection.shape);

            if group.kind.allows_intersection(left_hit, in_left, in_right) {
                result.push(*intersection);
            }

            if left_hit {
                in_left = !in_left;
            } else {
                in_right = !in_right;
            }
        }

        self.intersections = result;
    }

    pub fn hit(&self) -> Option<Intersection> {
        self.intersections
            .iter()
            .find(|intersection| intersection.t >= 0.0)
            .map(|intersection| *intersection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;
    use crate::bounding_box::BoundingBox;
    use crate::linalg::Matrix;
    use crate::material::consts::transparency::GLASS;
    use crate::material::Material;
    use crate::shape::{Element, GroupKind, ShapeArgs};

    use test_case::test_case;

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

        let mut intersections = Intersections::new();
        intersections.insert(i2);
        intersections.insert(i1);
        intersections.sort();

        let is: Vec<Intersection> = intersections.into_iter().collect();

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

        let mut is = Intersections::new();
        is.insert(i2);
        is.insert(i1);
        is.sort();

        let hit = is.hit();

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

        let mut is = Intersections::new();
        is.insert(i2);
        is.insert(i1);

        let hit = is.hit();

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

        let mut is = Intersections::new();
        is.insert(i1);
        is.insert(i2);
        is.insert(i3);
        is.insert(i4);
        is.sort();

        let hit = is.hit();

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

        let mut is = Intersections::new();
        is.insert(i2);
        is.insert(i1);

        let hit = is.hit();

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

        let state = i.prepare_state(ray, &Intersections::new());

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

        let state = i.prepare_state(ray, &Intersections::new());

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

        let state = i.prepare_state(ray, &Intersections::new());

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

        let state = intersection.prepare_state(ray, &Intersections::new());

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

        let state = intersection.prepare_state(ray, &Intersections::new());

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

        let mut is = Intersections::new();
        is.insert(i1);
        is.insert(i2);
        is.insert(i3);
        is.insert(i4);
        is.insert(i5);
        is.insert(i6);

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

        let state = intersection.prepare_state(ray, &Intersections::new());

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

        let mut intersections = Intersections::new();
        intersections.insert(i1);
        intersections.insert(i2);

        let state = i2.prepare_state(ray, &intersections);

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

        let mut intersections = Intersections::new();
        intersections.insert(i1);
        intersections.insert(i2);

        let state = i2.prepare_state(ray, &intersections);

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

        let mut intersections = Intersections::new();
        intersections.insert(i1);

        let state = i1.prepare_state(ray, &intersections);

        let reflectance = state.reflectance;

        assert!(reflectance.approx(&0.48873))
    }

    #[test_case(GroupKind::Union       , 0, 3 ; "union"       )]
    #[test_case(GroupKind::Intersection, 1, 2 ; "intersection")]
    #[test_case(GroupKind::Difference  , 0, 1 ; "difference"  )]
    fn filter_by_group(kind: GroupKind, i1: usize, i2: usize) {
        let sphere = Shape::sphere(ShapeArgs::default());
        let cube = Shape::cube(ShapeArgs::default());
        let group = Group {
            kind,
            bbox: BoundingBox::empty(),
            children: vec![
                Element::Primitive(sphere.clone()),
                Element::Primitive(cube.clone()),
            ],
        };

        let is1 = vec![
            Intersection {
                t: 1.0,
                shape: &sphere,
                u: None,
                v: None,
            },
            Intersection {
                t: 2.0,
                shape: &cube,
                u: None,
                v: None,
            },
            Intersection {
                t: 3.0,
                shape: &sphere,
                u: None,
                v: None,
            },
            Intersection {
                t: 4.0,
                shape: &cube,
                u: None,
                v: None,
            },
        ];

        let mut intersections = Intersections::new();
        intersections.insert(is1[0]);
        intersections.insert(is1[1]);
        intersections.insert(is1[2]);
        intersections.insert(is1[3]);

        intersections.filter_by_group(&group);
        let is2: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is2.len() == 2 && is2[0] == is1[i1] && is2[1] == is1[i2])
    }
}
