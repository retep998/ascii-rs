// Copyright © 2016, Peter Atashian

extern crate image;
extern crate wio;

use image::{open};
use pixel::{Pixel};
use std::env::{args};
use wio::console::{CharInfo, Input, InputBuffer, ScreenBuffer};

mod pixel;

const COLOR_TABLE: &'static [u32; 16] = &[
    0x000000, 0x800000, 0x008000, 0x808000,
    0x000080, 0x800080, 0x008080, 0xC0C0C0,
    0x808080, 0xFF0000, 0x00FF00, 0xFFFF00,
    0x0000FF, 0xFF00FF, 0x00FFFF, 0xFFFFFF,
];

struct Image {
    pixels: Vec<Pixel>,
    width: u32,
    height: u32,
}
impl Image {
    fn load(s: &str) -> Image {
        let img = open(s).unwrap();
        let img = img.to_rgba();
        let pixels = img.pixels().map(|pixel| {
            Pixel::from_srgb(pixel.data[0], pixel.data[1], pixel.data[2], pixel.data[3])
        }).collect();
        Image {
            pixels: pixels,
            width: img.width(),
            height: img.height(),
        }
    }
    fn increase_size(&self, nw: u32, nh: u32) -> Image {
        let mut buf = vec![Pixel::black(); (nw * nh) as usize];
        for y in 0..self.height {
            let orig = &self.pixels[(y * self.width) as usize..(y * self.width + self.width) as usize];
            let line = &mut buf[(y * nw) as usize..(y * nw + self.width) as usize];
            line.copy_from_slice(orig);
        }
        Image {
            pixels: buf,
            width: nw,
            height: nh,
        }
    }
    fn shrink_factor(&self, fw: u32, fh: u32) -> Image {
        let mult = 1. / ((fw * fh) as f32);
        let (nw, nh) = (self.width / fw, self.height / fh);
        let mut buf = vec![Pixel::black(); (nw * nh) as usize];
        for y in 0..nh {
            for x in 0..nw {
                let (bx, by) = (x * fw, y * fh);
                let mut p = Pixel::black();
                for yy in by..(by + fh) {
                    for xx in bx..(bx + fw) {
                        p = p + self.pixels[(yy * self.width + xx) as usize];
                    }
                }
                buf[(y * nw + x) as usize] = p * mult;
            }
        }
        Image {
            pixels: buf,
            width: nw,
            height: nh,
        }
    }
    fn make_text(&self) -> Vec<CharInfo> {
        let chars = [
            (' ' as u16, 0.00),
            ('░' as u16, 0.25),
            ('▒' as u16, 0.50),
            ('▓' as u16, 0.75),
        ];
        let (w, h) = (self.width, self.height);
        let mut pixels = self.pixels.clone();
        pixels.resize(self.pixels.len() + (w as usize) + 1, Pixel::black());
        let colors: Vec<Pixel> = COLOR_TABLE.iter().map(|&c| {
            Pixel::from_srgb((c & 0xff) as u8, ((c >> 8) & 0xff) as u8, ((c >> 16) & 0xff) as u8, 0xff)
        }).collect();
        let mut buf = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                let index = y * w + x;
                let pixel = pixels[index as usize];
                let mut best_fg = 0;
                let mut best_bg = 0;
                let mut best_char = 20;
                let mut best_color = Pixel::black();
                let mut best_diff = 100.;
                for c1 in 0..16 {
                    for c2 in 0..16 {
                        let fg = colors[c1];
                        let bg = colors[c2];
                        for &(ch, m) in &chars {
                            let combined = fg * m + bg * (1. - m);
                            let d = combined.diff_sq(pixel);
                            if d < best_diff {
                                best_fg = c1;
                                best_bg = c2;
                                best_char = ch;
                                best_color = combined;
                                best_diff = d;
                            }
                        }
                    }
                }
                buf.push(CharInfo::new(best_char, ((best_bg << 4) | best_fg) as u16));
                let err = pixel - best_color;
                pixels[(index + 1) as usize] += err * 0.4375;
                pixels[(index + w - 1) as usize] += err * 0.1875;
                pixels[(index + w) as usize] += err * 0.3125;
                pixels[(index + w + 1) as usize] += err * 0.0625;
            }
        }
        buf
    }
}

fn main() {
    // Load image from file
    let args: Vec<_> = args().collect();
    let img = Image::load(&args[1]);
    // Back up console colors
    let orig = ScreenBuffer::from_conout().unwrap();
    let orig_info = orig.info_ex().unwrap();
    // Create a new console buffer
    let cout = ScreenBuffer::new().unwrap();
    // Calculate some dimensions
    let (fw, fh) = cout.font_size().unwrap();
    let (fw, fh) = (fw as u32, fh as u32);
    let (w, h) = (img.width / fw + 1, img.height / fh + 1);
    // Setup the console buffer info
    let mut info = cout.info_ex().unwrap();
    info.0.ColorTable = *COLOR_TABLE;
    info.0.dwSize.X = w as i16;
    info.0.dwSize.Y = h as i16;
    cout.set_info_ex(info).unwrap();
    cout.set_active().unwrap();
    // Resize image
    let img = img.increase_size(w * fw, h * fh);
    let img = img.shrink_factor(fw, fh);
    // Display image
    let text = img.make_text();
    cout.write_output(&text, (w as i16, h as i16), (0, 0)).unwrap();
    // Wait for keyboard input
    let cin = InputBuffer::from_conin().unwrap();
    cin.flush_input().unwrap();
    'done: loop {
        for input in cin.read_input().unwrap() {
            if let Input::Key{key_code: 0x0D, ..} = input { break 'done }
        }
    }
    // Restore console colors
    orig.set_info_ex(orig_info).unwrap();
    orig.set_active().unwrap();
}