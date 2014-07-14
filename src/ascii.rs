
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



struct Ascii {
    table: Table,
    chars: String,
    cout: Box<Terminal<WriterWrapper>>,
    cin: BufferedReader<StdReader>,
    width: uint,
    char_width: uint,
    char_height: uint,

}

impl Ascii {
    fn new() -> Ascii {
        Ascii {
            table: Table::generate(),
            chars: Ascii::load_chars(),
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
    fn calc_coverage(&mut self) {
        let chars_img = Image::load(&Path::new("chars.png")).unwrap();
        let chars_img = chars_img.to_linear(&self.table, Color::black());
        let chars_per_row = (self.width - 1) / 2;
        let (x_off, y_off) = (17, 47);
        for (i, c) in self.chars.as_slice().chars().enumerate() {
            let (y, x) = div_rem(i, chars_per_row);
            let (xb, yb) = (x * self.char_width * 2 + x_off, y * self.char_height + y_off);
            let r1 = range(xb - 1, xb + self.char_width + 1);
            let r2 = range(yb, yb + self.char_height);
            let coverage = r1.map(|x| r2.map(|y| {
                let pixel = chars_img.get(x, y);
                pixel.r + pixel.g + pixel.b
            }).sum()).sum();
            let _ = writeln!(self.cout, "{} -> {}", c, coverage);
        }
    }
    fn convert(&mut self, path: &Path) {
        let img = Image::load(path).unwrap();
        let img = img.to_linear(&self.table, Color::black());
        let _ = writeln!(self.cout, "[{}, {}]", img.width(), img.height());
        self.calc_coverage();
        self.cin.read_line().unwrap();
    }
    fn print_chars(&mut self) {
        let chars_per_row = (self.width - 1) / 2;
        self.cout.fg(15).unwrap();
        self.cout.bg(0).unwrap();
        for (i, c) in self.chars.as_slice().chars().enumerate() {
            if i % chars_per_row == 0 {
                let _ = writeln!(self.cout, "");
            }
            let _ = write!(self.cout, " {}", c);
        }
        let _ = writeln!(self.cout, "");
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
