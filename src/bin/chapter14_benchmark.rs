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

fn sphere_block(n: usize, h: f64) -> Vec<Element> {
    let mut elements = vec![];

    for x in 0..n {
        for z in 0..n {
            for y in 0..n {
                let sphere = Element::sphere(ShapeArgs {
                    transform: Matrix::translation(
                        0.1 + x as f64 * h,
                        0.1 + y as f64 * h,
                        0.1 + z as f64 * h,
                    ) * Matrix::scaling(0.1, 0.1, 0.1),
                    material: Material {
                        pattern: Pattern::plain(Color::black()),
                        ..Material::default()
                    },
                    ..ShapeArgs::default()
                });

                elements.push(sphere);
            }
        }
    }

    elements
}

fn construct_world() -> (Camera, World) {
    let backdrop = Element::plane(ShapeArgs {
        material: Material {
            pattern: Pattern::plain(Color::white()),
            specular: 0.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let n = 5;
    let h = 0.3;

    // let mut spheres = sphere_block(n * 2, h);

    let mut spheres = vec![];
    let offset = (n - 1) as f64 * (0.1 + h) - 0.1;

    for x in 0..2 {
        for z in 0..2 {
            for y in 0..2 {
                let group = Element::composite(
                    Matrix::translation(x as f64 * offset, y as f64 * offset, z as f64 * offset),
                    None,
                    GroupKind::Aggregation,
                    sphere_block(n, h),
                );
                spheres.push(group);
            }
        }
    }

    let mut elements = vec![backdrop];
    elements.push(Element::composite(
        Matrix::id(),
        Some(Material {
            pattern: Pattern::gradient(
                Matrix::scaling(1.0, 2.9, 1.0) * Matrix::rotation_z(PI / 2.0),
                Pattern::plain(Color::new(1.0, 0.0, 0.0)),
                Pattern::plain(Color::new(0.0, 0.0, 1.0)),
            ),
            ..Material::default()
        }),
        GroupKind::Aggregation,
        spheres,
    ));

    let world = World {
        lights: vec![PointLight {
            intensity: Color::white(),
            origin: Vector::point(-5.0, 7.0, -1.0),
        }],
        elements,
    };

    let camera = Camera::new(
        4096,
        2160,
        1.0,
        Camera::transform(
            Vector::point(-8.0, 8.0, -8.0),
            Vector::point(2.0, 2.0, 2.0),
            Vector::vector(0.0, 1.0, 0.0),
        ),
    );

    (camera, world)
}

fn main() {
    let path = "./image/chapter14_benchmark";
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
