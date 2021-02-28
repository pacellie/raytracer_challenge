use raytracer::camera::Camera;
use raytracer::color::Color;
use raytracer::image::Image;
use raytracer::light::PointLight;
use raytracer::linalg::{Matrix, Vector};
use raytracer::material::{Material, Pattern};
use raytracer::obj::ObjParser;
use raytracer::shape::{Element, ShapeArgs};
use raytracer::world::World;

use std::f64::consts::PI;
use std::fs;
use std::process::Command;
use std::time::Instant;

fn construct_world() -> (Camera, World) {
    let obj_parser = ObjParser::new("obj/teapot_high.obj");

    let (_ignored, teapot) = obj_parser
        .parse_obj(
            Matrix::rotation_x(-PI / 2.0),
            Material {
                pattern: Pattern::plain(Color::new(0.7, 0.7, 1.0)),
                ambient: 0.1,
                diffuse: 0.6,
                specular: 0.4,
                reflective: 0.1,
                shininess: 5.0,
                ..Material::default()
            },
        )
        .unwrap();

    let material = Material {
        pattern: Pattern::plain(Color::black()),
        ambient: 0.02,
        diffuse: 0.7,
        specular: 0.0,
        reflective: 0.5,
        ..Material::default()
    };

    let floor = Element::plane(ShapeArgs {
        material: material.clone(),
        ..ShapeArgs::default()
    });

    let left_wall = Element::plane(ShapeArgs {
        material: material.clone(),
        transform: Matrix::rotation_y(-PI / 4.0)
            * Matrix::translation(0.0, 0.0, 30.0)
            * Matrix::rotation_x(PI / 2.0),
        ..ShapeArgs::default()
    });

    let right_wall = Element::plane(ShapeArgs {
        material,
        transform: Matrix::rotation_y(PI / 4.0)
            * Matrix::translation(0.0, 0.0, 30.0)
            * Matrix::rotation_x(PI / 2.0),
        ..ShapeArgs::default()
    });

    let elements = vec![floor, left_wall, right_wall, teapot];

    let world = World {
        lights: vec![
            PointLight {
                intensity: Color::new(0.7, 0.7, 0.7),
                origin: Vector::point(-100.0, 100.0, -100.0),
            },
            PointLight {
                intensity: Color::new(0.7, 0.7, 0.7),
                origin: Vector::point(100.0, 100.0, -100.0),
            },
        ],
        elements,
    };

    let camera = Camera::new(
        4096,
        2160,
        1.4,
        Camera::transform(
            Vector::point(0.0, 30.0, -50.0),
            Vector::point(0.0, 0.0, 10.0),
            Vector::vector(0.0, 1.0, 0.0),
        ),
    );

    (camera, world)
}

fn main() {
    let path = "./image/chapter15_teapot";
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
