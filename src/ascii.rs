
extern crate libc;

use image::Image;

mod image;


fn main() {
    let img = Image::load(&Path::new("RainbowDashFluff.png"));
}
