use raytracer::camera::Camera;
use raytracer::color::Color;
use raytracer::image::Image;
use raytracer::light::PointLight;
use raytracer::linalg::{Matrix, Vector};
use raytracer::material::{Material, Pattern};
use raytracer::shape::{Element, GroupKind, ShapeArgs};
use raytracer::world::World;

use std::f64::consts::PI;
use std::fs;
use std::process::Command;
use std::time::Instant;

fn leg(transform: Matrix) -> Element {
    let sphere = Element::sphere(ShapeArgs {
        transform: Matrix::translation(0.0, 0.0, -1.0) * Matrix::scaling(0.25, 0.25, 0.25),
        ..ShapeArgs::default()
    });

    let cylinder = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, -1.0)
                * Matrix::rotation_y(-0.5236)
                * Matrix::rotation_z(-1.5708)
                * Matrix::scaling(0.25, 1.0, 0.25),
            ..ShapeArgs::default()
        },
        0.0,
        1.0,
        false,
    );

    Element::composite(
        transform,
        None,
        GroupKind::Aggregation,
        vec![sphere, cylinder],
    )
}

fn cap(transform: Matrix) -> Element {
    let mut cones = vec![];

    for i in 0..6 {
        cones.push(Element::cone(
            ShapeArgs {
                transform: Matrix::rotation_y(PI / 3.0 * i as f64)
                    * Matrix::rotation_x(-0.7854)
                    * Matrix::scaling(0.24606, 1.37002, 0.24606),
                ..ShapeArgs::default()
            },
            -1.0,
            0.0,
            false,
        ))
    }

    Element::composite(transform, None, GroupKind::Aggregation, cones)
}

fn wacky(transform: Matrix, material: Material) -> Element {
    let mut elements = vec![];

    for i in 0..6 {
        elements.push(leg(Matrix::rotation_y(PI / 3.0 * i as f64)))
    }

    elements.push(cap(Matrix::translation(0.0, 1.0, 0.0)));
    elements.push(cap(
        Matrix::rotation_x(PI) * Matrix::translation(0.0, 1.0, 0.0)
    ));

    Element::composite(transform, Some(material), GroupKind::Aggregation, elements)
}

fn construct_world() -> (Camera, World) {
    let mut elements = vec![];

    elements.push(Element::plane(ShapeArgs {
        transform: Matrix::translation(0.0, 0.0, 100.0) * Matrix::rotation_x(1.5708),
        material: Material {
            pattern: Pattern::plain(Color::white()),
            ambient: 1.0,
            diffuse: 0.0,
            specular: 0.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    }));

    elements.push(wacky(
        Matrix::translation(-2.8, 0.0, 0.0)
            * Matrix::rotation_x(0.4363)
            * Matrix::rotation_y(0.1745),
        Material {
            pattern: Pattern::plain(Color::new(0.9, 0.2, 0.4)),
            ambient: 0.2,
            diffuse: 0.8,
            specular: 0.7,
            shininess: 20.0,
            ..Material::default()
        },
    ));

    elements.push(wacky(
        Matrix::rotation_y(0.1745),
        Material {
            pattern: Pattern::plain(Color::new(0.2, 0.9, 0.6)),
            ambient: 0.2,
            diffuse: 0.8,
            specular: 0.7,
            shininess: 20.0,
            ..Material::default()
        },
    ));

    elements.push(wacky(
        Matrix::translation(2.8, 0.0, 0.0)
            * Matrix::rotation_x(-0.4363)
            * Matrix::rotation_y(-0.1745),
        Material {
            pattern: Pattern::plain(Color::new(0.2, 0.3, 1.0)),
            ambient: 0.2,
            diffuse: 0.8,
            specular: 0.7,
            shininess: 20.0,
            ..Material::default()
        },
    ));

    let world = World {
        lights: vec![
            PointLight {
                intensity: Color::new(0.25, 0.25, 0.25),
                origin: Vector::point(10000.0, 10000.0, -10000.0),
            },
            PointLight {
                intensity: Color::new(0.25, 0.25, 0.25),
                origin: Vector::point(-10000.0, 10000.0, -10000.0),
            },
            PointLight {
                intensity: Color::new(0.25, 0.25, 0.25),
                origin: Vector::point(10000.0, -10000.0, -10000.0),
            },
            PointLight {
                intensity: Color::new(0.25, 0.25, 0.25),
                origin: Vector::point(-10000.0, -10000.0, -10000.0),
            },
        ],
        elements,
    };

    let camera = Camera::new(
        4096,
        2160,
        0.9,
        Camera::transform(
            Vector::point(0.0, 0.0, -9.0),
            Vector::point(0.0, 0.0, 0.0),
            Vector::vector(0.0, 1.0, 0.0),
        ),
    );

    (camera, world)
}

fn main() {
    let path = "./image/chapter14_title";
    let ppm = &format!("{}.ppm", path);
    let png = &format!("{}.png", path);

    // Build image
    let now = Instant::now();
    print!("Constructing world ...");
    let (camera, world) = construct_world();
    println!(" {} ms.", now.elapsed().as_millis());

    // Clean up old files
    let _ = fs::remove_file(png);

    // Render image
    let now = Instant::now();
    print!("Rendering image ...");
    let image = Image::par_render(camera, &world);
    println!(" {} ms.", now.elapsed().as_millis());

    // Write image to disk
    let now = Instant::now();
    print!("Writing image to disk ...");
    let _ = fs::write(ppm, image.ppm());
    let _ = Command::new("zsh")
        .arg("-c")
        .arg(format!("pnmtopng {} > {}", ppm, png))
        .output();
    let _ = fs::remove_file(ppm);
    println!(" {} ms.", now.elapsed().as_millis());
}
