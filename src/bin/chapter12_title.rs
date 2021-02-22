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
    let floor_ceiling = Element::cube(ShapeArgs {
        transform: Matrix::scaling(20.0, 7.0, 20.0) * Matrix::translation(0.0, 1.0, 0.0),
        material: Material {
            pattern: Pattern::checkers(
                Matrix::scaling(0.07, 0.07, 0.07),
                Pattern::plain(Color::black()),
                Pattern::plain(Color::new(0.25, 0.25, 0.25)),
            ),
            ambient: 0.25,
            diffuse: 0.7,
            specular: 0.9,
            shininess: 300.0,
            reflective: 0.1,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let walls = Element::cube(ShapeArgs {
        transform: Matrix::scaling(10.0, 10.0, 10.0),
        material: Material {
            pattern: Pattern::checkers(
                Matrix::scaling(0.05, 20.0, 0.05),
                Pattern::plain(Color::new(0.4863, 0.3765, 0.2941)),
                Pattern::plain(Color::new(0.3725, 0.2902, 0.2275)),
            ),
            ambient: 0.1,
            diffuse: 0.7,
            specular: 0.9,
            shininess: 300.0,
            reflective: 0.1,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let table_top = Element::cube(ShapeArgs {
        transform: Matrix::translation(0.0, 3.1, 0.0) * Matrix::scaling(3.0, 0.1, 2.0),
        material: Material {
            pattern: Pattern::stripes(
                Matrix::rotation_y(0.1) * Matrix::scaling(0.05, 0.05, 0.05),
                Pattern::plain(Color::new(0.5529, 0.4235, 0.3255)),
                Pattern::plain(Color::new(0.6588, 0.5098, 0.4000)),
            ),
            ambient: 0.1,
            diffuse: 0.7,
            specular: 0.9,
            shininess: 300.0,
            reflective: 0.2,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let leg1 = Element::cube(ShapeArgs {
        transform: Matrix::translation(2.7, 1.5, -1.7) * Matrix::scaling(0.1, 1.5, 0.1),
        material: Material {
            pattern: Pattern::plain(Color::new(0.5529, 0.4235, 0.3255)),
            ambient: 0.2,
            diffuse: 0.7,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let leg2 = Element::cube(ShapeArgs {
        transform: Matrix::translation(2.7, 1.5, 1.7) * Matrix::scaling(0.1, 1.5, 0.1),
        material: Material {
            pattern: Pattern::plain(Color::new(0.5529, 0.4235, 0.3255)),
            ambient: 0.2,
            diffuse: 0.7,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let leg3 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-2.7, 1.5, -1.7) * Matrix::scaling(0.1, 1.5, 0.1),
        material: Material {
            pattern: Pattern::plain(Color::new(0.5529, 0.4235, 0.3255)),
            ambient: 0.2,
            diffuse: 0.7,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let leg4 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-2.7, 1.5, 1.7) * Matrix::scaling(0.1, 1.5, 0.1),
        material: Material {
            pattern: Pattern::plain(Color::new(0.5529, 0.4235, 0.3255)),
            ambient: 0.2,
            diffuse: 0.7,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let glass_cube = Element::cube(ShapeArgs {
        transform: Matrix::translation(0.0, 3.45001, 0.0)
            * Matrix::rotation_y(0.2)
            * Matrix::scaling(0.25, 0.25, 0.25),
        material: Material {
            pattern: Pattern::plain(Color::new(1.0, 1.0, 0.8)),
            ambient: 0.0,
            diffuse: 0.3,
            specular: 0.9,
            shininess: 300.0,
            reflective: 0.7,
            transparency: 0.7,
            refractive_index: 1.5,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let little_cube1 = Element::cube(ShapeArgs {
        transform: Matrix::translation(1.0, 3.35, -0.9)
            * Matrix::rotation_y(-0.4)
            * Matrix::scaling(0.15, 0.15, 0.15),
        material: Material {
            pattern: Pattern::plain(Color::new(1.0, 0.5, 0.5)),
            reflective: 0.6,
            diffuse: 0.4,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let little_cube2 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-1.5, 3.27, 0.3)
            * Matrix::rotation_y(0.4)
            * Matrix::scaling(0.15, 0.07, 0.15),
        material: Material {
            pattern: Pattern::plain(Color::new(1.0, 1.0, 0.5)),
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let little_cube3 = Element::cube(ShapeArgs {
        transform: Matrix::translation(0.0, 3.25, 1.0)
            * Matrix::rotation_y(0.4)
            * Matrix::scaling(0.2, 0.05, 0.05),
        material: Material {
            pattern: Pattern::plain(Color::new(0.5, 1.0, 0.5)),
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let little_cube4 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-0.6, 3.4, -1.0)
            * Matrix::rotation_y(0.8)
            * Matrix::scaling(0.05, 0.2, 0.05),
        material: Material {
            pattern: Pattern::plain(Color::new(0.5, 0.5, 1.0)),
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let little_cube5 = Element::cube(ShapeArgs {
        transform: Matrix::translation(2.0, 3.4, 1.0)
            * Matrix::rotation_y(0.8)
            * Matrix::scaling(0.05, 0.2, 0.05),
        material: Material {
            pattern: Pattern::plain(Color::new(0.5, 1.0, 1.0)),
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let frame1 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-10.0, 4.0, 1.0) * Matrix::scaling(0.05, 1.0, 1.0),
        material: Material {
            pattern: Pattern::plain(Color::new(0.7098, 0.2471, 0.2196)),
            diffuse: 0.6,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let frame2 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-10.0, 3.4, 2.7) * Matrix::scaling(0.05, 0.4, 0.4),
        material: Material {
            pattern: Pattern::plain(Color::new(0.2667, 0.2706, 0.6902)),
            diffuse: 0.6,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let frame3 = Element::cube(ShapeArgs {
        transform: Matrix::translation(-10.0, 4.6, 2.7) * Matrix::scaling(0.05, 0.4, 0.4),
        material: Material {
            pattern: Pattern::plain(Color::new(0.3098, 0.5961, 0.3098)),
            diffuse: 0.6,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let mirror_frame = Element::cube(ShapeArgs {
        transform: Matrix::translation(-2.0, 3.5, 9.95) * Matrix::scaling(5.0, 1.5, 0.05),
        material: Material {
            pattern: Pattern::plain(Color::new(0.3882, 0.2627, 0.1882)),
            diffuse: 0.6,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let mirror = Element::cube(ShapeArgs {
        transform: Matrix::translation(-2.0, 3.5, 9.95) * Matrix::scaling(4.8, 1.4, 0.06),
        material: Material {
            pattern: Pattern::plain(Color::black()),
            ambient: 0.0,
            diffuse: 0.0,
            specular: 1.0,
            shininess: 300.0,
            reflective: 1.0,
            ..Material::default()
        },
        ..ShapeArgs::default()
    });

    let world = World {
        lights: vec![PointLight {
            intensity: Color::new(1.0, 1.0, 0.9),
            origin: Vector::point(0.0, 6.9, -5.0),
        }],
        elements: vec![
            floor_ceiling,
            walls,
            table_top,
            leg1,
            leg2,
            leg3,
            leg4,
            glass_cube,
            little_cube1,
            little_cube2,
            little_cube3,
            little_cube4,
            little_cube5,
            frame1,
            frame2,
            frame3,
            mirror_frame,
            mirror,
        ],
    };

    let camera = Camera::new(
        4096,
        2160,
        0.785,
        Camera::transform(
            Vector::point(8.0, 6.0, -8.0),
            Vector::point(0.0, 3.0, 0.0),
            Vector::vector(0.0, 1.0, 0.0),
        ),
    );

    (camera, world)
}

fn main() {
    let path = "./image/chapter12_title";
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
