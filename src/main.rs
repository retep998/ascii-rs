// Copyright © 2014, Peter Atashian

#![feature(slicing_syntax, tuple_indexing)]

extern crate image;
extern crate lodepng;
extern crate serialize;

use serialize::json::{decode};
use std::cell::Cell;
use std::io::{File};
use std::iter::AdditiveIterator;
use std::num::Zero;

#[deriving(Show)]
struct Coverage(char, f32);
impl PartialEq for Coverage {
    fn eq(&self, o: &Coverage) -> bool {
        self.1 == o.1
    }
}

#[deriving(Show, PartialEq)]
struct Pixel(f32, f32, f32);
impl Pixel {
    fn gray(&self) -> f32 {
        (self.0 + self.1 + self.2) / 3.
    }
}
impl Add<Pixel, Pixel> for Pixel {
    fn add(&self, o: &Pixel) -> Pixel {
        Pixel(self.0 + o.0, self.1 + o.1, self.2 + o.2)
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
}

#[deriving(Decodable, Show)]
struct Config {
    winwidth: u32,
    winheight: u32,
    charwidth: u32,
    charheight: u32,
    offsetx: u32,
    offsety: u32,
    chars: String
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
    colors: Vec<Pixel>,
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
        self.colors = range(0u32, 256).map(|i| {
            let x = i % self.config.winwidth * self.config.charwidth + self.config.offsetx;
            let y = i / self.config.winwidth * self.config.charheight + self.config.offsety;
            img.get(x, y)
        }).collect();
        let offset = (256 / self.config.winwidth + 1) * self.config.charheight
            + self.config.offsety;
        let (lo, hi) = (self.colors[0].gray(), self.colors[15].gray());
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
        self.chars.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        self.chars.dedup();
    }
    fn prepare(&mut self) {
        self.build_table();
        self.calc_colors();
    }
    fn convert(&self, p: &Path) {
        let img = self.load(p);
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
    // ascii.print_chars();
    ascii.prepare();
    ascii.convert(&Path::new("RainbowDashFluff.png"));
}

/*

extern crate term;

use image::{Color, Image, Table};
use std::io::{BufferedReader};
use std::io::stdio::{stdin};
use std::io::stdio::{StdReader};
use std::iter::{AdditiveIterator};
use std::num::{div_rem};
use std::os::{args};
use term::{stdout};
use term::{Terminal, WriterWrapper};

mod image;

struct Combo {
    fg: u16,
    bg: u16,
    color: Color<f32>,
    ch: char,
    lum: f32,
}

struct Ascii {
    table: Table,
    chars: String,
    colors: [Color<f32>, ..16],
    cout: Box<Terminal<WriterWrapper>>,
    cin: BufferedReader<StdReader>,
    width: uint,
    char_width: uint,
    char_height: uint,

}

impl Ascii {
    fn new() -> Ascii {
        let table = Table::generate();
        Ascii {
            table: table,
            chars: Ascii::load_chars(),
            colors: Ascii::terminal_colors(&table),
            cout: stdout().unwrap(),
            cin: stdin(),
            width: 100,
            char_width: 7,
            char_height: 14,
        }
    }
    fn load_chars() -> String {
        let s = include_str!("chars.txt");
        s.to_string()
    }
    fn terminal_colors(table: &Table) -> [Color<f32>, ..16] {
        [
            Color::from_rgb(0x00, 0x00, 0x00, table),
            Color::from_rgb(0x80, 0x00, 0x00, table),
            Color::from_rgb(0x00, 0x80, 0x00, table),
            Color::from_rgb(0x80, 0x80, 0x00, table),
            Color::from_rgb(0x00, 0x00, 0x80, table),
            Color::from_rgb(0x80, 0x00, 0x80, table),
            Color::from_rgb(0x00, 0x80, 0x80, table),
            Color::from_rgb(0xc0, 0xc0, 0xc0, table),
            Color::from_rgb(0x80, 0x80, 0x80, table),
            Color::from_rgb(0xff, 0x00, 0x00, table),
            Color::from_rgb(0x00, 0xff, 0x00, table),
            Color::from_rgb(0xff, 0xff, 0x00, table),
            Color::from_rgb(0x00, 0x00, 0xff, table),
            Color::from_rgb(0xff, 0x00, 0xff, table),
            Color::from_rgb(0x00, 0xff, 0xff, table),
            Color::from_rgb(0xff, 0xff, 0xff, table),
        ]
    }
    fn calc_coverage(&mut self) -> Vec<(char, f32)> {
        let chars_img = Image::load(&Path::new("chars.png")).unwrap();
        let chars_img = chars_img.to_linear(&self.table, Color::black());
        let chars_per_row = (self.width - 1) / 2;
        let (x_off, y_off) = (10, 33);
        self.chars.as_slice().chars().enumerate().map(|(i, c)| {
            let (y, x) = div_rem(i, chars_per_row);
            let xb = x * self.char_width * 2 + self.char_width + x_off;
            let yb = y * self.char_height + y_off;
            let r1 = range(xb - 1, xb + self.char_width + 1);
            let r2 = range(yb, yb + self.char_height);
            let coverage = r1.map(|x| r2.map(|y| {
                let pixel = chars_img.get(x, y);
                pixel.r + pixel.g + pixel.b
            }).sum()).sum() / (self.char_width * self.char_height) as f32;
            (c, coverage)
        }).collect()
    }
    fn gen_combos(&mut self) -> Vec<Combo> {
        let covers = self.calc_coverage();
        let mut combos = Vec::new();
        for &(ch, cover) in covers.iter() {
            for (fgi, fg) in self.colors.iter().enumerate() {
                for (bgi, bg) in self.colors.iter().enumerate() {
                    let color = fg * cover + bg * (1. - cover);
                    combos.push(Combo {
                        fg: fgi as u16,
                        bg: bgi as u16,
                        color: color,
                        ch: ch,
                        lum: fg.diff(bg).luminance(),
                    });
                }
            }
        }
        combos
    }
    fn convert(&mut self, path: &Path) {
        let img = Image::load(path).unwrap();
        let img = img.to_linear(&self.table, Color::black());
        (writeln!(self.cout, "[{}, {}]", img.width(), img.height())).unwrap();
        let combos = self.gen_combos();
        self.cin.read_line().unwrap();
    }
    fn print_chars(&mut self) {
        let chars_per_row = (self.width - 1) / 2;
        self.cout.fg(15).unwrap();
        self.cout.bg(0).unwrap();
        for (i, c) in self.chars.as_slice().chars().enumerate() {
            if i != 0 && i % chars_per_row == 0 {
                (writeln!(self.cout, "")).unwrap();
            }
            (write!(self.cout, " {}", c)).unwrap();
        }
        (writeln!(self.cout, "")).unwrap();
        self.cin.read_line().unwrap();
    }
}

fn main() {
    let args = args();
    let mut ascii = Ascii::new();
    if args.len() > 1 {
        ascii.convert(&Path::new(args.get(1).as_slice()));
    } else {
        ascii.print_chars();
    }
}
*/
