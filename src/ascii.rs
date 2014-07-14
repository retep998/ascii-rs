
extern crate libc;
extern crate term;

use image::Image;
use image::Table;
use image::Color;
use std::os::args;
use std::io::stdio::stdin;
use term::stdout;

mod image;

fn main() {
    let args = args();
    let table = Table::generate();
    let max_width = 100u;
    let char_width = 7u;
    let char_height = 14u;
    let chars_per_row = max_width / 2 - 1;
    let mut cout = stdout().unwrap();
    let mut cin = stdin();
    let chars = include_str!("chars.txt");
    if args.len() > 1 {
        let chars_img = Image::load(&Path::new("chars.png")).unwrap();
        let chars_img = chars_img.to_linear(&table, Color { r: 0, g: 0, b: 0, a: 0 });
        let img = Image::load(&Path::new(args.get(1).as_slice())).unwrap();
        let img = img.to_linear(&table, Color { r: 0, g: 0, b: 0, a: 0 });
        println!("[{}, {}]", img.width(), img.height());
    } else {
        cout.fg(15).unwrap();
        cout.bg(0).unwrap();
        for (i, c) in chars.chars().enumerate() {
            if i % chars_per_row == 0 {
                writeln!(cout, "");
            }
            write!(cout, " {}", c);
        }
    }
    cin.read_line().unwrap();
}
