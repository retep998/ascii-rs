
use libc::{c_uchar, c_uint, size_t, free, c_void};
use std::cell::{Cell};
use std::io::fs::{File};
use std::mem::{uninitialized};
use std::num::{Zero};
use std::slice::raw::{buf_as_slice};

#[link(name = "lodepng")]
extern {
    fn lodepng_decode32(outbuf: *mut *mut c_uchar,
                        width: *mut c_uint,
                        height: *mut c_uint,
                        inbuf: *const c_uchar,
                        insize: size_t) -> c_uint;
}

#[deriving(Show)]
pub struct Color<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T: Zero> Color<T> {
    pub fn black() -> Color<T> {
        Color {
            r: Zero::zero(),
            g: Zero::zero(),
            b: Zero::zero(),
            a: Zero::zero(),
        }
    }
}

impl<T: Signed> Color<T> {
    pub fn diff(&self, rhs: &Color<T>) -> Color<T> {
        Color {
            r: self.r.abs_sub(&rhs.r),
            g: self.g.abs_sub(&rhs.g),
            b: self.b.abs_sub(&rhs.b),
            a: self.a.abs_sub(&rhs.a),
        }
    }
}

impl<T: Add<T, T>> Color<T> {
    pub fn luminance(&self) -> T {
        self.r + self.g + self.b
    }
}

impl Color<u8> {
    fn to_linear(&self, table: &Table) -> Color<f32> {
        Color {
            r: table.r[self.r as uint],
            g: table.g[self.g as uint],
            b: table.b[self.b as uint],
            a: table.a[self.a as uint],
        }
    }
}

impl Color<f32> {
    pub fn from_rgb(r: u8, g: u8, b: u8, table: &Table) -> Color<f32> {
        Color {
            r: r,
            g: g,
            b: b,
            a: 0xff,
        }.to_linear(table)
    }
}

impl<T: Mul<T, T>> Mul<T, Color<T>> for Color<T> {
    fn mul(&self, rhs: &T) -> Color<T> {
        Color {
            r: self.r.mul(rhs),
            g: self.g.mul(rhs),
            b: self.b.mul(rhs),
            a: self.a.mul(rhs),
        }
    }
}

impl<T: Add<T, T>> Add<Color<T>, Color<T>> for Color<T> {
    fn add(&self, rhs: &Color<T>) -> Color<T> {
        Color {
            r: self.r.add(&rhs.r),
            g: self.g.add(&rhs.g),
            b: self.b.add(&rhs.b),
            a: self.a.add(&rhs.a),
        }
    }
}

impl<T: Sub<T, T>> Sub<Color<T>, Color<T>> for Color<T> {
    fn sub(&self, rhs: &Color<T>) -> Color<T> {
        Color {
            r: self.r.sub(&rhs.r),
            g: self.g.sub(&rhs.g),
            b: self.b.sub(&rhs.b),
            a: self.a.sub(&rhs.a),
        }
    }
}

pub struct Image<T> {
    width: uint,
    height: uint,
    pixels: Vec<Cell<Color<T>>>,
}

impl<T> Image<T> {
    pub fn width(&self) -> uint { self.width }
    pub fn height(&self) -> uint { self.height }
}

impl<T: Zero + Copy> Image<T> {
    pub fn get(&self, x: uint, y: uint) -> Color<T> {
        let i = y * self.width + x;
        match self.pixels.as_slice().get(i) {
            Some(c) => c.get(),
            None => Color::black(),
        }
    }
}

impl Image<u8> {
    pub fn load(path: &Path) -> Result<Image<u8>, &'static str> {
        let mut file = File::open(path);
        let data = match file.read_to_end() {
            Ok(data) => data,
            Err(_) => return Err("Failed to read file"),
        };
        let mut width = unsafe { uninitialized() };
        let mut height = unsafe { uninitialized() };
        let mut outbuf = unsafe { uninitialized() };
        match unsafe { lodepng_decode32(&mut outbuf, &mut width, &mut height,
                                        data.as_ptr(), data.len() as size_t) } {
            0 => (),
            _ => return Err("Failed to decode png data"),
        }
        let pixels = unsafe {
            buf_as_slice(outbuf as *const Color<u8>, (width * height) as uint,
                         |decoded| decoded.iter().map(|&c| Cell::new(c)).collect())
        };
        unsafe { free(outbuf as *mut c_void) };
        Ok(Image {
            width: width as uint,
            height: height as uint,
            pixels: pixels,
        })
    }
    pub fn to_linear(&self, table: &Table, back: Color<u8>) -> Image<f32> {
        let back = back.to_linear(table);
        let pixels = self.pixels.iter().map(|pixel| {
            let pixel = pixel.get().to_linear(table);
            Cell::new(Color {
                r: pixel.r * pixel.a + back.r * (1. - pixel.a),
                g: pixel.g * pixel.a + back.g * (1. - pixel.a),
                b: pixel.b * pixel.a + back.b * (1. - pixel.a),
                a: pixel.a * pixel.a + back.a * (1. - pixel.a),
            })
        }).collect();
        Image { width: self.width, height: self.height, pixels: pixels }
    }
}

pub struct Table {
    r: [f32, ..0x100],
    g: [f32, ..0x100],
    b: [f32, ..0x100],
    a: [f32, ..0x100],
}

impl Table {
    pub fn generate() -> Table {
        fn srgb_to_linear(x: f32) -> f32 {
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / (1. + 0.055)).powf(2.4)
            }
        }
        let mut r: [f32, ..0x100] = unsafe { uninitialized() };
        let mut g: [f32, ..0x100] = unsafe { uninitialized() };
        let mut b: [f32, ..0x100] = unsafe { uninitialized() };
        let mut a: [f32, ..0x100] = unsafe { uninitialized() };
        for i in range(0u, 0x100) {
            let val = srgb_to_linear(i as f32 / 255.);
            r[i] = val * (0.2126 * 100.);
            g[i] = val * (0.7152 * 100.);
            b[i] = val * (0.0722 * 100.);
            a[i] = val;
        }
        Table { r: r, g: g, b: b, a: a }
    }
}
