use crate::approx::Approx;

#[derive(Debug, Clone, Copy)]
pub enum Noise {
    Simplex { scale: f64 },
    Fractal { scale: f64, octaves: usize },
}

impl Approx<Noise> for Noise {
    fn approx(&self, other: &Noise) -> bool {
        match (self, other) {
            (Noise::Simplex { scale: sscale }, Noise::Simplex { scale: oscale }) => {
                sscale.approx(oscale)
            }
            (
                Noise::Fractal {
                    scale: sscale,
                    octaves: soctaves,
                },
                Noise::Fractal {
                    scale: oscale,
                    octaves: ooctaves,
                },
            ) => sscale.approx(oscale) && soctaves.approx(ooctaves),
            (_, _) => false,
        }
    }
}

impl Noise {
    pub fn jitter_3d(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        match self {
            Noise::Simplex { scale } => {
                let (nx, ny, nz) = (
                    simplex(x, y, z) * scale,
                    simplex(x, y, z + 1.0) * scale,
                    simplex(x, y, z + 2.0) * scale,
                );

                (x + nx, y + ny, z + nz)
            }
            Noise::Fractal { scale, octaves } => {
                let (nx, ny, nz) = (
                    fractal(x, y, z, *octaves) * scale,
                    fractal(x, y, z + 1.0, *octaves) * scale,
                    fractal(x, y, z + 2.0, *octaves) * scale,
                );

                (x + nx, y + ny, z + nz)
            }
        }
    }
}

#[rustfmt::skip]
const P: [u8; 512] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180,

    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180,
];

const F3: f64 = 1.0 / 3.0;
const G3: f64 = 1.0 / 6.0;

fn hash(i: usize) -> usize {
    P[i] as usize
}

#[rustfmt::skip]
fn grad(hash: usize, x: f64, y: f64, z: f64) -> f64 {
    match hash & 0xF {
        0x0 =>  x + y,
        0x1 => -x + y,
        0x2 =>  x - y,
        0x3 => -x - y,
        0x4 =>  x + z,
        0x5 => -x + z,
        0x6 =>  x - z,
        0x7 => -x - z,
        0x8 =>  y + z,
        0x9 => -y + z,
        0xA =>  y - z,
        0xB => -y - z,
        0xC =>  y + x,
        0xD => -y + z,
        0xE =>  y - x,
        0xF => -y - z,
        _ => unreachable!()
    }
}

fn fast_floor(x: f64) -> i32 {
    if x > 0.0 {
        x as i32
    } else {
        x as i32 - 1
    }
}

fn modulus(x: i32, m: i32) -> usize {
    let a = x % m;
    if a < 0 {
        (a + m) as usize
    } else {
        a as usize
    }
}

#[rustfmt::skip]
fn simplex(x: f64, y: f64, z: f64) -> f64 {
    let s = (x + y + z) * F3;

    let i = fast_floor(x + s);
    let j = fast_floor(y + s);
    let k = fast_floor(z + s);

    let t = (i + j + k) as f64 * G3;

    let x0 = x - (i as f64 - t);
    let y0 = y - (j as f64 - t);
    let z0 = z - (k as f64 - t);

    let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
        if y0 >= z0 {
            (1, 0, 0, 1, 1, 0)
        } else if x0 >= z0 {
            (1, 0, 0, 1, 0, 1)
        } else {
            (0, 0, 1, 1, 0, 1)
        }
    } else {
        if y0 < z0 {
            (0, 0, 1, 0, 1, 1)
        } else if x0 < z0 {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        }
    };

    let x1 = x0 - (i1 as f64) + G3;
    let y1 = y0 - (j1 as f64) + G3;
    let z1 = z0 - (k1 as f64) + G3;

    let x2 = x0 - (i2 as f64) + 2.0 * G3;
    let y2 = y0 - (j2 as f64) + 2.0 * G3;
    let z2 = z0 - (k2 as f64) + 2.0 * G3;

    let x3 = x0 - 1.0 + 3.0 * G3;
    let y3 = y0 - 1.0 + 3.0 * G3;
    let z3 = z0 - 1.0 + 3.0 * G3;

    let ii = modulus(i, 256);
    let jj = modulus(j, 256);
    let kk = modulus(k, 256);

    let gi0 = hash(ii +      hash(jj +      hash(kk     )));
    let gi1 = hash(ii + i1 + hash(jj + j1 + hash(kk + k1)));
    let gi2 = hash(ii + i2 + hash(jj + j2 + hash(kk + k2)));
    let gi3 = hash(ii +  1 + hash(jj +  1 + hash(kk +  1)));

    let mut t0 = 0.6 - x0 * x0 - y0 * y0 - z0 * z0;
    let n0 = if t0 < 0.0 {
        0.0
    } else {
        t0 *= t0;
        t0 * t0 * grad(gi0, x0, y0, z0)
    };

    let mut t1 = 0.6 - x1 * x1 - y1 * y1 - z1 * z1;
    let n1 = if t1 < 0.0 {
        0.0
    } else {
        t1 *= t1;
        t1 * t1 * grad(gi1, x1, y1, z1)
    };

    let mut t2 = 0.6 - x2 * x2 - y2 * y2 - z2 * z2;
    let n2 = if t2 < 0.0 {
        0.0
    } else {
        t2 *= t2;
        t2 * t2 * grad(gi2, x2, y2, z2)
    };

    let mut t3 = 0.6 - x3 * x3 - y3 * y3 - z3 * z3;
    let n3 = if t3 < 0.0 {
        0.0
    } else {
        t3 *= t3;
        t3 * t3 * grad(gi3, x3, y3, z3)
    };

    32.0 * (n0 + n1 + n2 + n3)
}

fn fractal(x: f64, y: f64, z: f64, octaves: usize) -> f64 {
    let mut output = 0.0;
    let mut denom = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let lacunarity = 2.0;
    let persistence = 0.5;

    for _ in 0..octaves {
        output += amplitude * simplex(x * frequency, y * frequency, z * frequency);
        denom += amplitude;
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    output / denom
}
