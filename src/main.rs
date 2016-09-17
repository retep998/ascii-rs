// Copyright © 2016, Peter Atashian

extern crate image;
extern crate wio;

use image::{open};
use pixel::{Pixel};
use std::env::{args};
use wio::console::{CharInfo, Input, InputBuffer, ScreenBuffer};

mod pixel;

const COLORS: &'static [(u8, u8, u8); 16] = &[
    (0x00, 0x00, 0x00), (0x00, 0x00, 0x80), (0x00, 0x80, 0x00), (0x00, 0x80, 0x80),
    (0x80, 0x00, 0x00), (0x80, 0x00, 0x80), (0x80, 0x80, 0x00), (0xC0, 0xC0, 0xC0),
    (0x80, 0x80, 0x80), (0x00, 0x00, 0xFF), (0x00, 0xFF, 0x00), (0x00, 0xFF, 0xFF),
    (0xFF, 0x00, 0x00), (0xFF, 0x00, 0xFF), (0xFF, 0xFF, 0x00), (0xFF, 0xFF, 0xFF),
];
const COLOR_TABLE: &'static [u32; 16] = &[
    0x000000, 0x800000, 0x008000, 0x808000,
    0x000080, 0x800080, 0x008080, 0xC0C0C0,
    0x808080, 0xFF0000, 0x00FF00, 0xFFFF00,
    0x0000FF, 0xFF00FF, 0x00FFFF, 0xFFFFFF,
];

const CHARS: &'static [u16; 256] = &[
    0x0020, 0x263a, 0x263b, 0x2665, 0x2666, 0x2663, 0x2660, 0x2022,
    0x25d8, 0x25cb, 0x25d9, 0x2642, 0x2640, 0x266a, 0x266b, 0x263c,
    0x25ba, 0x25c4, 0x2195, 0x203c, 0x00b6, 0x00a7, 0x25ac, 0x21a8,
    0x2191, 0x2193, 0x2192, 0x2190, 0x221f, 0x2194, 0x25b2, 0x25bc,
    0x0020, 0x0021, 0x0022, 0x0023, 0x0024, 0x0025, 0x0026, 0x0027,
    0x0028, 0x0029, 0x002a, 0x002b, 0x002c, 0x002d, 0x002e, 0x002f,
    0x0030, 0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037,
    0x0038, 0x0039, 0x003a, 0x003b, 0x003c, 0x003d, 0x003e, 0x003f,
    0x0040, 0x0041, 0x0042, 0x0043, 0x0044, 0x0045, 0x0046, 0x0047,
    0x0048, 0x0049, 0x004a, 0x004b, 0x004c, 0x004d, 0x004e, 0x004f,
    0x0050, 0x0051, 0x0052, 0x0053, 0x0054, 0x0055, 0x0056, 0x0057,
    0x0058, 0x0059, 0x005a, 0x005b, 0x005c, 0x005d, 0x005e, 0x005f,
    0x0060, 0x0061, 0x0062, 0x0063, 0x0064, 0x0065, 0x0066, 0x0067,
    0x0068, 0x0069, 0x006a, 0x006b, 0x006c, 0x006d, 0x006e, 0x006f,
    0x0070, 0x0071, 0x0072, 0x0073, 0x0074, 0x0075, 0x0076, 0x0077,
    0x0078, 0x0079, 0x007a, 0x007b, 0x007c, 0x007d, 0x007e, 0x2302,
    0x00c7, 0x00fc, 0x00e9, 0x00e2, 0x00e4, 0x00e0, 0x00e5, 0x00e7,
    0x00ea, 0x00eb, 0x00e8, 0x00ef, 0x00ee, 0x00ec, 0x00c4, 0x00c5,
    0x00c9, 0x00e6, 0x00c6, 0x00f4, 0x00f6, 0x00f2, 0x00fb, 0x00f9,
    0x00ff, 0x00d6, 0x00dc, 0x00a2, 0x00a3, 0x00a5, 0x20a7, 0x0192,
    0x00e1, 0x00ed, 0x00f3, 0x00fa, 0x00f1, 0x00d1, 0x00aa, 0x00ba,
    0x00bf, 0x2310, 0x00ac, 0x00bd, 0x00bc, 0x00a1, 0x00ab, 0x00bb,
    0x2591, 0x2592, 0x2593, 0x2502, 0x2524, 0x2561, 0x2562, 0x2556,
    0x2555, 0x2563, 0x2551, 0x2557, 0x255d, 0x255c, 0x255b, 0x2510,
    0x2514, 0x2534, 0x252c, 0x251c, 0x2500, 0x253c, 0x255e, 0x255f,
    0x255a, 0x2554, 0x2569, 0x2566, 0x2560, 0x2550, 0x256c, 0x2567,
    0x2568, 0x2564, 0x2565, 0x2559, 0x2558, 0x2552, 0x2553, 0x256b,
    0x256a, 0x2518, 0x250c, 0x2588, 0x2584, 0x258c, 0x2590, 0x2580,
    0x03b1, 0x00df, 0x0393, 0x03c0, 0x03a3, 0x03c3, 0x00b5, 0x03c4,
    0x03a6, 0x0398, 0x03a9, 0x03b4, 0x221e, 0x03c6, 0x03b5, 0x2229,
    0x2261, 0x00b1, 0x2265, 0x2264, 0x2320, 0x2321, 0x00f7, 0x2248,
    0x00b0, 0x2219, 0x00b7, 0x221a, 0x207f, 0x00b2, 0x25a0, 0x00a0,
];

struct Image {
    pixels: Vec<Pixel>,
    width: u32,
    height: u32,
}
impl Image {
    fn from_srgb(img: &[(u8, u8, u8)], width: u32, height: u32) -> Image {
        let pixels = img.iter().map(|p| {
            Pixel::from_srgb(p.0, p.1, p.2)
        }).collect();
        Image {
            pixels: pixels,
            width: width,
            height: height,
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
}
fn load(s: &str) -> (u32, u32, Vec<(u8, u8, u8)>) {
    let img = open(s).unwrap();
    let img = img.to_rgba();
    let data = img.pixels().map(|pixel| {
        let (r, g, b, a) = (pixel.data[0] as u16, pixel.data[1] as u16, pixel.data[2] as u16, pixel.data[3] as u16);
        ((r * a / 255) as u8, (g * a / 255) as u8, (b * a / 255) as u8)
    }).collect();
    (img.width(), img.height(), data)
}
// Someone please make this better
fn optimal_palette(data: &[(u8, u8, u8)]) -> Vec<(u8, u8, u8)> {
    let mut sets = vec![data.to_owned()];
    for _ in 0..4 {
        let mut next = vec![];
        for set in &sets {
            let (d1, d2) = mean_cut(set);
            next.push(d1);
            next.push(d2);
        }
        sets = next;
    }
    sets.iter().map(|set| average(set)).collect()
}
fn average(data: &[(u8, u8, u8)]) -> (u8, u8, u8) {
    let total = data.len() as u64;
    let (r, g, b) = data.iter().fold((0u64, 0u64, 0u64), |(r, g, b), p| {
        (r + p.0 as u64, g + p.1 as u64, b + p.2 as u64)
    });
    ((r / total) as u8, (g / total) as u8, (b / total) as u8)
}
fn mean_cut(data: &[(u8, u8, u8)]) -> (Vec<(u8, u8, u8)>, Vec<(u8, u8, u8)>) {
    let (ar, ag, ab) = average(data);
    let (rd, gd, bd) = data.iter().fold((0u64, 0u64, 0u64), |(r, g, b), p| {
        let (dr, dg, db) = ((ar - p.0) as u64, (ag - p.1) as u64, (ab - p.2) as u64);
        (r + dr * dr, g + dg * dg, b + db * db)
    });
    if rd > gd && rd > bd {
        (data.iter().cloned().filter(|&(r, _, _)| r < ar).collect(),
         data.iter().cloned().filter(|&(r, _, _)| r >= ar).collect())
    } else if gd > rd && gd > bd {
        (data.iter().cloned().filter(|&(_, g, _)| g < ag).collect(),
         data.iter().cloned().filter(|&(_, g, _)| g >= ag).collect())
    } else {
        (data.iter().cloned().filter(|&(_, _, b)| b < ab).collect(),
         data.iter().cloned().filter(|&(_, _, b)| b >= ab).collect())
    }
}
fn color_table(d: &[(u8, u8, u8)]) -> [u32; 16] {
    fn c((r, g, b): (u8, u8, u8)) -> u32 {
        (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
    }
    [
        c(d[0x0]), c(d[0x1]), c(d[0x2]), c(d[0x3]),
        c(d[0x4]), c(d[0x5]), c(d[0x6]), c(d[0x7]),
        c(d[0x8]), c(d[0x9]), c(d[0xa]), c(d[0xb]),
        c(d[0xc]), c(d[0xd]), c(d[0xe]), c(d[0xf]),
    ]
}
fn make_text(img: Image, chars: &[(u16, f32)], colors: &[(u8, u8, u8)]) -> Vec<CharInfo> {
    let (w, h) = (img.width, img.height);
    let mut pixels = img.pixels;
    pixels.resize((w * h + w + 1) as usize, Pixel::black());
    let colors: Vec<Pixel> = colors.iter().map(|&(r, g, b)| {
        Pixel::from_srgb(r, g, b)
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
                    for &(ch, m) in chars {
                        let combined = fg * m + bg * (1. - m);
                        let d1 = pixel.diff_sq(fg);
                        let d2 = pixel.diff_sq(bg);
                        let dd = pixel.diff_sq(combined);
                        let d = (d1 + d2) * 0.1 + dd;
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
fn calculate_chars(w: u32, h: u32) -> Vec<(u16, f32)> {
    let name = format!("{}x{}.png", w, h);
    let img = open(&name).unwrap().to_rgba();
    let mult = 1. / ((w * h) as f32);
    let mut res: Vec<_> = CHARS.iter().enumerate().map(|(i, &ch)| {
        let i = i as u32;
        let mut sum = 0;
        let (bx, by) = (i % 16 * w, i / 16 * h);
        for y in by..(by + h) {
            for x in bx..(bx + w) {
                let pix = img.get_pixel(x, y);
                if pix.data[0] != 0 { sum += 1; }
            }
        }
        (ch, sum)
    }).filter(|&(_, sum)| sum * 2 <= w * h).collect();
    res.sort_by(|a, b| a.1.cmp(&b.1));
    let mut last = 273;
    res.retain(|&(_, sum)| if sum == last { false } else { last = sum; true });
    res.into_iter().map(|(ch, sum)| (ch, (sum as f32) * mult)).collect()
}
fn main() {
    // Load image from file
    let args: Vec<_> = args().collect();
    let (width, height, srgb) = load(&args[1]);
    let img = Image::from_srgb(&srgb, width, height);
    // Back up console colors
    let orig = ScreenBuffer::from_conout().unwrap();
    let orig_info = orig.info_ex().unwrap();
    // Create a new console buffer
    let cout = ScreenBuffer::new().unwrap();
    // Calculate some dimensions
    let (fw, fh) = cout.font_size().unwrap();
    let (fw, fh) = (fw as u32, fh as u32);
    let (w, h) = (img.width / fw + 1, img.height / fh + 1);
    // Figure out characters
    let chars = calculate_chars(fw, fh);
    //let colors = optimal_palette(&srgb);
    let colors = COLORS.to_owned();
    // Setup the console buffer info
    let mut info = cout.info_ex().unwrap();
    info.0.ColorTable = color_table(&colors);
    info.0.dwSize.X = w as i16;
    info.0.dwSize.Y = h as i16;
    cout.set_info_ex(info).unwrap();
    cout.set_active().unwrap();
    // Resize image
    let img = img.increase_size(w * fw, h * fh);
    let img = img.shrink_factor(fw, fh);
    // Display image
    let text = make_text(img, &chars, &colors);
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