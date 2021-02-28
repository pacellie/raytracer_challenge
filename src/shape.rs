// use lazy_static::lazy;

use crate::approx::Approx;
use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::config::EPSILON;
use crate::intersection::{Intersection, Intersections};
use crate::light::PointLight;
use crate::linalg::{Matrix, Vector};
use crate::material::Material;
// use crate::material::Pattern;
use crate::ray::Ray;

use std::cell::Cell;
use std::default::Default;
use std::hash::{Hash, Hasher};

thread_local! {
    static ID: Cell<usize> = Cell::new(0);
}

fn next_id() -> usize {
    ID.with(|cell| {
        let i = cell.get();
        cell.set(i + 1);
        i
    })
}

#[derive(Debug)]
pub enum Element {
    Composite(Group),
    Primitive(Shape),
}

impl Approx<Element> for Element {
    fn approx(&self, other: &Element) -> bool {
        match (self, other) {
            (Element::Composite(sgroup), Element::Composite(ogroup)) => sgroup.approx(ogroup),
            (Element::Primitive(sshape), Element::Primitive(oshape)) => sshape.approx(oshape),
            (_, _) => false,
        }
    }
}

impl Element {
    fn propagate_inverses(
        &mut self,
        transform: Matrix,
        inv: Matrix,
        inv_tsp: Matrix,
        material: Option<Material>,
    ) {
        match self {
            Element::Composite(group) => {
                for child in &mut group.children {
                    child.propagate_inverses(transform, inv, inv_tsp, material.clone());
                }
                group.bbox = group.bbox.transform(transform);
            }
            Element::Primitive(shape) => {
                shape.transform_inv = shape.transform_inv * inv;
                shape.transform_inv_tsp = inv_tsp * shape.transform_inv_tsp;
                if let Some(material) = material {
                    shape.material = material;
                    shape.material_inv = inv;
                } else {
                    shape.material_inv = shape.material_inv * inv;
                }
            }
        }
    }

    pub fn composite(
        transform: Matrix,
        material: Option<Material>,
        kind: GroupKind,
        children: Vec<Element>,
    ) -> Element {
        match kind {
            GroupKind::Aggregation => (),
            _ => assert!(children.len() == 2),
        }

        let inv = transform.inverse();
        let inv_tsp = inv.transpose();

        let mut bbox = BoundingBox::empty();
        for child in &children {
            bbox = bbox.union(&child.bbox());
        }

        let mut composite = Element::Composite(Group {
            kind,
            bbox,
            children,
        });
        composite.propagate_inverses(transform, inv, inv_tsp, material);

        composite
    }

    pub fn sphere(args: ShapeArgs) -> Element {
        Element::Primitive(Shape::sphere(args))
    }

    pub fn plane(args: ShapeArgs) -> Element {
        Element::Primitive(Shape::plane(args))
    }

    pub fn cube(args: ShapeArgs) -> Element {
        Element::Primitive(Shape::cube(args))
    }

    pub fn cylinder(args: ShapeArgs, min: f64, max: f64, closed: bool) -> Element {
        Element::Primitive(Shape::cylinder(args, min, max, closed))
    }

    pub fn cone(args: ShapeArgs, min: f64, max: f64, closed: bool) -> Element {
        Element::Primitive(Shape::cone(args, min, max, closed))
    }

    pub fn triangle(args: ShapeArgs, p1: Vector, p2: Vector, p3: Vector) -> Element {
        Element::Primitive(Shape::triangle(args, p1, p2, p3))
    }

    pub fn smooth_triangle(
        args: ShapeArgs,
        p1: Vector,
        p2: Vector,
        p3: Vector,
        n1: Vector,
        n2: Vector,
        n3: Vector,
    ) -> Element {
        Element::Primitive(Shape::smooth_triangle(args, p1, p2, p3, n1, n2, n3))
    }

    pub fn intersect<'a>(&'a self, ray: Ray, intersections: &mut Intersections<'a>) {
        match self {
            Element::Composite(group) => group.intersect(ray, intersections),
            Element::Primitive(shape) => shape.intersect(ray, intersections),
        }
    }

    pub fn bbox(&self) -> BoundingBox {
        match self {
            Element::Composite(group) => group.bbox,
            Element::Primitive(shape) => shape.bbox,
        }
    }

    pub fn includes(&self, shape: &Shape) -> bool {
        match self {
            Element::Composite(group) => group.includes(shape),
            Element::Primitive(s) => *s == *shape,
        }
    }
}

#[derive(Debug)]
pub enum GroupKind {
    Union,
    Intersection,
    Difference,
    Aggregation,
}

impl GroupKind {
    pub fn allows_intersection(&self, left_hit: bool, in_left: bool, in_right: bool) -> bool {
        match self {
            GroupKind::Union => (left_hit && !in_right) || (!left_hit && !in_left),
            GroupKind::Intersection => (left_hit && in_right) || (!left_hit && in_left),
            GroupKind::Difference => (left_hit && !in_right) || (!left_hit && in_left),
            GroupKind::Aggregation => true,
        }
    }
}

#[derive(Debug)]
pub struct Group {
    pub kind: GroupKind,
    pub bbox: BoundingBox,
    pub children: Vec<Element>,
}

impl Approx<Group> for Group {
    fn approx(&self, other: &Group) -> bool {
        self.bbox.approx(&other.bbox) && self.children.approx(&other.children)
    }
}

// lazy_static::lazy_static! {
//     static ref DEBUG: Shape = Shape::cube(ShapeArgs {
//         material: Material {
//             pattern: Pattern::plain(Color::new(1.0, 0.0, 0.0)),
//             ..Material::default()
//         },
//         ..ShapeArgs::default()
//     });
// }

// fn intersect_bbox<'a>(
//     bbox: &BoundingBox,
//     shape: &'a Shape,
//     ray: Ray,
//     intersections: &mut Intersections<'a>,
// ) {
//     let (x_t_min, x_t_max) =
//         Geometry::intersect_cube_axis(ray.origin.x, ray.direction.x, bbox.min.x, bbox.max.x);
//     let (y_t_min, y_t_max) =
//         Geometry::intersect_cube_axis(ray.origin.y, ray.direction.y, bbox.min.y, bbox.max.y);
//     let (z_t_min, z_t_max) =
//         Geometry::intersect_cube_axis(ray.origin.z, ray.direction.z, bbox.min.z, bbox.max.z);

//     let t_min = x_t_min.max(y_t_min).max(z_t_min);
//     let t_max = x_t_max.min(y_t_max).min(z_t_max);

//     if t_min <= t_max {
//         intersections.insert(Intersection { t: t_min, shape });
//         intersections.insert(Intersection { t: t_max, shape });
//     }
// }

impl Group {
    pub fn includes(&self, shape: &Shape) -> bool {
        self.children.iter().any(|element| element.includes(shape))
    }

    pub fn intersect<'a>(&'a self, ray: Ray, intersections: &mut Intersections<'a>) {
        // intersect_bbox(&self.bbox, &DEBUG, ray, intersections);

        if self.bbox.intersects(ray) {
            match self.kind {
                GroupKind::Aggregation => {
                    for child in &self.children {
                        child.intersect(ray, intersections);
                    }
                }
                _ => {
                    let mut tmp = Intersections::new();
                    for child in &self.children {
                        child.intersect(ray, &mut tmp);
                    }
                    tmp.sort();
                    tmp.filter_by_group(self);
                    intersections.append(&mut tmp);
                }
            }
        }
    }
}

pub struct ShapeArgs {
    pub transform: Matrix,
    pub material: Material,
    pub casts_shadow: bool,
}

impl Default for ShapeArgs {
    fn default() -> Self {
        ShapeArgs {
            transform: Matrix::id(),
            material: Material::default(),
            casts_shadow: true,
        }
    }
}

impl Approx<ShapeArgs> for ShapeArgs {
    fn approx(&self, other: &ShapeArgs) -> bool {
        self.transform.approx(&other.transform)
            && self.material.approx(&other.material)
            && self.casts_shadow.approx(&other.casts_shadow)
    }
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub transform_inv: Matrix,
    pub transform_inv_tsp: Matrix,
    pub bbox: BoundingBox,
    pub material_inv: Matrix,
    pub material: Material,
    pub geometry: Geometry,
    pub casts_shadow: bool,
    id: usize,
}

impl PartialEq for Shape {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Shape {}

impl Hash for Shape {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Approx<Shape> for Shape {
    fn approx(&self, other: &Shape) -> bool {
        self.transform_inv.approx(&other.transform_inv)
            && self.transform_inv_tsp.approx(&other.transform_inv_tsp)
            && self.bbox.approx(&other.bbox)
            && self.material_inv.approx(&other.material_inv)
            && self.material.approx(&other.material)
            && self.geometry.approx(&other.geometry)
            && self.casts_shadow.approx(&other.casts_shadow)
    }
}

impl Shape {
    fn shape(args: ShapeArgs, geometry: Geometry) -> Shape {
        let inv = args.transform.inverse();
        Shape {
            transform_inv: inv,
            transform_inv_tsp: inv.transpose(),
            bbox: geometry.bbox().transform(args.transform),
            material_inv: inv,
            material: args.material,
            geometry,
            casts_shadow: args.casts_shadow,
            id: next_id(),
        }
    }

    pub fn sphere(args: ShapeArgs) -> Shape {
        Shape::shape(args, Geometry::Sphere)
    }

    pub fn plane(args: ShapeArgs) -> Shape {
        Shape::shape(args, Geometry::Plane)
    }

    pub fn cube(args: ShapeArgs) -> Shape {
        Shape::shape(args, Geometry::Cube)
    }

    pub fn cylinder(args: ShapeArgs, min: f64, max: f64, closed: bool) -> Shape {
        Shape::shape(args, Geometry::Cylinder { min, max, closed })
    }

    pub fn cone(args: ShapeArgs, min: f64, max: f64, closed: bool) -> Shape {
        Shape::shape(args, Geometry::Cone { min, max, closed })
    }

    pub fn triangle(args: ShapeArgs, p1: Vector, p2: Vector, p3: Vector) -> Shape {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let n = e2.cross(e1).normalize();

        Shape::shape(
            args,
            Geometry::Triangle {
                p1,
                p2,
                p3,
                e1,
                e2,
                n,
            },
        )
    }

    pub fn smooth_triangle(
        args: ShapeArgs,
        p1: Vector,
        p2: Vector,
        p3: Vector,
        n1: Vector,
        n2: Vector,
        n3: Vector,
    ) -> Shape {
        let e1 = p2 - p1;
        let e2 = p3 - p1;

        Shape::shape(
            args,
            Geometry::SmoothTriangle {
                p1,
                p2,
                p3,
                e1,
                e2,
                n1,
                n2,
                n3,
            },
        )
    }

    pub fn intersect<'a>(&'a self, ray: Ray, intersections: &mut Intersections<'a>) {
        let ray = ray.transform(self.transform_inv);
        self.geometry.intersect(self, ray, intersections)
    }

    pub fn normal(&self, point: Vector, u: Option<f64>, v: Option<f64>) -> Vector {
        let shape_point = self.transform_inv * point;
        let shape_normal = self.geometry.normal(shape_point, u, v);

        let mut world_normal = self.transform_inv_tsp * shape_normal;
        world_normal.w = 0.0;

        world_normal.normalize()
    }

    pub fn lighting(
        &self,
        light: PointLight,
        point: Vector,
        eye: Vector,
        normal: Vector,
        shadowed: bool,
    ) -> Color {
        let color = self.material.pattern.color_at(self.material_inv * point);

        let effective_color = color * light.intensity;
        let intensity = light.intensity;

        let light = (light.origin - point).normalize();

        let ambient = effective_color * self.material.ambient;
        let mut diffuse = Color::black();
        let mut specular = Color::black();

        let light_dot_normal = light.dot(normal);
        if !shadowed && light_dot_normal >= 0.0 {
            diffuse = effective_color * self.material.diffuse * light_dot_normal;

            let reflect = (-light).reflect(normal);
            let reflect_dot_eye = reflect.dot(eye);
            if reflect_dot_eye > 0.0 {
                specular = intensity
                    * self.material.specular
                    * reflect_dot_eye.powf(self.material.shininess);
            }
        }

        ambient + diffuse + specular
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Geometry {
    Sphere,
    Plane,
    Cube,
    Cylinder {
        min: f64,
        max: f64,
        closed: bool,
    },
    Cone {
        min: f64,
        max: f64,
        closed: bool,
    },
    Triangle {
        p1: Vector,
        p2: Vector,
        p3: Vector,
        e1: Vector,
        e2: Vector,
        n: Vector,
    },
    SmoothTriangle {
        p1: Vector,
        p2: Vector,
        p3: Vector,
        e1: Vector,
        e2: Vector,
        n1: Vector,
        n2: Vector,
        n3: Vector,
    },
}

impl Approx<Geometry> for Geometry {
    fn approx(&self, other: &Geometry) -> bool {
        match (self, other) {
            (Geometry::Sphere, Geometry::Sphere) => true,
            (Geometry::Plane, Geometry::Plane) => true,
            (Geometry::Cube, Geometry::Cube) => true,
            (
                Geometry::Cylinder {
                    min: smin,
                    max: smax,
                    closed: sclosed,
                },
                Geometry::Cylinder {
                    min: omin,
                    max: omax,
                    closed: oclosed,
                },
            ) => smin.approx(omin) && smax.approx(omax) && sclosed.approx(oclosed),
            (
                Geometry::Cone {
                    min: smin,
                    max: smax,
                    closed: sclosed,
                },
                Geometry::Cone {
                    min: omin,
                    max: omax,
                    closed: oclosed,
                },
            ) => smin.approx(omin) && smax.approx(omax) && sclosed.approx(oclosed),
            (
                Geometry::Triangle {
                    p1: sp1,
                    p2: sp2,
                    p3: sp3,
                    e1: se1,
                    e2: se2,
                    n: sn,
                },
                Geometry::Triangle {
                    p1: op1,
                    p2: op2,
                    p3: op3,
                    e1: oe1,
                    e2: oe2,
                    n: on,
                },
            ) => {
                sp1.approx(op1)
                    && sp2.approx(op2)
                    && sp3.approx(op3)
                    && se1.approx(oe1)
                    && se2.approx(oe2)
                    && sn.approx(on)
            }
            (
                Geometry::SmoothTriangle {
                    p1: sp1,
                    p2: sp2,
                    p3: sp3,
                    e1: se1,
                    e2: se2,
                    n1: sn1,
                    n2: sn2,
                    n3: sn3,
                },
                Geometry::SmoothTriangle {
                    p1: op1,
                    p2: op2,
                    p3: op3,
                    e1: oe1,
                    e2: oe2,
                    n1: on1,
                    n2: on2,
                    n3: on3,
                },
            ) => {
                sp1.approx(op1)
                    && sp2.approx(op2)
                    && sp3.approx(op3)
                    && se1.approx(oe1)
                    && se2.approx(oe2)
                    && sn1.approx(on1)
                    && sn2.approx(on2)
                    && sn3.approx(on3)
            }
            (_, _) => false,
        }
    }
}

impl Geometry {
    fn intersect_sphere<'a>(shape: &'a Shape, ray: Ray, intersections: &mut Intersections<'a>) {
        let sphere_to_ray = ray.origin - Vector::point(0.0, 0.0, 0.0);

        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * ray.direction.dot(sphere_to_ray);
        let c = sphere_to_ray.dot(sphere_to_ray) - 1.0;

        let discriminant = b.powi(2) - 4.0 * a * c;

        if discriminant < 0.0 {
            return;
        }

        let t0 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t1 = (-b + discriminant.sqrt()) / (2.0 * a);

        intersections.insert(Intersection {
            t: t0,
            shape,
            u: None,
            v: None,
        });
        intersections.insert(Intersection {
            t: t1,
            shape,
            u: None,
            v: None,
        });
    }

    fn intersect_plane<'a>(shape: &'a Shape, ray: Ray, intersections: &mut Intersections<'a>) {
        if ray.direction.y.approx(&0.0) {
            return;
        }

        intersections.insert(Intersection {
            t: -ray.origin.y / ray.direction.y,
            shape,
            u: None,
            v: None,
        });
    }

    pub fn intersect_cube_axis(origin: f64, direction: f64, min: f64, max: f64) -> (f64, f64) {
        let t_min_numerator = min - origin;
        let t_max_numerator = max - origin;

        let (t_min, t_max) = if direction.abs() >= EPSILON {
            (t_min_numerator / direction, t_max_numerator / direction)
        } else {
            (
                t_min_numerator * f64::INFINITY,
                t_max_numerator * f64::INFINITY,
            )
        };

        if t_min > t_max {
            (t_max, t_min)
        } else {
            (t_min, t_max)
        }
    }

    fn intersect_cube<'a>(shape: &'a Shape, ray: Ray, intersections: &mut Intersections<'a>) {
        let (x_t_min, x_t_max) =
            Geometry::intersect_cube_axis(ray.origin.x, ray.direction.x, -1.0, 1.0);
        let (y_t_min, y_t_max) =
            Geometry::intersect_cube_axis(ray.origin.y, ray.direction.y, -1.0, 1.0);
        let (z_t_min, z_t_max) =
            Geometry::intersect_cube_axis(ray.origin.z, ray.direction.z, -1.0, 1.0);

        let t_min = x_t_min.max(y_t_min).max(z_t_min);
        let t_max = x_t_max.min(y_t_max).min(z_t_max);

        if t_min <= t_max {
            intersections.insert(Intersection {
                t: t_min,
                shape,
                u: None,
                v: None,
            });
            intersections.insert(Intersection {
                t: t_max,
                shape,
                u: None,
                v: None,
            });
        }
    }

    fn intersects_cap(ray: Ray, t: f64, radius: f64) -> bool {
        let x = ray.origin.x + t * ray.direction.x;
        let z = ray.origin.z + t * ray.direction.z;

        (x.powi(2) + z.powi(2)) <= radius.powi(2)
    }

    fn intersect_cap<'a>(
        shape: &'a Shape,
        ray: Ray,
        min: f64,
        max: f64,
        min_radius: f64,
        max_radius: f64,
        closed: bool,
        intersections: &mut Intersections<'a>,
    ) {
        if !closed || ray.direction.y.approx(&0.0) {
            return;
        }

        let t = (min - ray.origin.y) / ray.direction.y;
        if Geometry::intersects_cap(ray, t, min_radius) {
            intersections.insert(Intersection {
                t,
                shape,
                u: None,
                v: None,
            });
        }

        let t = (max - ray.origin.y) / ray.direction.y;
        if Geometry::intersects_cap(ray, t, max_radius) {
            intersections.insert(Intersection {
                t,
                shape,
                u: None,
                v: None,
            });
        }
    }

    fn intersect_cylinder<'a>(
        shape: &'a Shape,
        ray: Ray,
        min: f64,
        max: f64,
        closed: bool,
        intersections: &mut Intersections<'a>,
    ) {
        let o = ray.origin;
        let d = ray.direction;

        let a = d.x.powi(2) + d.z.powi(2);

        if !a.approx(&0.0) {
            let b = 2.0 * o.x * d.x + 2.0 * o.z * d.z;
            let c = o.x.powi(2) + o.z.powi(2) - 1.0;

            let discriminant = b.powi(2) - 4.0 * a * c;
            if discriminant >= 0.0 {
                let t0 = (-b - discriminant.sqrt()) / (2.0 * a);
                let y0 = ray.origin.y + t0 * ray.direction.y;
                if min < y0 && y0 < max {
                    intersections.insert(Intersection {
                        t: t0,
                        shape,
                        u: None,
                        v: None,
                    });
                }

                let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
                let y1 = ray.origin.y + t1 * ray.direction.y;
                if min < y1 && y1 < max {
                    intersections.insert(Intersection {
                        t: t1,
                        shape,
                        u: None,
                        v: None,
                    })
                }
            }
        }

        Geometry::intersect_cap(shape, ray, min, max, 1.0, 1.0, closed, intersections);
    }

    fn intersect_cone<'a>(
        shape: &'a Shape,
        ray: Ray,
        min: f64,
        max: f64,
        closed: bool,
        intersections: &mut Intersections<'a>,
    ) {
        let o = ray.origin;
        let d = ray.direction;

        let a = d.x.powi(2) - d.y.powi(2) + d.z.powi(2);
        let b = 2.0 * o.x * d.x - 2.0 * o.y * d.y + 2.0 * o.z * d.z;

        if !a.approx(&0.0) || !b.approx(&0.0) {
            let c = o.x.powi(2) - o.y.powi(2) + o.z.powi(2);
            if !a.approx(&0.0) {
                let discriminant = b.powi(2) - 4.0 * a * c;
                if discriminant >= 0.0 {
                    let t0 = (-b - discriminant.sqrt()) / (2.0 * a);
                    let y0 = ray.origin.y + t0 * ray.direction.y;
                    if min < y0 && y0 < max {
                        intersections.insert(Intersection {
                            t: t0,
                            shape,
                            u: None,
                            v: None,
                        });
                    }

                    let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
                    let y1 = ray.origin.y + t1 * ray.direction.y;
                    if min < y1 && y1 < max {
                        intersections.insert(Intersection {
                            t: t1,
                            shape,
                            u: None,
                            v: None,
                        })
                    }
                }
            } else {
                intersections.insert(Intersection {
                    t: -c / (2.0 * b),
                    shape,
                    u: None,
                    v: None,
                })
            }
        }

        Geometry::intersect_cap(shape, ray, min, max, min, max, closed, intersections);
    }

    fn intersect_triangle<'a>(
        shape: &'a Shape,
        ray: Ray,
        p1: Vector,
        e1: Vector,
        e2: Vector,
        intersections: &mut Intersections<'a>,
    ) {
        let dir_cross_e2 = ray.direction.cross(e2);
        let determinant = e1.dot(dir_cross_e2);

        if determinant.abs() < EPSILON {
            return;
        }

        let f = 1.0 / determinant;
        let p1_to_origin = ray.origin - p1;
        let u = f * p1_to_origin.dot(dir_cross_e2);

        if u < 0.0 || u > 1.0 {
            return;
        }

        let origin_cross_e1 = p1_to_origin.cross(e1);
        let v = f * ray.direction.dot(origin_cross_e1);

        if v < 0.0 || u + v > 1.0 {
            return;
        }

        intersections.insert(Intersection {
            t: f * e2.dot(origin_cross_e1),
            shape,
            u: Some(u),
            v: Some(v),
        });
    }

    pub fn intersect<'a>(&self, shape: &'a Shape, ray: Ray, intersections: &mut Intersections<'a>) {
        match self {
            Geometry::Sphere => Geometry::intersect_sphere(shape, ray, intersections),
            Geometry::Plane => Geometry::intersect_plane(shape, ray, intersections),
            Geometry::Cube => Geometry::intersect_cube(shape, ray, intersections),
            Geometry::Cylinder { min, max, closed } => {
                Geometry::intersect_cylinder(shape, ray, *min, *max, *closed, intersections)
            }
            Geometry::Cone { min, max, closed } => {
                Geometry::intersect_cone(shape, ray, *min, *max, *closed, intersections)
            }
            Geometry::Triangle { p1, e1, e2, .. } => {
                Geometry::intersect_triangle(shape, ray, *p1, *e1, *e2, intersections)
            }
            Geometry::SmoothTriangle { p1, e1, e2, .. } => {
                Geometry::intersect_triangle(shape, ray, *p1, *e1, *e2, intersections)
            }
        }
    }

    fn normal_cube(point: Vector) -> Vector {
        let x_abs = point.x.abs();
        let y_abs = point.y.abs();
        let z_abs = point.z.abs();

        let max = x_abs.max(y_abs).max(z_abs);

        if max == x_abs {
            Vector::vector(point.x, 0.0, 0.0)
        } else if max == y_abs {
            Vector::vector(0.0, point.y, 0.0)
        } else {
            Vector::vector(0.0, 0.0, point.z)
        }
    }

    fn normal_cylinder(point: Vector, min: f64, max: f64) -> Vector {
        let distance = point.x.powi(2) + point.z.powi(2);

        if distance < 1.0 && point.y >= max - EPSILON {
            Vector::vector(0.0, 1.0, 0.0)
        } else if distance < 1.0 && point.y <= min + EPSILON {
            Vector::vector(0.0, -1.0, 0.0)
        } else {
            Vector::vector(point.x, 0.0, point.z)
        }
    }

    fn normal_cone(point: Vector, min: f64, max: f64) -> Vector {
        let distance = point.x.powi(2) + point.z.powi(2);

        if distance < 1.0 && point.y >= max - EPSILON {
            Vector::vector(0.0, 1.0, 0.0)
        } else if distance < 1.0 && point.y <= min + EPSILON {
            Vector::vector(0.0, -1.0, 0.0)
        } else {
            let mut y = distance.sqrt();
            if point.y > 0.0 {
                y = -y;
            }
            Vector::vector(point.x, y, point.z)
        }
    }

    pub fn normal(&self, point: Vector, u: Option<f64>, v: Option<f64>) -> Vector {
        match self {
            Geometry::Sphere => Vector::vector(point.x, point.y, point.z),
            Geometry::Plane => Vector::vector(0.0, 1.0, 0.0),
            Geometry::Cube => Geometry::normal_cube(point),
            Geometry::Cylinder { min, max, .. } => Geometry::normal_cylinder(point, *min, *max),
            Geometry::Cone { min, max, .. } => Geometry::normal_cone(point, *min, *max),
            Geometry::Triangle { n, .. } => *n,
            Geometry::SmoothTriangle { n1, n2, n3, .. } => {
                let u = u.unwrap();
                let v = v.unwrap();

                *n2 * u + *n3 * v + *n1 * (1.0 - u - v)
            }
        }
    }

    pub fn bbox(&self) -> BoundingBox {
        match self {
            Geometry::Sphere => BoundingBox::new(
                Vector::point(-1.0, -1.0, -1.0),
                Vector::point(1.0, 1.0, 1.0),
            ),
            Geometry::Plane => BoundingBox::new(
                Vector::point(f64::NEG_INFINITY, 0.0, f64::NEG_INFINITY),
                Vector::point(f64::INFINITY, 0.0, f64::INFINITY),
            ),
            Geometry::Cube => BoundingBox::new(
                Vector::point(-1.0, -1.0, -1.0),
                Vector::point(1.0, 1.0, 1.0),
            ),
            Geometry::Cylinder { min, max, closed } => {
                if *closed {
                    BoundingBox::new(
                        Vector::point(-1.0, *min, -1.0),
                        Vector::point(1.0, *max, 1.0),
                    )
                } else {
                    BoundingBox::new(
                        Vector::point(-1.0, f64::NEG_INFINITY, -1.0),
                        Vector::point(1.0, f64::INFINITY, 1.0),
                    )
                }
            }
            Geometry::Cone { min, max, closed } => {
                if *closed {
                    let limit = min.abs().max(max.abs());
                    BoundingBox::new(
                        Vector::point(-limit, *min, -limit),
                        Vector::point(limit, *max, limit),
                    )
                } else {
                    BoundingBox::new(
                        Vector::point(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
                        Vector::point(f64::INFINITY, f64::INFINITY, f64::INFINITY),
                    )
                }
            }
            Geometry::Triangle { p1, p2, p3, .. } => {
                BoundingBox::empty().insert(*p1).insert(*p2).insert(*p3)
            }
            Geometry::SmoothTriangle { p1, p2, p3, .. } => {
                BoundingBox::empty().insert(*p1).insert(*p2).insert(*p3)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;
    use crate::material::Pattern;

    use test_case::test_case;

    // Sphere Tests

    #[test_case(Vector::point(0.0, 0.0, -5.0), Vector::vector(0.0, 0.0, 1.0), Matrix::id()                  ,  4.0,  6.0 ; "before" )]
    #[test_case(Vector::point(0.0, 0.0,  0.0), Vector::vector(0.0, 0.0, 1.0), Matrix::id()                  , -1.0,  1.0 ; "inside" )]
    #[test_case(Vector::point(0.0, 0.0,  5.0), Vector::vector(0.0, 0.0, 1.0), Matrix::id()                  , -6.0, -4.0 ; "behind" )]
    #[test_case(Vector::point(0.0, 0.0, -5.0), Vector::vector(0.0, 0.0, 1.0), Matrix::scaling(2.0, 2.0, 2.0),  3.0,  7.0 ; "scaled" )]
    #[test_case(Vector::point(0.0, 1.0, -5.0), Vector::vector(0.0, 0.0, 1.0), Matrix::id()                  ,  5.0,  5.0 ; "tangent")]
    fn ray_sphere_hit(origin: Vector, direction: Vector, transform: Matrix, t1: f64, t2: f64) {
        let ray = Ray { origin, direction };
        let sphere = Shape::sphere(ShapeArgs {
            transform,
            ..ShapeArgs::default()
        });
        let mut intersections = Intersections::new();
        sphere.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(
            is.len() == 2
                && is[0].t.approx(&t1)
                && is[0].shape == &sphere
                && is[1].t.approx(&t2)
                && is[1].shape == &sphere
        )
    }

    #[test_case(Vector::point(0.0, 2.0, -5.0), Vector::vector(0.0, 0.0, 1.0), Matrix::id()                       ; "no transform")]
    #[test_case(Vector::point(0.0, 0.0, -5.0), Vector::vector(0.0, 0.0, 1.0), Matrix::translation(5.0, 0.0, 0.0) ; "translated"  )]
    fn ray_sphere_miss(origin: Vector, direction: Vector, transform: Matrix) {
        let ray = Ray { origin, direction };
        let sphere = Element::sphere(ShapeArgs {
            transform,
            ..ShapeArgs::default()
        });
        let mut intersections = Intersections::new();
        sphere.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 0)
    }

    #[test_case(Vector::point(1.0, 0.0, 0.0), Vector::vector(1.0, 0.0, 0.0), Matrix::id(); "x axis"    )]
    #[test_case(Vector::point(0.0, 1.0, 0.0), Vector::vector(0.0, 1.0, 0.0), Matrix::id(); "y axis"    )]
    #[test_case(Vector::point(0.0, 0.0, 1.0), Vector::vector(0.0, 0.0, 1.0), Matrix::id(); "z axis"    )]
    #[test_case(
        Vector::point(3.0f64.sqrt() / 3.0, 3.0f64.sqrt() / 3.0, 3.0f64.sqrt() / 3.0),
        Vector::vector(3.0f64.sqrt() / 3.0, 3.0f64.sqrt() / 3.0, 3.0f64.sqrt() / 3.0),
        Matrix::id() ;
        "non axial"
    )]
    #[test_case(
        Vector::point(0.0, 1.70711, -0.70711),
        Vector::vector(0.0, 0.70711, -0.70711),
        Matrix::translation(0.0, 1.0, 0.0) ;
        "translated"
    )]
    #[test_case(
        Vector::point(0.0, 2.0f64.sqrt() / 2.0, 2.0f64.sqrt() / -2.0),
        Vector::vector(0.0, 0.97014, -0.24254),
        Matrix::scaling(1.0, 0.5, 1.0) * Matrix::rotation_z(std::f64::consts::PI / 5.0);
        "scaling and rotation"
    )]
    fn sphere_normal(point: Vector, expected: Vector, transform: Matrix) {
        let sphere = Shape::sphere(ShapeArgs {
            transform,
            ..ShapeArgs::default()
        });
        let normal = sphere.normal(point, None, None);

        assert!(normal.approx(&expected))
    }

    #[test]
    fn sphere_normal_normalized() {
        let sphere = Shape::sphere(ShapeArgs::default());
        let normal = sphere.normal(
            Vector::point(
                3.0f64.sqrt() / 3.0,
                3.0f64.sqrt() / 3.0,
                3.0f64.sqrt() / 3.0,
            ),
            None,
            None,
        );

        assert!(normal.approx(&normal.normalize()))
    }

    #[test]
    fn sphere_bbox() {
        let sphere = Element::sphere(ShapeArgs {
            transform: Matrix::translation(1.0, -3.0, 5.0) * Matrix::scaling(0.5, 2.0, 4.0),
            ..ShapeArgs::default()
        });
        let bbox = sphere.bbox();

        assert!(
            bbox.min.approx(&Vector::point(0.5, -5.0, 1.0))
                && bbox.max.approx(&Vector::point(1.5, -1.0, 9.0))
        )
    }

    // Plane Tests

    #[test_case(Vector::point(0.0,  1.0, 0.0), Vector::vector(0.0, -1.0, 0.0), 1.0 ; "above")]
    #[test_case(Vector::point(0.0, -1.0, 0.0), Vector::vector(0.0,  1.0, 0.0), 1.0 ; "below")]
    fn ray_plane_hit(origin: Vector, direction: Vector, t: f64) {
        let plane = Shape::plane(ShapeArgs::default());
        let ray = Ray { origin, direction };
        let mut intersections = Intersections::new();
        plane.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 1 && is[0].t.approx(&t) && is[0].shape == &plane)
    }

    #[test_case(Vector::point(0.0, 10.0, 0.0), Vector::vector(0.0, 0.0, 1.0) ; "parallel")]
    #[test_case(Vector::point(0.0,  0.0, 0.0), Vector::vector(0.0, 0.0, 1.0) ; "coplanar")]
    fn ray_plane_miss(origin: Vector, direction: Vector) {
        let plane = Shape::plane(ShapeArgs::default());
        let ray = Ray { origin, direction };
        let mut intersections = Intersections::new();
        plane.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 0)
    }

    #[test_case(Vector::point(  0.0, 0.0,    0.0), Vector::vector(0.0, 1.0, 0.0) ; "example 1")]
    #[test_case(Vector::point( 10.0, 0.0, - 10.0), Vector::vector(0.0, 1.0, 0.0) ; "example 2")]
    #[test_case(Vector::point(- 5.0, 0.0,  150.0), Vector::vector(0.0, 1.0, 0.0) ; "example 3")]
    fn plane_normal(point: Vector, expected: Vector) {
        let plane = Shape::plane(ShapeArgs::default());
        let normal = plane.normal(point, None, None);

        assert!(normal.approx(&expected))
    }

    // Cube Tests

    #[test_case(Vector::point( 5.0,  0.5,  0.0), Vector::vector(-1.0,  0.0,  0.0),  4.0, 6.0 ; "positive x")]
    #[test_case(Vector::point(-5.0,  0.5,  0.0), Vector::vector( 1.0,  0.0,  0.0),  4.0, 6.0 ; "negative x")]
    #[test_case(Vector::point( 0.5,  5.0,  0.0), Vector::vector( 0.0, -1.0,  0.0),  4.0, 6.0 ; "positive y")]
    #[test_case(Vector::point( 0.5, -5.0,  0.0), Vector::vector( 0.0,  1.0,  0.0),  4.0, 6.0 ; "negative y")]
    #[test_case(Vector::point( 0.5,  0.0,  5.0), Vector::vector( 0.0,  0.0, -1.0),  4.0, 6.0 ; "positive z")]
    #[test_case(Vector::point( 0.5,  0.0, -5.0), Vector::vector( 0.0,  0.0,  1.0),  4.0, 6.0 ; "negative z")]
    #[test_case(Vector::point( 0.0,  0.5,  0.0), Vector::vector( 0.0,  0.0,  1.0), -1.0, 1.0 ; "inside")]
    fn ray_cube_hit(origin: Vector, direction: Vector, t1: f64, t2: f64) {
        let cube = Shape::cube(ShapeArgs::default());
        let ray = Ray { origin, direction };
        let mut intersections = Intersections::new();
        cube.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(
            is.len() == 2
                && is[0].t.approx(&t1)
                && is[0].shape == &cube
                && is[1].t.approx(&t2)
                && is[1].shape == &cube
        )
    }

    #[test_case(Vector::point(-2.0,  0.0,  0.0), Vector::vector( 0.2673,  0.5345,  0.8018) ; "1")]
    #[test_case(Vector::point( 0.0, -2.0,  0.0), Vector::vector( 0.8018,  0.2673,  0.5345) ; "2")]
    #[test_case(Vector::point( 0.0,  0.0, -2.0), Vector::vector( 0.5345,  0.8018,  0.2673) ; "3")]
    #[test_case(Vector::point( 2.0,  0.0,  2.0), Vector::vector( 0.0   ,  0.0   , -1.0   ) ; "4")]
    #[test_case(Vector::point( 0.0,  2.0,  2.0), Vector::vector( 0.0   , -1.0   ,  0.0   ) ; "5")]
    #[test_case(Vector::point( 2.0,  2.0,  0.0), Vector::vector(-1.0   ,  0.0   ,  0.0   ) ; "6")]
    fn ray_cube_miss(origin: Vector, direction: Vector) {
        let cube = Shape::cube(ShapeArgs::default());
        let ray = Ray { origin, direction };
        let mut intersections = Intersections::new();
        cube.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 0)
    }

    #[test_case(Vector::point( 1.0,  0.5, -0.8), Vector::vector( 1.0,  0.0,  0.0); "example 1")]
    #[test_case(Vector::point(-1.0, -0.2,  0.9), Vector::vector(-1.0,  0.0,  0.0); "example 2")]
    #[test_case(Vector::point(-0.4,  1.0, -0.1), Vector::vector( 0.0,  1.0,  0.0); "example 3")]
    #[test_case(Vector::point( 0.3, -1.0, -0.7), Vector::vector( 0.0, -1.0,  0.0); "example 4")]
    #[test_case(Vector::point(-0.6,  0.3,  1.0), Vector::vector( 0.0,  0.0,  1.0); "example 5")]
    #[test_case(Vector::point( 0.4,  0.4, -1.0), Vector::vector( 0.0,  0.0, -1.0); "example 6")]
    #[test_case(Vector::point( 1.0,  1.0,  1.0), Vector::vector( 1.0,  0.0,  0.0); "example 7")]
    #[test_case(Vector::point(-1.0, -1.0, -1.0), Vector::vector(-1.0,  0.0,  0.0); "example 8")]
    fn cube_normal(point: Vector, expected: Vector) {
        let cube = Shape::cube(ShapeArgs::default());
        let normal = cube.normal(point, None, None);

        assert!(normal.approx(&expected))
    }

    // Cylinder Tests

    #[test_case(Vector::point(0.0, 0.0, -5.0), Vector::vector(0.0, 0.0, 1.0), 4.0    , 6.0     ; "example 1")]
    #[test_case(Vector::point(0.5, 0.0, -5.0), Vector::vector(0.1, 1.0, 1.0), 6.80798, 7.08872 ; "example 2")]
    #[test_case(Vector::point(1.0, 0.0, -5.0), Vector::vector(0.0, 0.0, 1.0), 5.0    , 5.0     ; "tangent"  )]
    fn ray_cylinder_hit(origin: Vector, direction: Vector, t0: f64, t1: f64) {
        let cylinder = Shape::cylinder(
            ShapeArgs::default(),
            f64::NEG_INFINITY,
            f64::INFINITY,
            false,
        );
        let ray = Ray {
            origin,
            direction: direction.normalize(),
        };
        let mut intersections = Intersections::new();
        cylinder.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(
            is.len() == 2
                && is[0].t.approx(&t0)
                && is[0].shape == &cylinder
                && is[1].t.approx(&t1)
                && is[1].shape == &cylinder
        )
    }

    #[test_case(Vector::point(1.0, 0.0,  0.0), Vector::vector(0.0, 1.0, 0.0) ; "example 1")]
    #[test_case(Vector::point(0.0, 0.0,  0.0), Vector::vector(0.0, 1.0, 0.0) ; "example 2")]
    #[test_case(Vector::point(1.0, 0.0, -5.0), Vector::vector(1.0, 1.0, 1.0) ; "example 3")]
    fn ray_cylinder_miss(origin: Vector, direction: Vector) {
        let cylinder = Shape::cylinder(
            ShapeArgs::default(),
            f64::NEG_INFINITY,
            f64::INFINITY,
            false,
        );
        let ray = Ray {
            origin,
            direction: direction.normalize(),
        };
        let mut intersections = Intersections::new();
        cylinder.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 0)
    }

    #[test_case(Vector::point(0.0, 1.5,  0.0), Vector::vector(0.1, 1.0, 0.0), 0 ; "example 1")]
    #[test_case(Vector::point(0.0, 3.0, -5.0), Vector::vector(0.0, 0.0, 1.0), 0 ; "example 2")]
    #[test_case(Vector::point(0.0, 0.0, -5.0), Vector::vector(0.0, 0.0, 1.0), 0 ; "example 3")]
    #[test_case(Vector::point(0.0, 2.0, -5.0), Vector::vector(0.0, 0.0, 1.0), 0 ; "example 4")]
    #[test_case(Vector::point(0.0, 1.0, -5.0), Vector::vector(0.0, 0.0, 1.0), 0 ; "example 5")]
    #[test_case(Vector::point(0.0, 1.5, -5.0), Vector::vector(0.0, 0.0, 1.0), 2 ; "example 6")]
    fn ray_cylinder_constrained(origin: Vector, direction: Vector, count: usize) {
        let cylinder = Shape::cylinder(ShapeArgs::default(), 1.0, 2.0, false);
        let ray = Ray {
            origin,
            direction: direction.normalize(),
        };
        let mut intersections = Intersections::new();
        cylinder.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == count)
    }

    #[test_case(Vector::point(0.0,  3.0,  0.0), Vector::vector(0.0, -1.0, 0.0), 2 ; "example 1")]
    #[test_case(Vector::point(0.0,  3.0, -2.0), Vector::vector(0.0, -1.0, 2.0), 2 ; "example 2")]
    #[test_case(Vector::point(0.0,  4.0, -2.0), Vector::vector(0.0, -1.0, 1.0), 2 ; "example 3")]
    #[test_case(Vector::point(0.0,  0.0, -2.0), Vector::vector(0.0,  1.0, 2.0), 2 ; "example 4")]
    #[test_case(Vector::point(0.0, -1.0, -2.0), Vector::vector(0.0,  1.0, 1.0), 2 ; "example 5")]
    fn ray_cylinder_constrained_closed(origin: Vector, direction: Vector, count: usize) {
        let cylinder = Shape::cylinder(ShapeArgs::default(), 1.0, 2.0, true);
        let ray = Ray {
            origin,
            direction: direction.normalize(),
        };
        let mut intersections = Intersections::new();
        cylinder.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == count)
    }

    #[test_case(Vector::point( 1.0,  0.0,  0.0), Vector::vector( 1.0, 0.0,  0.0) ; "example 1")]
    #[test_case(Vector::point( 0.0,  5.0, -1.0), Vector::vector( 0.0, 0.0, -1.0) ; "example 2")]
    #[test_case(Vector::point( 0.0, -2.0,  1.0), Vector::vector( 0.0, 0.0,  1.0) ; "example 3")]
    #[test_case(Vector::point(-1.0,  1.0,  0.0), Vector::vector(-1.0, 0.0,  0.0) ; "example 4")]
    fn cylinder_normal(point: Vector, expected: Vector) {
        let cylinder = Shape::cylinder(
            ShapeArgs::default(),
            f64::NEG_INFINITY,
            f64::INFINITY,
            false,
        );
        let normal = cylinder.normal(point, None, None);

        assert!(normal.approx(&expected))
    }

    #[test_case(Vector::point(0.0, 1.0, 0.0), Vector::vector(0.0, -1.0, 0.0) ; "example 1")]
    #[test_case(Vector::point(0.5, 1.0, 0.0), Vector::vector(0.0, -1.0, 0.0) ; "example 2")]
    #[test_case(Vector::point(0.0, 1.0, 0.5), Vector::vector(0.0, -1.0, 0.0) ; "example 3")]
    #[test_case(Vector::point(0.0, 2.0, 0.0), Vector::vector(0.0,  1.0, 0.0) ; "example 4")]
    #[test_case(Vector::point(0.5, 2.0, 0.0), Vector::vector(0.0,  1.0, 0.0) ; "example 5")]
    #[test_case(Vector::point(0.0, 2.0, 0.5), Vector::vector(0.0,  1.0, 0.0) ; "example 6")]
    fn cylinder_normal_cap(point: Vector, expected: Vector) {
        let cylinder = Shape::cylinder(ShapeArgs::default(), 1.0, 2.0, true);
        let normal = cylinder.normal(point, None, None);

        assert!(normal.approx(&expected))
    }

    // Cone Tests

    #[test_case(Vector::point(0.0, 0.0, -5.0), Vector::vector( 0.0,  0.0, 1.0), 5.0    ,  5.0     ; "example 1")]
    #[test_case(Vector::point(0.0, 0.0, -5.0), Vector::vector( 1.0,  1.0, 1.0), 8.66025,  8.66025 ; "example 2")]
    #[test_case(Vector::point(1.0, 1.0, -5.0), Vector::vector(-0.5, -1.0, 1.0), 4.55006, 49.44994 ; "example 3")]
    fn ray_cone_hit(origin: Vector, direction: Vector, t0: f64, t1: f64) {
        let cone = Shape::cone(
            ShapeArgs::default(),
            f64::NEG_INFINITY,
            f64::INFINITY,
            false,
        );
        let ray = Ray {
            origin,
            direction: direction.normalize(),
        };
        let mut intersections = Intersections::new();
        cone.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(
            is.len() == 2
                && is[0].t.approx(&t0)
                && is[0].shape == &cone
                && is[1].t.approx(&t1)
                && is[1].shape == &cone
        )
    }

    #[test]
    fn ray_cone_parallel_hit() {
        let cone = Shape::cone(
            ShapeArgs::default(),
            f64::NEG_INFINITY,
            f64::INFINITY,
            false,
        );
        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -1.0),
            direction: Vector::vector(0.0, 1.0, 1.0).normalize(),
        };
        let mut intersections = Intersections::new();
        cone.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 1 && is[0].t.approx(&0.35355) && is[0].shape == &cone)
    }

    #[test_case(Vector::point(0.0, 0.0, -5.0 ), Vector::vector(0.0, 1.0, 0.0), 0 ; "example 1")]
    #[test_case(Vector::point(0.0, 0.0, -0.25), Vector::vector(0.0, 1.0, 1.0), 2 ; "example 2")]
    #[test_case(Vector::point(0.0, 0.0, -0.25), Vector::vector(0.0, 1.0, 0.0), 4 ; "example 3")]
    fn ray_cone_contrained(origin: Vector, direction: Vector, count: usize) {
        let cone = Shape::cone(ShapeArgs::default(), -0.5, 0.5, true);
        let ray = Ray {
            origin,
            direction: direction.normalize(),
        };
        let mut intersections = Intersections::new();
        cone.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == count)
    }

    #[test_case(Vector::point( 0.0,  0.0, 0.0), Vector::vector( 0.0,  0.0          , 0.0) ; "example 1")]
    #[test_case(Vector::point( 1.0,  1.0, 1.0), Vector::vector( 1.0, -2.0f64.sqrt(), 1.0) ; "example 2")]
    #[test_case(Vector::point(-1.0, -1.0, 0.0), Vector::vector(-1.0,  1.0          , 0.0) ; "example 3")]
    fn cone_normal(point: Vector, expected: Vector) {
        let cone = Shape::cone(ShapeArgs::default(), f64::NEG_INFINITY, f64::INFINITY, true);
        let normal = cone.geometry.normal(point, None, None);

        assert!(normal.approx(&expected))
    }

    // Triangle Tests

    #[test]
    fn ray_triangle_hit() {
        let triangle = Shape::triangle(
            ShapeArgs::default(),
            Vector::point(0.0, 1.0, 0.0),
            Vector::point(-1.0, 0.0, 0.0),
            Vector::point(1.0, 0.0, 0.0),
        );
        let ray = Ray {
            origin: Vector::point(0.0, 0.5, -2.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };
        let mut intersections = Intersections::new();
        triangle.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 1 && is[0].t.approx(&2.0) && is[0].shape == &triangle)
    }

    #[test_case(Vector::point( 0.0, -1.0, -2.0), Vector::vector(0.0, 1.0, 0.0) ; "parallel"  )]
    #[test_case(Vector::point( 1.0,  1.0, -2.0), Vector::vector(0.0, 0.0, 1.0) ; "p1 p3 edge")]
    #[test_case(Vector::point(-1.0,  1.0, -2.0), Vector::vector(0.0, 0.0, 1.0) ; "p1 p2 edge")]
    #[test_case(Vector::point( 0.0, -1.0, -2.0), Vector::vector(0.0, 0.0, 1.0) ; "p2 p3 edge")]
    fn ray_triangle_miss(origin: Vector, direction: Vector) {
        let triangle = Shape::triangle(
            ShapeArgs::default(),
            Vector::point(0.0, 1.0, 0.0),
            Vector::point(-1.0, 0.0, 0.0),
            Vector::point(1.0, 0.0, 0.0),
        );
        let ray = Ray { origin, direction };
        let mut intersections = Intersections::new();
        triangle.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 0)
    }

    // Smooth Triangle Tests

    fn smooth_triangle() -> Shape {
        Shape::smooth_triangle(
            ShapeArgs::default(),
            Vector::point(0.0, 1.0, 0.0),
            Vector::point(-1.0, 0.0, 0.0),
            Vector::point(1.0, 0.0, 0.0),
            Vector::vector(0.0, 1.0, 0.0),
            Vector::vector(-1.0, 0.0, 0.0),
            Vector::vector(1.0, 0.0, 0.0),
        )
    }

    #[test]
    fn ray_smooth_triangle_u_v() {
        let ray = Ray {
            origin: Vector::point(-0.2, 0.3, -2.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };
        let triangle = smooth_triangle();
        let mut intersections = Intersections::new();
        triangle.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is[0].u.unwrap().approx(&0.44999) && is[0].v.unwrap().approx(&0.24999))
    }

    #[test]
    fn smooth_triangle_normal() {
        let triangle = smooth_triangle();
        let normal = triangle.normal(Vector::point(0.0, 0.0, 0.0), Some(0.45), Some(0.25));

        assert!(normal.approx(&Vector::vector(-0.5547, 0.83205, 0.0)))
    }

    // Group Tests

    #[test]
    fn ray_group_miss() {
        let group = Element::composite(Matrix::id(), None, GroupKind::Aggregation, vec![]);
        let ray = Ray {
            origin: Vector::point(0.0, 0.0, 0.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };
        let mut intersections = Intersections::new();
        group.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 0)
    }

    fn shape(element: &Element) -> &Shape {
        match element {
            Element::Composite(_) => {
                panic!("Expected primitive shape, found group.")
            }
            Element::Primitive(shape) => shape,
        }
    }

    #[test]
    fn ray_group_hit() {
        let sphere1 = Shape::sphere(ShapeArgs::default());
        let sphere2 = Shape::sphere(ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, -3.0),
            ..ShapeArgs::default()
        });
        let sphere3 = Shape::sphere(ShapeArgs {
            transform: Matrix::translation(5.0, 0.0, 0.0),
            ..ShapeArgs::default()
        });

        let group = Element::composite(
            Matrix::id(),
            None,
            GroupKind::Aggregation,
            vec![
                Element::Primitive(sphere1),
                Element::Primitive(sphere2),
                Element::Primitive(sphere3),
            ],
        );

        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };

        let mut intersections = Intersections::new();
        group.intersect(ray, &mut intersections);
        intersections.sort();
        let is: Vec<Intersection> = intersections.into_iter().collect();

        if let Element::Composite(Group { children, .. }) = &group {
            assert!(
                is.len() == 4
                    && is[0].shape == shape(&children[1])
                    && is[1].shape == shape(&children[1])
                    && is[2].shape == shape(&children[0])
                    && is[3].shape == shape(&children[0])
            )
        }
    }

    #[test]
    fn ray_group_hit_transformed() {
        let sphere = Element::sphere(ShapeArgs {
            transform: Matrix::translation(5.0, 0.0, 0.0),
            ..ShapeArgs::default()
        });
        let group = Element::composite(
            Matrix::scaling(2.0, 2.0, 2.0),
            None,
            GroupKind::Aggregation,
            vec![sphere],
        );
        let ray = Ray {
            origin: Vector::point(10.0, 0.0, -10.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };
        let mut intersections = Intersections::new();
        group.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 2)
    }

    #[test]
    fn group_normal() {
        let sphere = Element::sphere(ShapeArgs {
            transform: Matrix::translation(5.0, 0.0, 0.0),
            ..ShapeArgs::default()
        });
        let group2 = Element::composite(
            Matrix::scaling(1.0, 2.0, 3.0),
            None,
            GroupKind::Aggregation,
            vec![sphere],
        );
        let group1 = Element::composite(
            Matrix::rotation_y(std::f64::consts::PI / 2.0),
            None,
            GroupKind::Aggregation,
            vec![group2],
        );

        if let Element::Composite(Group { children, .. }) = group1 {
            if let Element::Composite(Group { children, .. }) = &children[0] {
                if let Element::Primitive(shape) = &children[0] {
                    let normal = shape.normal(Vector::point(1.7321, 1.1547, -5.5774), None, None);
                    let expected = Vector::vector(0.285703, 0.428543, -0.857160);

                    assert!(normal.approx(&expected))
                }
            }
        }
    }

    #[test]
    fn group_bbox() {
        let sphere = Element::sphere(ShapeArgs {
            transform: Matrix::translation(2.0, 5.0, -3.0) * Matrix::scaling(2.0, 2.0, 2.0),
            ..ShapeArgs::default()
        });
        let cylinder = Element::cylinder(
            ShapeArgs {
                transform: Matrix::translation(-4.0, -1.0, 4.0) * Matrix::scaling(0.5, 1.0, 0.5),
                ..ShapeArgs::default()
            },
            -2.0,
            2.0,
            true,
        );
        let group = Element::composite(
            Matrix::id(),
            None,
            GroupKind::Aggregation,
            vec![sphere, cylinder],
        );
        let bbox = group.bbox();

        assert!(
            bbox.min.approx(&Vector::point(-4.5, -3.0, -5.0))
                && bbox.max.approx(&Vector::point(4.0, 7.0, 4.5))
        )
    }

    #[test_case(GroupKind::Union, true , true , true , false ; "union 1")]
    #[test_case(GroupKind::Union, true , true , false, true  ; "union 2")]
    #[test_case(GroupKind::Union, true , false, true , false ; "union 3")]
    #[test_case(GroupKind::Union, true , false, false, true  ; "union 4")]
    #[test_case(GroupKind::Union, false, true , true , false ; "union 5")]
    #[test_case(GroupKind::Union, false, true , false, false ; "union 6")]
    #[test_case(GroupKind::Union, false, false, true , true  ; "union 7")]
    #[test_case(GroupKind::Union, false, false, false, true  ; "union 8")]
    #[test_case(GroupKind::Intersection, true , true , true , true  ; "intersection 1")]
    #[test_case(GroupKind::Intersection, true , true , false, false ; "intersection 2")]
    #[test_case(GroupKind::Intersection, true , false, true , true  ; "intersection 3")]
    #[test_case(GroupKind::Intersection, true , false, false, false ; "intersection 4")]
    #[test_case(GroupKind::Intersection, false, true , true , true  ; "intersection 5")]
    #[test_case(GroupKind::Intersection, false, true , false, true  ; "intersection 6")]
    #[test_case(GroupKind::Intersection, false, false, true , false ; "intersection 7")]
    #[test_case(GroupKind::Intersection, false, false, false, false ; "intersection 8")]
    #[test_case(GroupKind::Difference, true , true , true , false ; "difference 1")]
    #[test_case(GroupKind::Difference, true , true , false, true  ; "difference 2")]
    #[test_case(GroupKind::Difference, true , false, true , false ; "difference 3")]
    #[test_case(GroupKind::Difference, true , false, false, true  ; "difference 4")]
    #[test_case(GroupKind::Difference, false, true , true , true  ; "difference 5")]
    #[test_case(GroupKind::Difference, false, true , false, true  ; "difference 6")]
    #[test_case(GroupKind::Difference, false, false, true , false ; "difference 7")]
    #[test_case(GroupKind::Difference, false, false, false, false ; "difference 8")]
    fn allowed_intersection(
        kind: GroupKind,
        left_hit: bool,
        in_left: bool,
        in_right: bool,
        expected: bool,
    ) {
        assert!(kind
            .allows_intersection(left_hit, in_left, in_right)
            .approx(&expected))
    }

    #[test]
    fn ray_csg_miss() {
        let sphere = Element::sphere(ShapeArgs::default());
        let cube = Element::cube(ShapeArgs::default());
        let group = Element::composite(Matrix::id(), None, GroupKind::Union, vec![sphere, cube]);
        let ray = Ray {
            origin: Vector::point(0.0, 2.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };
        let mut intersections = Intersections::new();
        group.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(is.len() == 0)
    }

    #[test]
    fn ray_csg_hit() {
        let sphere1 = Shape::sphere(ShapeArgs::default());
        let sphere2 = Shape::sphere(ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, 0.5),
            ..ShapeArgs::default()
        });
        let group = Element::composite(
            Matrix::id(),
            None,
            GroupKind::Union,
            vec![
                Element::Primitive(sphere1.clone()),
                Element::Primitive(sphere2.clone()),
            ],
        );
        let ray = Ray {
            origin: Vector::point(0.0, 0.0, -5.0),
            direction: Vector::vector(0.0, 0.0, 1.0),
        };
        let mut intersections = Intersections::new();
        group.intersect(ray, &mut intersections);
        let is: Vec<Intersection> = intersections.into_iter().collect();

        assert!(
            is.len() == 2
                && is[0].t.approx(&4.0)
                && is[0].shape == &sphere1
                && is[1].t.approx(&6.5)
                && is[1].shape == &sphere2
        )
    }

    // Lighting Tests

    #[test_case(
        Vector::vector(0.0, 0.0, -1.0),
        Vector::point(0.0, 0.0, -10.0),
        false,
        Color::new(1.9, 1.9, 1.9) ;
        "light eye surface"
    )]
    #[test_case(
        Vector::vector(0.0, 2.0f64.sqrt() / 2.0,2.0f64.sqrt() / -2.0),
        Vector::point(0.0, 0.0, -10.0),
        false,
        Color::white() ;
        "light eye surface (eye offset)"
    )]
    #[test_case(
        Vector::vector(0.0, 0.0, -1.0),
        Vector::point(0.0, 10.0, -10.0),
        false,
        Color::new(0.7364, 0.7364, 0.7364) ;
        "eye light surface (light offset)"
    )]
    #[test_case(
        Vector::vector(0.0, 2.0f64.sqrt() / -2.0, 2.0f64.sqrt() / -2.0),
        Vector::point(0.0, 10.0, -10.0),
        false,
        Color::new(1.6364, 1.6364, 1.6364) ;
        "eye light surface (eye and light offset)"
    )]
    #[test_case(
        Vector::vector(0.0, 0.0, -1.0),
        Vector::point(0.0, 0.0, 10.0),
        false,
        Color::new(0.1, 0.1, 0.1) ;
        "eye surface light"
    )]
    #[test_case(
        Vector::vector(0.0, 0.0, -1.0),
        Vector::point(0.0, 0.0, -10.0),
        true,
        Color::new(0.1, 0.1, 0.1) ;
        "shadowed"
    )]
    fn lightning(eye: Vector, light_origin: Vector, shadowed: bool, expected: Color) {
        let shape = Shape::sphere(ShapeArgs::default());
        let position = Vector::point(0.0, 0.0, 0.0);
        let normal = Vector::vector(0.0, 0.0, -1.0);
        let light = PointLight {
            intensity: Color::white(),
            origin: light_origin,
        };

        let lighting = shape.lighting(light, position, eye, normal, shadowed);

        assert!(lighting.approx(&expected))
    }

    #[test]
    fn lighting_with_stripe_pattern() {
        let shape = Shape::sphere(ShapeArgs {
            material: Material {
                pattern: Pattern::stripes(
                    Matrix::id(),
                    Pattern::plain(Color::white()),
                    Pattern::plain(Color::black()),
                ),
                ambient: 1.0,
                diffuse: 0.0,
                specular: 0.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        });

        let eye = Vector::vector(0.0, 0.0, -1.0);
        let normal = Vector::vector(0.0, 0.0, -1.0);
        let light = PointLight {
            intensity: Color::white(),
            origin: Vector::point(0.0, 0.0, -10.0),
        };

        let color1 = shape.lighting(light, Vector::point(0.9, 0.0, 0.0), eye, normal, false);
        let color2 = shape.lighting(light, Vector::point(1.1, 0.0, 0.0), eye, normal, false);

        assert!(color1.approx(&Color::white()) && color2.approx(&Color::black()))
    }
}
