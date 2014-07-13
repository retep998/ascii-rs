
use libc::{c_uchar, c_uint, size_t, free, c_void};
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

struct Srgb;
struct Linear;

#[deriving(Show)]
pub struct Color<T> {
    r: T,
    g: T,
    b: T,
    a: T,
}

pub struct Image<T> {
    width: u32,
    height: u32,
    pixels: Vec<Color<T>>,
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
        let pixels = unsafe { buf_as_slice(outbuf as *const Color<u8>, (width * height) as uint,
                                           |decoded| decoded.iter().map(|&c| c).collect()) };
        unsafe { free(outbuf as *mut c_void) };
        Ok(Image {
            width: width, height: height, pixels: pixels,
        })
    }
}
