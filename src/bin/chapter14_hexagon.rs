use raytracer::camera::Camera;
use raytracer::color::Color;
use raytracer::image::Image;
use raytracer::light::PointLight;
use raytracer::linalg::{Matrix, Vector};
use raytracer::material::{Material, Pattern};
use raytracer::noise::Noise;
use raytracer::shape::{Element, GroupKind, ShapeArgs};
use raytracer::world::World;

use std::f64::consts::PI;
use std::fs;
use std::process::Command;
use std::time::Instant;

fn hexagon_corner() -> Element {
    Element::sphere(ShapeArgs {
        transform: Matrix::translation(0.0, 0.0, -1.0) * Matrix::scaling(0.25, 0.25, 0.25),
        ..ShapeArgs::default()
    })
}

fn hexagon_edge() -> Element {
    Element::cylinder(
        ShapeArgs {
            transform: Matrix::translation(0.0, 0.0, -1.0)
                * Matrix::rotation_y(-PI / 6.0)
                * Matrix::rotation_z(-PI / 2.0)
                * Matrix::scaling(0.25, 1.0, 0.25),
            ..ShapeArgs::default()
        },
        0.0,
        1.0,
        false,
    )
}

fn hexagon_side(transform: Matrix) -> Element {
    Element::composite(
        transform,
        None,
        GroupKind::Aggregation,
        vec![hexagon_corner(), hexagon_edge()],
    )
}

fn hexagon() -> Element {
    let material: Material = Material {
        pattern: Pattern::point_jitter(
            Noise::Simplex { scale: 0.3 },
            Pattern::stripes(
                Matrix::rotation_z(PI / 2.0) * Matrix::scaling(0.05, 0.05, 0.05),
                Pattern::plain(Color::white()),
                Pattern::plain(Color::black()),
            ),
        ),
        ..Material::default()
    };

    let mut sides = vec![];
    for n in 0..6 {
        sides.push(hexagon_side(Matrix::rotation_y(n as f64 * PI / 3.0)));
    }

    Element::composite(Matrix::id(), Some(material), GroupKind::Aggregation, sides)
}

fn construct_world() -> (Camera, World) {
    let world = World {
        lights: vec![PointLight {
            intensity: Color::white(),
            origin: Vector::point(1.0, 6.9, -4.9),
        }],
        elements: vec![hexagon()],
    };

    let camera = Camera::new(
        4096,
        2160,
        0.314,
        Camera::transform(
            Vector::point(8.0, 5.0, 8.0),
            Vector::point(0.0, 0.0, 0.0),
            Vector::vector(0.0, 1.0, 0.0),
        ),
    );

    (camera, world)
}

fn main() {
    let path = "./image/chapter14_hexagon";
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
    let image = Image::par_render(&camera, &world);
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
