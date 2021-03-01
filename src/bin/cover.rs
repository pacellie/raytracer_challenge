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
    let material_white = Material {
        pattern: Pattern::plain(Color::white()),
        diffuse: 0.7,
        ambient: 0.1,
        specular: 0.0,
        reflective: 0.1,
        ..Material::default()
    };

    let material_blue = Material {
        pattern: Pattern::plain(Color::new(0.537, 0.831, 0.914)),
        ..material_white
    };

    let material_red = Material {
        pattern: Pattern::plain(Color::new(0.941, 0.322, 0.388)),
        ..material_white
    };

    let material_purple = Material {
        pattern: Pattern::plain(Color::new(0.373, 0.404, 0.550)),
        ..material_white
    };

    let standard_transform = Matrix::scaling(0.5, 0.5, 0.5) * Matrix::translation(1.0, -1.0, 1.0);
    let large_object = Matrix::scaling(3.5, 3.5, 3.5) * standard_transform;
    let medium_object = Matrix::scaling(3.0, 3.0, 3.0) * standard_transform;
    let small_object = Matrix::scaling(2.0, 2.0, 2.0) * standard_transform;

    let backdrop = Element::plane(ShapeArgs {
        transform: Matrix::translation(0.0, 0.0, 500.0) * Matrix::rotation_x(PI / 2.0),
        material: Material {
            pattern: Pattern::plain(Color::white()),
            ambient: 1.0,
            diffuse: 0.0,
            specular: 0.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let sphere = Element::sphere(ShapeArgs {
        transform: large_object,
        material: Material {
            pattern: Pattern::plain(Color::new(0.373, 0.404, 0.550)),
            diffuse: 0.2,
            ambient: 0.0,
            specular: 1.0,
            shininess: 200.0,
            reflective: 0.7,
            transparency: 0.7,
            refractive_index: 1.5,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let cube1 = Element::cube(ShapeArgs {
        transform: Matrix::translation(4.0, 0.0, 0.0) * medium_object,
        material: material_white.clone(),
        ..ShapeArgs::default()
    });

    let cube2 = Element::cube(ShapeArgs {
        transform: Matrix::translation(8.5, 1.5, -0.5) * large_object,
        material: material_blue.clone(),
        ..ShapeArgs::default()
    });

    let cube3 = Element::cube(ShapeArgs {
        transform: Matrix::translation(0.0, 0.0, 4.0) * large_object,
        material: material_red.clone(),
        ..ShapeArgs::default()
    });

    let cube4 = Element::cube(ShapeArgs {
        transform: Matrix::translation(4.0, 0.0, 4.0) * small_object,
        material: material_white.clone(),
        ..ShapeArgs::default()
    });

    let cube5 = Element::cube(ShapeArgs {
        transform: Matrix::translation(7.5, 0.5, 4.0) * medium_object,
        material: material_purple.clone(),
        ..ShapeArgs::default()
    });

    let cube6 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-0.25, 0.25, 8.0) * medium_object,
        material: material_white.clone(),
        ..ShapeArgs::default()
    });

    let cube7 = Element::cube(ShapeArgs {
        transform: Matrix::translation(4.0, 1.0, 7.5) * large_object,
        material: material_blue.clone(),
        ..ShapeArgs::default()
    });

    let cube8 = Element::cube(ShapeArgs {
        transform: Matrix::translation(10.0, 2.0, 7.5) * medium_object,
        material: material_red.clone(),
        ..ShapeArgs::default()
    });

    let cube9 = Element::cube(ShapeArgs {
        transform: Matrix::translation(8.0, 2.0, 12.0) * small_object,
        material: material_white.clone(),
        ..ShapeArgs::default()
    });

    let cube10 = Element::cube(ShapeArgs {
        transform: Matrix::translation(20.0, 1.0, 9.0) * small_object,
        material: material_white.clone(),
        ..ShapeArgs::default()
    });

    let cube11 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-0.5, -5.0, 0.25) * large_object,
        material: material_blue.clone(),
        ..ShapeArgs::default()
    });

    let cube12 = Element::cube(ShapeArgs {
        transform: Matrix::translation(4.0, -4.0, 0.0) * large_object,
        material: material_red.clone(),
        ..ShapeArgs::default()
    });

    let cube13 = Element::cube(ShapeArgs {
        transform: Matrix::translation(8.5, -4.0, 0.0) * large_object,
        material: material_white.clone(),
        ..ShapeArgs::default()
    });

    let cube14 = Element::cube(ShapeArgs {
        transform: Matrix::translation(0.0, -4.0, 4.0) * large_object,
        material: material_white.clone(),
        ..ShapeArgs::default()
    });

    let cube15 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-0.5, -4.5, 8.0) * large_object,
        material: material_purple.clone(),
        ..ShapeArgs::default()
    });

    let cube16 = Element::cube(ShapeArgs {
        transform: Matrix::translation(0.0, -8.0, 4.0) * large_object,
        material: material_white.clone(),
        ..ShapeArgs::default()
    });

    let cube17 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-0.5, -8.5, 8.0) * large_object,
        material: material_white.clone(),
        ..ShapeArgs::default()
    });

    let group_top = Element::composite(
        Matrix::id(),
        None,
        raytracer::shape::GroupKind::Aggregation,
        vec![
            sphere, cube1, cube2, cube3, cube4, cube5, cube6, cube7, cube8, cube9, cube10,
        ],
    );

    let group_bot = Element::composite(
        Matrix::id(),
        None,
        raytracer::shape::GroupKind::Aggregation,
        vec![cube11, cube12, cube13, cube14, cube15, cube16, cube17],
    );

    let group_all = Element::composite(
        Matrix::id(),
        None,
        raytracer::shape::GroupKind::Aggregation,
        vec![group_top, group_bot],
    );

    let world = World {
        lights: vec![
            PointLight {
                intensity: Color::white(),
                origin: Vector::point(50.0, 100.0, -50.0),
            },
            PointLight {
                intensity: Color::new(0.2, 0.2, 0.2),
                origin: Vector::point(-400.0, 50.0, -10.0),
            },
        ],
        elements: vec![backdrop, group_all],
    };

    let camera = Camera::new(
        4096,
        4096,
        0.785,
        Camera::transform(
            Vector::point(-6.0, 6.0, -10.0),
            Vector::point(6.0, 0.0, 6.0),
            Vector::vector(-0.45, 1.0, 0.0),
        ),
    );

    (camera, world)
}

fn main() {
    let path = "./image/cover";
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
