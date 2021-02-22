use raytracer::camera::Camera;
use raytracer::color::Color;
use raytracer::image::Image;
use raytracer::light::PointLight;
use raytracer::linalg::{Matrix, Vector};
use raytracer::material::{Material, Pattern};
use raytracer::shape::{Element, ShapeArgs};
use raytracer::world::World;

use std::fs;
use std::process::Command;
use std::time::Instant;

fn construct_world() -> (Camera, World) {
    let floor = Element::plane(ShapeArgs {
        material: Material {
            pattern: Pattern::checkers(
                Matrix::rotation_y(0.3) * Matrix::scaling(0.25, 0.25, 0.25),
                Pattern::plain(Color::new(0.5, 0.5, 0.5)),
                Pattern::plain(Color::new(0.75, 0.75, 0.75)),
            ),
            ambient: 0.2,
            diffuse: 0.9,
            specular: 0.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let cylinder = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(-1.0, 0.0, 1.0) * Matrix::scaling(0.5, 1.0, 0.5),
            material: Material {
                pattern: Pattern::plain(Color::new(0.0, 0.0, 0.6)),
                diffuse: 0.1,
                specular: 0.9,
                shininess: 300.0,
                reflective: 0.9,
                ..Material::default()
            },
            ..ShapeArgs::default()
        },
        0.0,
        0.75,
        true,
    );

    let concentric1 = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(1.0, 0.0, 0.0) * Matrix::scaling(0.8, 1.0, 0.8),
            material: Material {
                pattern: Pattern::plain(Color::new(1.0, 1.0, 0.3)),
                ambient: 0.1,
                diffuse: 0.8,
                specular: 0.9,
                shininess: 300.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        },
        0.0,
        0.2,
        false,
    );

    let concentric2 = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(1.0, 0.0, 0.0) * Matrix::scaling(0.6, 1.0, 0.6),
            material: Material {
                pattern: Pattern::plain(Color::new(1.0, 0.9, 0.4)),
                ambient: 0.1,
                diffuse: 0.8,
                specular: 0.9,
                shininess: 300.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        },
        0.0,
        0.3,
        false,
    );

    let concentric3 = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(1.0, 0.0, 0.0) * Matrix::scaling(0.4, 1.0, 0.4),
            material: Material {
                pattern: Pattern::plain(Color::new(1.0, 0.8, 0.5)),
                ambient: 0.1,
                diffuse: 0.8,
                specular: 0.9,
                shininess: 300.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        },
        0.0,
        0.4,
        false,
    );

    let concentric4 = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(1.0, 0.0, 0.0) * Matrix::scaling(0.2, 1.0, 0.2),
            material: Material {
                pattern: Pattern::plain(Color::new(1.0, 0.7, 0.6)),
                ambient: 0.1,
                diffuse: 0.8,
                specular: 0.9,
                shininess: 300.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        },
        0.0,
        0.5,
        true,
    );

    let deco1 = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, -0.75) * Matrix::scaling(0.05, 1.0, 0.05),
            material: Material {
                pattern: Pattern::plain(Color::new(1.0, 0.0, 0.0)),
                ambient: 0.1,
                diffuse: 0.9,
                specular: 0.9,
                shininess: 300.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        },
        0.0,
        0.3,
        true,
    );

    let deco2 = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, -2.25)
                * Matrix::rotation_y(-0.15)
                * Matrix::translation(0.0, 0.0, 1.5)
                * Matrix::scaling(0.05, 1.0, 0.05),
            material: Material {
                pattern: Pattern::plain(Color::new(1.0, 1.0, 0.0)),
                ambient: 0.1,
                diffuse: 0.9,
                specular: 0.9,
                shininess: 300.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        },
        0.0,
        0.3,
        true,
    );

    let deco3 = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, -2.25)
                * Matrix::rotation_y(-0.3)
                * Matrix::translation(0.0, 0.0, 1.5)
                * Matrix::scaling(0.05, 1.0, 0.05),
            material: Material {
                pattern: Pattern::plain(Color::new(0.0, 1.0, 0.0)),
                ambient: 0.1,
                diffuse: 0.9,
                specular: 0.9,
                shininess: 300.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        },
        0.0,
        0.3,
        true,
    );

    let deco4 = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, -2.25)
                * Matrix::rotation_y(-0.45)
                * Matrix::translation(0.0, 0.0, 1.5)
                * Matrix::scaling(0.05, 1.0, 0.05),
            material: Material {
                pattern: Pattern::plain(Color::new(0.0, 1.0, 1.0)),
                ambient: 0.1,
                diffuse: 0.9,
                specular: 0.9,
                shininess: 300.0,
                ..Material::default()
            },
            ..ShapeArgs::default()
        },
        0.0,
        0.3,
        true,
    );

    let glass_cylinder = Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, -1.5) * Matrix::scaling(0.33, 1.0, 0.33),
            material: Material {
                pattern: Pattern::plain(Color::new(0.25, 0.0, 0.0)),
                diffuse: 0.1,
                specular: 0.9,
                shininess: 300.0,
                reflective: 0.9,
                transparency: 0.9,
                refractive_index: 1.5,
                ..Material::default()
            },
            ..ShapeArgs::default()
        },
        0.0001,
        0.5,
        true,
    );

    let world = World {
        lights: vec![PointLight {
            intensity: Color::white(),
            origin: Vector::point(1.0, 6.9, -4.9),
        }],
        elements: vec![
            floor,
            cylinder,
            concentric1,
            concentric2,
            concentric3,
            concentric4,
            deco1,
            deco2,
            deco3,
            deco4,
            glass_cylinder,
        ],
    };

    let camera = Camera::new(
        4096,
        2160,
        0.314,
        Camera::transform(
            Vector::point(8.0, 3.5, -9.0),
            Vector::point(0.0, 0.3, 0.0),
            Vector::vector(0.0, 1.0, 0.0),
        ),
    );

    (camera, world)
}

fn main() {
    let path = "./image/chapter13_title";
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
