use raytracer::camera::Camera;
use raytracer::color::Color;
use raytracer::image::Image;
use raytracer::light::PointLight;
use raytracer::linalg::{Matrix, Vector};
use raytracer::material::{Material, Pattern};
use raytracer::shape::{Element, ShapeArgs};
use raytracer::world::World;

use std::f64::consts::PI;
use std::fs;
use std::process::Command;
use std::time::Instant;

fn construct_world() -> (Camera, World) {
    let floor = Element::plane(ShapeArgs {
        transform: Matrix::translation(0.0, -10.0, 0.0),
        material: Material {
            pattern: Pattern::checkers(
                Matrix::id(),
                Pattern::plain(Color::white()),
                Pattern::plain(Color::black()),
            ),
            specular: 0.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let glass = Element::sphere(ShapeArgs {
        material: Material {
            pattern: Pattern::plain(Color::black()),
            diffuse: 0.1,
            shininess: 300.0,
            reflective: 1.0,
            transparency: 1.0,
            refractive_index: 1.52,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let air = Element::sphere(ShapeArgs {
        transform: Matrix::scaling(0.5, 0.5, 0.5),
        material: Material {
            pattern: Pattern::plain(Color::black()),
            diffuse: 0.1,
            shininess: 300.0,
            reflective: 1.0,
            transparency: 1.0,
            refractive_index: 1.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let world = World {
        lights: vec![PointLight {
            intensity: Color::new(0.7, 0.7, 0.7),
            origin: Vector::point(20.0, 10.0, 0.0),
        }],
        elements: vec![floor, glass, air],
    };

    let camera = Camera::new(
        4096,
        4096,
        PI / 3.0,
        Camera::transform(
            Vector::point(0.0, 2.5, 0.0),
            Vector::point(0.0, 0.0, 0.0),
            Vector::vector(0.0, 0.0, 1.0),
        ),
    );

    (camera, world)
}

fn main() {
    let path = "./image/chapter11_glass_air_bubble";
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
