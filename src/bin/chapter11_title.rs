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
    let wall_material = Material {
        pattern: Pattern::stripes(
            Matrix::scaling(0.25, 0.25, 0.25) * Matrix::rotation_y(PI / 2.0),
            Pattern::plain(Color::new(0.45, 0.45, 0.45)),
            Pattern::plain(Color::new(0.55, 0.55, 0.55)),
        ),
        ambient: 0.0,
        diffuse: 0.4,
        specular: 0.0,
        reflective: 0.3,
        ..Material::default()
    };

    let floor = Element::plane(ShapeArgs {
        transform: Matrix::rotation_y(0.31415),
        material: Material {
            pattern: Pattern::checkers(
                Matrix::id(),
                Pattern::plain(Color::new(0.35, 0.35, 0.35)),
                Pattern::plain(Color::new(0.65, 0.65, 0.65)),
            ),
            specular: 0.0,
            reflective: 0.4,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let ceiling = Element::plane(ShapeArgs {
        transform: Matrix::translation(0.0, 5.0, 0.0),
        material: Material {
            pattern: Pattern::plain(Color::new(0.8, 0.8, 0.8)),
            ambient: 0.3,
            specular: 0.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let west_wall = Element::plane(ShapeArgs {
        transform: Matrix::translation(-5.0, 0.0, 0.0)
            * Matrix::rotation_z(PI / 2.0)
            * Matrix::rotation_y(PI / 2.0),
        material: wall_material.clone(),
        ..ShapeArgs::default()
    });

    let east_wall = Element::plane(ShapeArgs {
        transform: Matrix::translation(5.0, 0.0, 0.0)
            * Matrix::rotation_z(PI / 2.0)
            * Matrix::rotation_y(PI / 2.0),
        material: wall_material.clone(),
        ..ShapeArgs::default()
    });

    let north_wall = Element::plane(ShapeArgs {
        transform: Matrix::translation(0.0, 0.0, 5.0) * Matrix::rotation_x(PI / 2.0),
        material: wall_material.clone(),
        ..ShapeArgs::default()
    });

    let south_wall = Element::plane(ShapeArgs {
        transform: Matrix::translation(0.0, 0.0, -5.0) * Matrix::rotation_x(PI / 2.0),
        material: wall_material.clone(),
        ..ShapeArgs::default()
    });

    let background1 = Element::sphere(ShapeArgs {
        transform: Matrix::translation(4.6, 0.4, 1.0) * Matrix::scaling(0.4, 0.4, 0.4),
        material: Material {
            pattern: Pattern::plain(Color::new(0.8, 0.5, 0.3)),
            shininess: 50.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let background2 = Element::sphere(ShapeArgs {
        transform: Matrix::translation(4.7, 0.3, 0.4) * Matrix::scaling(0.3, 0.3, 0.3),
        material: Material {
            pattern: Pattern::plain(Color::new(0.9, 0.4, 0.5)),
            shininess: 50.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let background3 = Element::sphere(ShapeArgs {
        transform: Matrix::translation(-1.0, 0.5, 4.5) * Matrix::scaling(0.5, 0.5, 0.5),
        material: Material {
            pattern: Pattern::plain(Color::new(0.4, 0.9, 0.6)),
            shininess: 50.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let background4 = Element::sphere(ShapeArgs {
        transform: Matrix::translation(-1.7, 0.3, 4.7) * Matrix::scaling(0.3, 0.3, 0.3),
        material: Material {
            pattern: Pattern::plain(Color::new(0.4, 0.6, 0.9)),
            shininess: 50.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let foreground1 = Element::sphere(ShapeArgs {
        transform: Matrix::translation(-0.6, 1.0, 0.6),
        material: Material {
            pattern: Pattern::plain(Color::new(1.0, 0.3, 0.2)),
            specular: 0.4,
            shininess: 5.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let foreground2 = Element::sphere(ShapeArgs {
        transform: Matrix::translation(0.6, 0.7, -0.6) * Matrix::scaling(0.7, 0.7, 0.7),
        material: Material {
            pattern: Pattern::plain(Color::new(0.0, 0.0, 0.2)),
            ambient: 0.0,
            diffuse: 0.4,
            specular: 0.9,
            shininess: 300.0,
            reflective: 0.9,
            transparency: 0.9,
            refractive_index: 1.5,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let foreground3 = Element::sphere(ShapeArgs {
        transform: Matrix::translation(-0.7, 0.5, -0.8) * Matrix::scaling(0.5, 0.5, 0.5),
        material: Material {
            pattern: Pattern::plain(Color::new(0.0, 0.2, 0.0)),
            ambient: 0.0,
            diffuse: 0.4,
            specular: 0.9,
            shininess: 300.0,
            reflective: 0.9,
            transparency: 0.9,
            refractive_index: 1.5,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let world = World {
        lights: vec![PointLight {
            intensity: Color::white(),
            origin: Vector::point(-4.9, 4.9, -1.0),
        }],
        elements: vec![
            floor,
            ceiling,
            west_wall,
            east_wall,
            north_wall,
            south_wall,
            background1,
            background2,
            background3,
            background4,
            foreground1,
            foreground2,
            foreground3,
        ],
    };

    let camera = Camera::new(
        4096,
        2160,
        1.152,
        Camera::transform(
            Vector::point(-2.6, 1.5, -3.9),
            Vector::point(-0.6, 1.0, -0.8),
            Vector::vector(0.0, 1.0, 0.0),
        ),
    );

    (camera, world)
}

fn main() {
    let path = "./image/chapter11_title";
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
