// Copyright © 2014, Peter Atashian

#![feature(slicing_syntax, tuple_indexing)]

extern crate core;
extern crate image;
extern crate lodepng;
extern crate serialize;

use core::slice::{Found, NotFound};
use serialize::json::{decode};
use std::cell::{Cell};
use std::cmp::{max};
use std::collections::{PriorityQueue};
use std::io::{File};
use std::iter::{AdditiveIterator};
use std::num::{Zero};

#[deriving(Show, Clone)]
struct TermColor(Pixel, u8);
impl PartialEq for TermColor {
    fn eq(&self, o: &TermColor) -> bool {
        self.0 == o.0
    }
}

#[deriving(Show)]
struct Coverage(char, f32);
impl PartialEq for Coverage {
    fn eq(&self, o: &Coverage) -> bool {
        self.1 == o.1
    }
}

#[deriving(Show, PartialEq, PartialOrd, Clone)]
struct Pixel(f32, f32, f32);
impl Pixel {
    fn gray(&self) -> f32 {
        (self.0 + self.1 + self.2) / 3.
    }
    fn abs(&self) -> Pixel {
        Pixel(self.0.abs(), self.1.abs(), self.2.abs())
    }
    fn sqsum(&self) -> f32 {
        self.0 * self.0 + self.1 * self.1 + self.2 * self.2
    }
    fn magnitude(&self) -> f32 {
        self.sqsum().sqrt()
    }
    fn dot(&self, o: &Pixel) -> f32 {
        self.0 * o.0 + self.1 * o.1 + self.2 * o.2
    }
}
impl Add<Pixel, Pixel> for Pixel {
    fn add(&self, o: &Pixel) -> Pixel {
        Pixel(self.0 + o.0, self.1 + o.1, self.2 + o.2)
    }
}
impl Sub<Pixel, Pixel> for Pixel {
    fn sub(&self, o: &Pixel) -> Pixel {
        Pixel(self.0 - o.0, self.1 - o.1, self.2 - o.2)
    }
}
impl Zero for Pixel {
    fn zero() -> Pixel {
        Pixel(0., 0., 0.)
    }
    fn is_zero(&self) -> bool {
        *self == Zero::zero()
    }
}
impl Mul<Pixel, Pixel> for Pixel {
    fn mul(&self, o: &Pixel) -> Pixel {
        Pixel(self.0 * o.0, self.1 * o.1, self.2 * o.2)
    }
}
impl Div<Pixel, Pixel> for Pixel {
    fn div(&self, o: &Pixel) -> Pixel {
        Pixel(self.0 / o.0, self.1 / o.1, self.2 / o.2)
    }
}

#[deriving(Show)]
struct Image {
    pixels: Vec<Cell<Pixel>>,
    width: u32,
    height: u32,
}
impl Image {
    fn get(&self, x: u32, y: u32) -> Pixel {
        self.pixels[(y * self.width + x) as uint].get()
    }
    fn resize(&self, width: u32, height: u32) -> Image {
        let (rw, rh) = (self.width as f32 / (width as f32), self.height as f32 / (height as f32));
        let mut pixels = Vec::with_capacity((width * height) as uint);
        for y in range(0, height) {
            for x in range(0, width) {
                let (x1, x2) = ((x as f32 * rw) as u32, ((x + 1) as f32 * rw) as u32);
                let (y1, y2) = ((y as f32 * rh) as u32, ((y + 1) as f32 * rh) as u32);
                let (x2, y2) = (max(x2, x1 + 1), max(y2, y1 + 1));
                let c = range(x1, x2).map(|x| {
                    range(y1, y2).map(|y| {
                        self.get(x, y)
                    }).sum()
                }).sum();
                let m = 1. / (((x2 - x1) * (y2 - y1)) as f32);
                pixels.push(Cell::new(c * Pixel(m, m, m)));
            }
        }
        assert!(pixels.len() as u32 == width * height);
        Image {
            pixels: pixels,
            width: width,
            height: height,
        }
    }
}

#[deriving(Decodable, Show)]
struct Config {
    winwidth: u32,
    winheight: u32,
    charwidth: u32,
    charheight: u32,
    offsetx: u32,
    offsety: u32,
    chars: String,
    fullcolor: bool,
}
impl Config {
    fn load() -> Config {
        let mut file = File::open(&Path::new("config.json")).unwrap();
        let data = file.read_to_string().unwrap();
        decode(data.as_slice()).unwrap()
    }
}

#[deriving(Show)]
struct Ascii {
    config: Config,
    table: Vec<f32>,
    colors: Vec<TermColor>,
    chars: Vec<Coverage>,
}
impl Ascii {
    fn new() -> Ascii {
        let config = Config::load();
        Ascii {
            config: config,
            table: Vec::new(),
            colors: Vec::new(),
            chars: Vec::new(),
        }
    }
    #[allow(dead_code)]
    fn print_chars(&self) {
        for x in range(0u16, 256) {
            bg(x as u8);
            print!(" ");
        }
        println!("");
        fg(15);
        bg(0);
        println!("{}", self.config.chars);
        let lines = 256 / self.config.winwidth;
        let lines = lines + self.config.chars[].chars().count() as u32 / self.config.winwidth;
        for _ in range(lines + 5, self.config.winheight) {
            println!("");
        }
    }
    fn build_table(&mut self) {
        fn decode(x: f32) -> f32 {
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / (1. + 0.055)).powf(2.4)
            }
        }
        self.table = range(0u32, 256).map(|x| decode(x as f32 * (1. / 255.))).collect();
    }
    fn load(&self, p: &Path) -> Image {
        use image::GenericImage;
        assert!(self.table.len() == 256);
        let img = match lodepng::load(p) {
            Ok(x) => x,
            Err(_) => image::open(p).unwrap().to_rgba(),
        };
        let (w, h) = img.dimensions();
        let pixels = img.into_vec();
        let pixels = pixels.iter().map(|p| {
            let (&r, &g, &b, &a) = unsafe {(
                self.table[].unsafe_get(p.0 as uint),
                self.table[].unsafe_get(p.1 as uint),
                self.table[].unsafe_get(p.2 as uint),
                self.table[].unsafe_get(p.3 as uint),
            )};
            Cell::new(Pixel(r * a, g * a, b * a))
        }).collect();
        Image {
            pixels: pixels,
            width: w,
            height: h,
        }
    }
    fn calc_colors(&mut self) {
        let img = self.load(&Path::new("colors.png"));
        self.colors = range(0u32, if self.config.fullcolor { 256 } else { 16 }).map(|i| {
            let x = i % self.config.winwidth * self.config.charwidth + self.config.offsetx;
            let y = i / self.config.winwidth * self.config.charheight + self.config.offsety;
            TermColor(img.get(x, y), i as u8)
        }).collect();
        let offset = (256 / self.config.winwidth + 1) * self.config.charheight
            + self.config.offsety;
        let (lo, hi) = (self.colors[0].0.gray(), self.colors[15].0.gray());
        self.chars = self.config.chars[].chars().enumerate().map(|(i, c)| {
            let x = i as u32 % self.config.winwidth * self.config.charwidth + self.config.offsetx;
            let y = i as u32 / self.config.winwidth * self.config.charheight + offset;
            let p = range(x, x + self.config.charwidth).map(|x| {
                range(y, y + self.config.charheight).map(|y| {
                    img.get(x, y).gray()
                }).sum()
            }).sum() / (self.config.charwidth * self.config.charheight) as f32;
            Coverage(c, (p - lo) / (hi - lo))
        }).collect();
        self.colors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        self.colors.dedup();
        self.chars.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        self.chars.dedup();
        self.chars.retain(|x| x.1 <= 0.5);
    }
    fn prepare(&mut self) {
        self.build_table();
        self.calc_colors();
    }
    fn fit_window(&self, img: &Image) -> Image {
        let (ww, wh) = (self.config.winwidth - 1, self.config.winheight - 3);
        let (cw, ch) = (self.config.charwidth, self.config.charheight);
        let fw = (cw * ww) as f32;
        let fh = (ch * wh) as f32;
        let ratio = fh / fw;
        let imgratio = img.height as f32 / img.width as f32;
        if ratio > imgratio {
            let h = (fw * imgratio / ch as f32) as u32;
            img.resize(ww, h)
        } else {
            let w = (fh / imgratio / cw as f32) as u32;
            img.resize(w, wh)
        }
    }
    fn get_closest(&self, p: &Pixel) -> Vec<TermColor> {
        struct Entry(Pixel, u8, f32);
        impl Ord for Entry {
            fn cmp(&self, o: &Entry) -> Ordering {
                self.partial_cmp(o).unwrap()
            }
        }
        impl PartialOrd for Entry {
            fn partial_cmp(&self, o: &Entry) -> Option<Ordering> {
                self.2.partial_cmp(&o.2).map(|x| x.reverse())
            }
        }
        impl Eq for Entry {}
        impl PartialEq for Entry {
            fn eq(&self, o: &Entry) -> bool {
                self.2 == o.2
            }
        }
        if self.config.fullcolor {
            let mut colors: PriorityQueue<_> = self.colors.iter().map(|c| {
                let d = (c.0 - *p).abs().gray();
                Entry(c.0, c.1, d)
            }).collect();
            range(0, 20u).map(|_| {
                let e = colors.pop().unwrap();
                TermColor(e.0, e.1)
            }).collect()
        } else {
            self.colors.clone()
        }
    }
    fn closest(&self, p: &Pixel) -> (u8, u8, char, Pixel) {
        let colors = self.get_closest(p);
        let mut bestdiff = 100f32;
        let mut best = (0, colors[0].1, ' ', p - colors[0].0);
        for c1 in colors.iter() {
            for c2 in colors.iter() {
                let (v1, v2) = (p - c1.0, c2.0 - c1.0);
                let cover = if v2.abs().gray() < 0.0001 {
                    0.
                } else {
                    v1.dot(&v2) / v2.sqsum()
                };
                if cover < 0. || cover > 0.5 { continue }
                let Coverage(ch, n) = match self.chars[].binary_search(|x| {
                    x.1.partial_cmp(&cover).unwrap()
                }) {
                    Found(x) => self.chars[x],
                    NotFound(x) => self.chars[x - 1],
                };
                let color = c1.0 * Pixel(1. - n, 1. - n, 1. - n) + c2.0 * Pixel(n, n, n);
                let diff = *p - color;
                let sdiff = c2.0 - c1.0;
                let d = diff.abs().magnitude() + sdiff.abs().magnitude() * 0.2;
                if d < bestdiff {
                    bestdiff = d;
                    best = (c2.1, c1.1, ch, diff);
                }
            }
        }
        best
    }
    fn convert(&self, p: &Path) {
        let img = self.load(p);
        let mut img = self.fit_window(&img);
        let width = img.width;
        img.pixels.reserve_additional(width as uint * 2);
        let pixels = img.pixels[];
        let mut fg = 15;
        let mut bg = 0;
        let set = |f: u8, b: u8| {
            if f == fg && b == bg { return }
            if f == fg {
                print!("\x1b[48;5;{}m", b);
            } else if b == bg {
                print!("\x1b[38;5;{}m", f);
            } else {
                print!("\x1b[38;5;{};48;5;{}m", f, b);
            }
            fg = f;
            bg = b;
        };
        for (y, chunk) in pixels.chunks(img.width as uint).enumerate() {
            for (x, p) in chunk.iter().enumerate() {
                let (f, b, c, diff) = self.closest(&p.get());
                let adjust = |ox, oy, mult| {
                    let cell = unsafe { pixels.unsafe_get((y + oy) * (width as uint) + x + ox) };
                    cell.set(cell.get() + diff * Pixel(mult, mult, mult));
                };
                adjust( 1, 0, 5. / 32.);
                adjust( 2, 0, 3. / 32.);
                adjust( 2, 1, 2. / 32.);
                adjust(-1, 1, 4. / 32.);
                adjust(-0, 1, 5. / 32.);
                adjust( 1, 1, 4. / 32.);
                adjust( 2, 1, 2. / 32.);
                adjust(-1, 2, 2. / 32.);
                adjust( 0, 2, 3. / 32.);
                adjust( 1, 2, 2. / 32.);
                set(f, b);
                print!("{}", c);
            }
            set(15, 0);
            println!("");
        }
    }
}

fn fg(x: u8) {
    print!("\x1b[38;5;{}m", x);
}
fn bg(x: u8) {
    print!("\x1b[48;5;{}m", x);
}


fn main() {
    let mut ascii = Ascii::new();
    let args = std::os::args();
    if args.len() <= 1 {
        ascii.print_chars();
    } else {
        ascii.prepare();
        ascii.convert(&Path::new(args[1][]));
    }
}
