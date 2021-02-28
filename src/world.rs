use crate::intersection::{Intersection, State};
use crate::light::PointLight;
use crate::linalg::{Matrix, Vector};
use crate::material::{Material, Pattern};
use crate::ray::Ray;
use crate::shape::Element;
use crate::{color::Color, shape::ShapeArgs};

use std::default::Default;

#[derive(Debug)]
pub struct World {
    pub lights: Vec<PointLight>,
    pub elements: Vec<Element>,
}

impl World {
    fn intersect<'a>(&'a self, ray: Ray, intersections: &mut Vec<Intersection<'a>>) {
        intersections.clear();

        for element in &self.elements {
            element.intersect(ray, intersections);
        }
    }

    fn is_shadowed<'a>(
        &'a self,
        light: PointLight,
        point: Vector,
        intersections: &mut Vec<Intersection<'a>>,
    ) -> bool {
        let vector = light.origin - point;
        let distance = vector.magnitude();

        let ray = Ray {
            origin: point,
            direction: vector.normalize(),
        };

        self.intersect(ray, intersections);
        Intersection::sort(intersections);

        if let Some(hit) = Intersection::hit(intersections) {
            hit.shape.casts_shadow && hit.t < distance
        } else {
            false
        }
    }

    fn shade_hit<'a>(
        &'a self,
        state: &State,
        fuel: i32,
        intersections: &mut Vec<Intersection<'a>>,
    ) -> Color {
        let mut color = Color::black();

        for light in &self.lights {
            let shadowed = self.is_shadowed(*light, state.over_point, intersections);

            let surface_color =
                state
                    .shape
                    .lighting(*light, state.over_point, state.eye, state.normal, shadowed);

            let reflected_color = self.reflected_color(state, fuel, intersections);

            let refracted_color = self.refracted_color(state, fuel, intersections);

            color += surface_color
                + if state.shape.material.reflective > 0.0
                    && state.shape.material.transparency > 0.0
                {
                    reflected_color * state.reflectance
                        + refracted_color * (1.0 - state.reflectance)
                } else {
                    reflected_color + refracted_color
                }
        }

        color
    }

    fn reflected_color<'a>(
        &'a self,
        state: &State,
        fuel: i32,
        intersections: &mut Vec<Intersection<'a>>,
    ) -> Color {
        if fuel <= 0 || state.shape.material.reflective == 0.0 {
            Color::black()
        } else {
            let reflect_ray = Ray {
                origin: state.over_point,
                direction: state.reflect,
            };

            let color = self.color_at(reflect_ray, fuel - 1, intersections);

            color * state.shape.material.reflective
        }
    }

    fn refracted_color<'a>(
        &'a self,
        state: &State,
        fuel: i32,
        intersections: &mut Vec<Intersection<'a>>,
    ) -> Color {
        if fuel <= 0 || state.shape.material.transparency == 0.0 {
            Color::black()
        } else {
            let n_ratio = state.n1 / state.n2;
            let cos_i = state.eye.dot(state.normal);
            let sin2_t = n_ratio.powi(2) * (1.0 - cos_i.powi(2));

            if sin2_t > 1.0 {
                Color::black()
            } else {
                let cos_t = (1.0 - sin2_t).sqrt();
                let direction = state.normal * (n_ratio * cos_i - cos_t) - state.eye * n_ratio;

                let refract_ray = Ray {
                    origin: state.under_point,
                    direction,
                };

                self.color_at(refract_ray, fuel - 1, intersections)
                    * state.shape.material.transparency
            }
        }
    }

    pub fn color_at<'a>(
        &'a self,
        ray: Ray,
        fuel: i32,
        intersections: &mut Vec<Intersection<'a>>,
    ) -> Color {
        self.intersect(ray, intersections);
        Intersection::sort(intersections);

        if let Some(hit) = Intersection::hit(intersections) {
            let state = hit.prepare_state(ray, intersections);
            self.shade_hit(&state, fuel, intersections)
        } else {
            Color::black()
        }
    }
}

impl Default for World {
    fn default() -> Self {
        let light = PointLight {
            intensity: Color::white(),
            origin: Vector::point(-10.0, 10.0, -10.0),
        };

        let sphere1 = Element::sphere(ShapeArgs {
            material: Material {
                pattern: Pattern::plain(Color {
                    r: 0.8,
                    g: 1.0,
                    b: 0.6,
                }),
                diffuse: 0.7,
                specular: 0.2,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let sphere2 = Element::sphere(ShapeArgs {
            transform: Matrix::scaling(0.5, 0.5, 0.5),
            ..ShapeArgs::default()
        });

        World {
            lights: vec![light],
            elements: vec![sphere1, sphere2],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;
    use crate::config::FUEL;
    use crate::intersection::Intersection;
    use crate::shape::Shape;

    fn shape(element: &Element) -> &Shape {
        match element {
            Element::Composite(_) => panic!("Expected primitive shape, found group."),
            Element::Primitive(shape) => shape,
        }
    }

    #[test]
    fn intersect_default_world_ray() {
        let world = World::default();
        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let mut is = vec![];
        world.intersect(ray, &mut is);
        Intersection::sort(&mut is);

        assert!(
            is.len() == 4
                && is[0].t.approx(&4.0)
                && is[1].t.approx(&4.5)
                && is[2].t.approx(&5.5)
                && is[3].t.approx(&6.0)
        )
    }

    #[test]
    fn shade_intersection_outside() {
        let world = World::default();
        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let sphere = &world.elements[0];

        let i = Intersection {
            t: 4.0,
            shape: shape(sphere),
            u: None,
            v: None,
        };

        let state = i.prepare_state(ray, &vec![]);

        let color = world.shade_hit(&state, FUEL, &mut vec![]);

        assert!(color.approx(&Color::new(0.38066, 0.47583, 0.28550,)))
    }

    #[test]
    fn shade_intersection_inside() {
        let world = World {
            lights: vec![PointLight {
                intensity: Color::white(),
                origin: Vector::point(0.0, 0.25, 0.0),
            }],
            ..World::default()
        };

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 0.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let sphere = &world.elements[1];

        let i = Intersection {
            t: 0.5,
            shape: shape(sphere),
            u: None,
            v: None,
        };

        let state = i.prepare_state(ray, &vec![]);

        let color = world.shade_hit(&state, FUEL, &mut vec![]);

        assert!(color.approx(&Color::new(0.90498, 0.90498, 0.90498,)))
    }

    #[test]
    fn color_ray_miss() {
        let world = World::default();

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 1.0, 0.0),
        };

        let color = world.color_at(ray, FUEL, &mut vec![]);

        assert!(color.approx(&Color::black()))
    }

    #[test]
    fn color_ray_hit() {
        let world = World::default();

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let color = world.color_at(ray, FUEL, &mut vec![]);

        assert!(color.approx(&Color::new(0.38066, 0.47583, 0.28550,)))
    }

    #[test]
    fn color_intersection_behind_ray() {
        let world = World {
            elements: vec![
                Element::sphere(ShapeArgs {
                    material: Material {
                        pattern: Pattern::plain(Color {
                            r: 0.8,
                            g: 1.0,
                            b: 0.6,
                        }),
                        diffuse: 0.7,
                        specular: 0.2,
                        ambient: 1.0,
                        ..Material::default()
                    },
                    ..ShapeArgs::default()
                }),
                Element::sphere(ShapeArgs {
                    transform: Matrix::scaling(0.5, 0.5, 0.5),
                    material: Material {
                        ambient: 1.0,
                        ..Material::default()
                    },
                    ..ShapeArgs::default()
                }),
            ],
            ..World::default()
        };

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 0.75),
            direction: Vector::vector(0.0, 0.0, -1.0),
        };

        let color = world.color_at(ray, FUEL, &mut vec![]);

        assert!(color.approx(&Color::white()))
    }

    #[test]
    fn shadow_nothing_collinar_point_light() {
        let world = World::default();
        let point = Vector::point(0.0, 10.0, 0.0);

        assert!(!world.is_shadowed(world.lights[0], point, &mut vec![]));
    }

    #[test]
    fn shadow_object_between_point_light() {
        let world = World::default();
        let point = Vector::point(10.0, -10.0, 10.0);

        assert!(world.is_shadowed(world.lights[0], point, &mut vec![]));
    }

    #[test]
    fn shadow_object_behind_light() {
        let world = World::default();
        let point = Vector::point(-20.0, 20.0, -20.0);

        assert!(!world.is_shadowed(world.lights[0], point, &mut vec![]));
    }

    #[test]
    fn shadow_object_behind_point() {
        let world = World::default();
        let point = Vector::point(-2.0, 2.0, -2.0);

        assert!(!world.is_shadowed(world.lights[0], point, &mut vec![]));
    }

    #[test]
    fn color_intersection_in_shadow() {
        let world = World {
            lights: vec![PointLight {
                origin: Vector::point(0.0, 0.0, -10.0),
                intensity: Color::white(),
            }],
            elements: vec![
                Element::sphere(ShapeArgs::default()),
                Element::sphere(ShapeArgs {
                    transform: Matrix::translation(0.0, 0.0, 10.0),
                    ..ShapeArgs::default()
                }),
            ],
        };

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let intersection = Intersection {
            t: 4.0,
            shape: &Shape::sphere(ShapeArgs {
                transform: Matrix::translation(0.0, 0.0, 10.0),
                ..ShapeArgs::default()
            }),
            u: None,
            v: None,
        };

        let color = world.shade_hit(&intersection.prepare_state(ray, &vec![]), FUEL, &mut vec![]);

        assert!(color.approx(&Color::new(0.1, 0.1, 0.1,)))
    }

    #[test]
    fn reflected_color_nonreflective_materiall() {
        let sphere1 = Element::sphere(ShapeArgs {
            material: Material {
                pattern: Pattern::plain(Color {
                    r: 0.8,
                    g: 1.0,
                    b: 0.6,
                }),
                diffuse: 0.7,
                specular: 0.2,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let sphere2 = Element::sphere(ShapeArgs {
            transform: Matrix::scaling(0.5, 0.5, 0.5),
            material: Material {
                ambient: 1.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let world = World {
            elements: vec![sphere1, sphere2],
            ..World::default()
        };

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 0.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let intersection = Intersection {
            t: 1.0,
            shape: shape(&world.elements[1]),
            u: None,
            v: None,
        };

        let state = intersection.prepare_state(ray, &vec![]);

        let color = world.reflected_color(&state, FUEL, &mut vec![]);

        assert!(color.approx(&Color::black()))
    }

    #[test]
    fn reflected_color_reflective_material() {
        let mut world = World::default();

        let plane = Element::plane(ShapeArgs {
            transform: Matrix::translation(0.0, -1.0, 0.0),
            material: Material {
                reflective: 0.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        world.elements.push(plane);

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -3.0),
            direction: Vector::vector(0.0, 2.0f64.sqrt() / -2.0, 2.0f64.sqrt() / 2.0),
        };

        let intersection = Intersection {
            t: 2.0f64.sqrt(),
            shape: shape(&world.elements[2]),
            u: None,
            v: None,
        };

        let state = intersection.prepare_state(ray, &vec![]);

        let color = world.reflected_color(&state, FUEL, &mut vec![]);

        assert!(color.approx(&Color::new(0.190332, 0.237915, 0.142749,)))
    }

    #[test]
    fn shade_hit_reflective_material() {
        let mut world = World::default();

        let plane = Element::plane(ShapeArgs {
            transform: Matrix::translation(0.0, -1.0, 0.0),
            material: Material {
                reflective: 0.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        world.elements.push(plane);

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -3.0),
            direction: Vector::vector(0.0, 2.0f64.sqrt() / -2.0, 2.0f64.sqrt() / 2.0),
        };

        let intersection = Intersection {
            t: 2.0f64.sqrt(),
            shape: shape(&world.elements[2]),
            u: None,
            v: None,
        };

        let state = intersection.prepare_state(ray, &vec![]);

        let color = world.shade_hit(&state, FUEL, &mut vec![]);

        assert!(color.approx(&Color::new(0.876757, 0.924340, 0.829174,)))
    }

    #[test]
    fn color_at_mutually_reflective_surfaces() {
        let light = PointLight {
            origin: Vector::point(0.0, 0.0, 0.0),
            intensity: Color::white(),
        };

        let lower_plane = Element::plane(ShapeArgs {
            transform: Matrix::translation(0.0, -1.0, 0.0),
            material: Material {
                reflective: 1.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let upper_plane = Element::plane(ShapeArgs {
            transform: Matrix::translation(0.0, 1.0, 0.0),
            material: Material {
                reflective: 1.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let world = World {
            lights: vec![light],
            elements: vec![lower_plane, upper_plane],
        };

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 0.0),
            direction: Vector::vector(0.0, 1.0, 0.0),
        };

        let result = std::panic::catch_unwind(|| world.color_at(ray, FUEL, &mut vec![]));

        assert!(result.is_ok())
    }

    #[test]
    fn reflected_color_maximum_recursive_depth() {
        let mut world = World::default();

        let plane = Element::plane(ShapeArgs {
            transform: Matrix::translation(0.0, -1.0, 0.0),
            material: Material {
                reflective: 0.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        world.elements.push(plane);

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -3.0),
            direction: Vector::vector(0.0, 2.0f64.sqrt() / -2.0, 2.0f64.sqrt() / 2.0),
        };

        let intersection = Intersection {
            t: 2.0f64.sqrt(),
            shape: shape(&world.elements[2]),
            u: None,
            v: None,
        };

        let state = intersection.prepare_state(ray, &vec![]);

        let color = world.reflected_color(&state, 0, &mut vec![]);

        assert!(color.approx(&Color::black()))
    }

    #[test]
    fn refracted_color_opaque_surface() {
        let world = World::default();

        let shape = shape(&world.elements[0]);

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let i1 = Intersection {
            t: 4.0,
            shape,
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: 6.0,
            shape,
            u: None,
            v: None,
        };

        let is = vec![i1, i2];

        let state = i1.prepare_state(ray, &is);

        let color = world.refracted_color(&state, 5, &mut vec![]);

        assert!(color.approx(&Color::black()))
    }

    #[test]
    fn refracted_color_maximum_recursive_depth() {
        let mut world = World::default();

        if let Element::Primitive(shape) = &mut world.elements[0] {
            shape.material.transparency = 1.0;
            shape.material.refractive_index = 1.5;
        }

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let i1 = Intersection {
            t: 4.0,
            shape: shape(&world.elements[0]),
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: 6.0,
            shape: shape(&world.elements[0]),
            u: None,
            v: None,
        };

        let is = vec![i1, i2];

        let state = i1.prepare_state(ray, &is);

        let color = world.refracted_color(&state, 0, &mut vec![]);

        assert!(color.approx(&Color::black()))
    }

    #[test]
    fn refracted_color_under_total_internal_reflection() {
        let mut world = World::default();

        if let Element::Primitive(shape) = &mut world.elements[0] {
            shape.material.transparency = 1.0;
            shape.material.refractive_index = 1.5;
        }

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 2.0f64.sqrt() / 2.0),
            direction: Vector::vector(0.0, 1.0, 0.0),
        };

        let i1 = Intersection {
            t: 2.0f64.sqrt() / -2.0,
            shape: shape(&world.elements[0]),
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: 2.0f64.sqrt() / 2.0,
            shape: shape(&world.elements[0]),
            u: None,
            v: None,
        };

        let is = vec![i1, i2];

        let state = i2.prepare_state(ray, &is);

        let color = world.refracted_color(&state, 5, &mut vec![]);

        assert!(color.approx(&Color::black()))
    }

    #[test]
    fn refracted_color_with_refracted_ray() {
        let mut world = World::default();

        if let Element::Primitive(shape) = &mut world.elements[0] {
            shape.material.ambient = 1.0;
            shape.material.pattern = Pattern::Debug;
        }

        if let Element::Primitive(shape) = &mut world.elements[1] {
            shape.material.transparency = 1.0;
            shape.material.refractive_index = 1.5;
        }

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 0.1),
            direction: Vector::vector(0.0, 1.0, 0.0),
        };

        let i1 = Intersection {
            t: -0.9899,
            shape: shape(&world.elements[0]),
            u: None,
            v: None,
        };

        let i2 = Intersection {
            t: -0.4899,
            shape: shape(&world.elements[1]),
            u: None,
            v: None,
        };

        let i3 = Intersection {
            t: 0.4899,
            shape: shape(&world.elements[1]),
            u: None,
            v: None,
        };

        let i4 = Intersection {
            t: 0.9899,
            shape: shape(&world.elements[0]),
            u: None,
            v: None,
        };

        let is = vec![i1, i2, i3, i4];

        let state = i3.prepare_state(ray, &is);

        let color = world.refracted_color(&state, 5, &mut vec![]);

        assert!(color.approx(&Color::new(0.0, 0.998874, 0.047218,)))
    }

    #[test]
    fn shade_hit_with_transparent_material() {
        let mut world = World::default();

        let floor = Element::plane(ShapeArgs {
            transform: Matrix::translation(0.0, -1.0, 0.0),
            material: Material {
                transparency: 0.5,
                refractive_index: 1.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let ball = Element::sphere(ShapeArgs {
            transform: Matrix::translation(0.0, -3.5, -0.5),
            material: Material {
                pattern: Pattern::plain(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                }),
                ambient: 0.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        world.elements.push(floor);
        world.elements.push(ball);

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -3.0),
            direction: Vector::vector(0.0, 2.0f64.sqrt() / -2.0, 2.0f64.sqrt() / 2.0),
        };

        let i1 = Intersection {
            t: 2.0f64.sqrt(),
            shape: shape(&world.elements[2]),
            u: None,
            v: None,
        };

        let is = vec![i1];

        let state = i1.prepare_state(ray, &is);

        let color = world.shade_hit(&state, 5, &mut vec![]);

        assert!(color.approx(&Color::new(0.93642, 0.68642, 0.68642,)))
    }

    #[test]
    fn shade_hit_with_reflective_and_transparent_material() {
        let mut world = World::default();

        let floor = Element::plane(ShapeArgs {
            transform: Matrix::translation(0.0, -1.0, 0.0),
            material: Material {
                reflective: 0.5,
                transparency: 0.5,
                refractive_index: 1.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let ball = Element::sphere(ShapeArgs {
            transform: Matrix::translation(0.0, -3.5, -0.5),
            material: Material {
                pattern: Pattern::plain(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                }),
                ambient: 0.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        world.elements.push(floor);
        world.elements.push(ball);

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -3.0),
            direction: Vector::vector(0.0, 2.0f64.sqrt() / -2.0, 2.0f64.sqrt() / 2.0),
        };

        let i1 = Intersection {
            t: 2.0f64.sqrt(),
            shape: shape(&world.elements[2]),
            u: None,
            v: None,
        };

        let is = vec![i1];

        let state = i1.prepare_state(ray, &is);

        let color = world.shade_hit(&state, 5, &mut vec![]);

        assert!(color.approx(&Color::new(0.93391, 0.69643, 0.69243,)))
    }
}
