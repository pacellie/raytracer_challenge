use crate::camera::Camera;
use crate::color::Color;
use crate::config::FUEL;
use crate::world::World;

use rayon::prelude::*;

#[derive(Debug)]
pub struct Image {
    hsize: usize,
    vsize: usize,
    pixels: Vec<Color>,
}

impl Image {
    pub fn new(hsize: usize, vsize: usize) -> Image {
        Image {
            hsize,
            vsize,
            pixels: vec![Color::black(); hsize * vsize],
        }
    }

    pub fn render(camera: Camera, world: &World) -> Image {
        let mut pixels = vec![];

        for y in 0..camera.vsize {
            for x in 0..camera.hsize {
                let ray = camera.ray_at_pixel(x, y);
                let color = world.color_at(ray, FUEL);
                pixels.push(color);
            }
        }

        Image {
            hsize: camera.hsize,
            vsize: camera.vsize,
            pixels,
        }
    }

    pub fn par_render(camera: Camera, world: &World) -> Image {
        let pixels: Vec<Color> = (0..(camera.hsize * camera.vsize))
            .into_par_iter()
            .map(|i| {
                let x = i % camera.hsize;
                let y = i / camera.hsize;
                let ray = camera.ray_at_pixel(x, y);
                world.color_at(ray, FUEL)
            })
            .collect();

        Image {
            hsize: camera.hsize,
            vsize: camera.vsize,
            pixels,
        }
    }

    pub fn write(&mut self, x: usize, y: usize, color: Color) {
        let i = self.xy_to_idx(x, y);
        self.pixels[i] = color;
    }

    pub fn read(&self, x: usize, y: usize) -> Color {
        let i = self.xy_to_idx(x, y);
        self.pixels[i]
    }

    pub fn ppm(&self) -> String {
        let mut ppm = format!("P3\n{} {}\n255", self.hsize, self.vsize);

        let mut j = 0;
        for (i, color) in self.pixels.iter().enumerate() {
            let (r, g, b) = color.clamp();

            if i % self.hsize == 0 || j % 5 == 0 {
                ppm.push('\n');
                ppm.push_str(&format!("{} {} {}", r, g, b));
                j = 1;
            } else {
                ppm.push_str(&format!(" {} {} {}", r, g, b));
                j += 1;
            }
        }
        ppm.push('\n');

        ppm
    }

    fn xy_to_idx(&self, x: usize, y: usize) -> usize {
        y * self.hsize + x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;
    use crate::linalg::Vector;

    use std::f64::consts::PI;

    #[test]
    fn rendering_default_world() {
        let from = Vector::point(0.0, 0.0, -5.0);
        let to = Vector::point(0.0, 0.0, 0.0);
        let up = Vector::vector(0.0, 1.0, 0.0);

        let width = 11;
        let height = 11;

        let camera = Camera::new(width, height, PI / 2.0, Camera::transform(from, to, up));

        let world = World::default();

        let image = Image::render(camera, &world);

        assert!(image
            .read(5, 5)
            .approx(&Color::new(0.38066, 0.47583, 0.28550,)))
    }

    #[test]
    fn image_ppm_example_01() {
        let mut image = Image::new(5, 3);

        let color1 = Color::new(1.5, 0.0, 0.0);
        let color2 = Color::new(0.0, 0.5, 0.0);
        let color3 = Color::new(-0.5, 0.0, 1.0);

        image.write(0, 0, color1);
        image.write(2, 1, color2);
        image.write(4, 2, color3);

        let ppm = image.ppm();

        let expected = "P3\n\
            5 3\n\
            255\n\
            255 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
            0 0 0 0 0 0 0 128 0 0 0 0 0 0 0\n\
            0 0 0 0 0 0 0 0 0 0 0 0 0 0 255\n";

        assert_eq!(ppm, expected);
    }

    #[test]
    fn image_ppm_example_02() {
        let mut image = Image::new(9, 2);

        let color = Color::new(1.0, 0.8, 0.6);

        for x in 0..image.hsize {
            for y in 0..image.vsize {
                image.write(x, y, color);
            }
        }

        let ppm = image.ppm();

        let expected = "P3\n\
            9 2\n\
            255\n\
            255 204 153 255 204 153 255 204 153 255 204 153 255 204 153\n\
            255 204 153 255 204 153 255 204 153 255 204 153\n\
            255 204 153 255 204 153 255 204 153 255 204 153 255 204 153\n\
            255 204 153 255 204 153 255 204 153 255 204 153\n";

        assert_eq!(ppm, expected);
    }
}
