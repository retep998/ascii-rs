
use libc::{c_uchar, c_uint, size_t, free, c_void};
use std::cell::Cell;
use std::io::fs::File;
use std::mem::uninitialized;
use std::slice::raw::buf_as_slice;

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

pub struct Image<T> {
    width: u32,
    height: u32,
    pixels: Vec<Cell<Color<T>>>,
}

impl<T> Image<T> {
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
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
            width: width, height: height, pixels: pixels,
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
                a: 1.
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
            r[i] = val * 0.2126;
            g[i] = val * 0.7152;
            b[i] = val * 0.0722;
            a[i] = val;
        }
        Table { r: r, g: g, b: b, a: a }
    }
}
