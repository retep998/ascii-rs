
extern crate libc;
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
