// Copyright © 2014, Peter Atashian

#![allow(dead_code)]
#![feature(slice_patterns, test, vec_resize)]

extern crate image;
extern crate kernel32;
extern crate nalgebra as na;
extern crate test;
extern crate winapi;

use colors::{Pixel, RGB, Lab};
use na::{Vec3};
use std::io::{stdin};
use std::path::{Path};
use test::{Bencher};
use test::black_box as bb;
use wincon::{Attr, CharInfo, ConInfoEx, Console, Rect, Std, Vec2};

mod colors;
mod wincon;

#[derive(Debug)]
struct Ascii {
    console: Console,
    csize: (u32, u32),
    cinfo: ConInfoEx,
}
impl Ascii {
    fn new() -> Ascii {
        let console = Console::get(Std::Output).unwrap();
        let finfo = console.get_font_info().unwrap();
        let csize = (finfo.width() as u32, finfo.height() as u32);
        let cinfo = console.get_info_ex().unwrap();
        Ascii {
            console: console,
            csize: csize,
            cinfo: cinfo,
        }
    }
    fn draw(&self) {
        let buf = (0..16).map(|i| {
            CharInfo::new('X', Attr::new_color(i, i))
        }).collect::<Vec<_>>();
        let info = self.console.get_info_ex().unwrap();
        let pos = info.cursor();
        let win = Rect::new(
            pos.x(), pos.y(), pos.x() + 16, pos.y() + 1,
        );
        self.console.write_output(
            &buf, Vec2::new(16, 1), Vec2::new(0, 0), win,
        ).unwrap();
    }
    
    fn convert(&self, name: &str) {
        fn cbrt(t: f32) -> f32 {
            use std::mem::transmute;
            let ix: u32 = unsafe { transmute(t) };
            let ix = ix / 4 + ix / 16;
            let ix = ix + ix / 16;
            let ix = ix + ix / 256;
            let ix = 0x2a5137a0 + ix;
            let x: f32 = unsafe { transmute(ix) };
            let x = 0.33333333 * (2. * x + t / (x * x));
            0.33333333 * (2. * x + t / (x * x))
        }
        self.display("Loading image");
        let img = image::open(&Path::new(name)).unwrap();
        self.display("Converting to RGBA");
        let img = img.to_rgba();
        let mut count = 0;
        for r in 1..256 {
            for g in 1..256 {
                for b in 1..256 {
                    let rgb = Pixel::decode(r as u8, g as u8, b as u8);
                    let a = rgb.rgb_to_xyz();
                    let b = a.xyz_to_lab().lab_to_xyz();
                    let diff = (b - a).magnitude().log2();
                    if diff > -18. { count += 1 }
                }
            }
            self.display(&format!("{}%", r * 100 / 255));
        }
        self.display(&format!("Failures: {}/{}", count, 255 * 255 * 255));
        println!("");
        self.draw();
    }
    fn display(&self, s: &str) {
        let mut buf = s.chars().map(|c| (c as u32) as u16).collect::<Vec<_>>();
        let info = self.console.get_info_ex().unwrap();
        let win = info.window();
        let width = win.right() - win.left();
        buf.resize(width as usize, 32);
        let pos = info.cursor();
        self.console.write_output_chars(&buf, pos).unwrap();
    }
}
impl Drop for Ascii {
    fn drop(&mut self) {
        let mut s = String::new();
        stdin().read_line(&mut s).unwrap();
        self.console.set_info_ex(&self.cinfo).unwrap();
    }
}
#[bench] fn bench_to_lab(b: &mut Bencher) {
    b.iter(|| {
        Pixel::new(bb(0.5), bb(0.5), bb(0.5)).xyz_to_lab()
    })
}
#[bench] fn bench_from_lab(b: &mut Bencher) {
    b.iter(|| {
        Pixel::new(bb(0.5), bb(0.5), bb(0.5)).lab_to_xyz()
    })
}
#[bench] fn bench_to_xyz(b: &mut Bencher) {
    b.iter(|| {
        Pixel::new(bb(0.5), bb(0.5), bb(0.5)).rgb_to_xyz()
    })
}
#[bench] fn bench_from_xyz(b: &mut Bencher) {
    b.iter(|| {
        Pixel::new(bb(0.5), bb(0.5), bb(0.5)).xyz_to_rgb()
    })
}
#[bench] fn bench_decode(b: &mut Bencher) {
    b.iter(|| {
        Pixel::decode(bb(40), bb(41), bb(42))
    })
}
#[bench] fn bench_encode(b: &mut Bencher) {
    b.iter(|| {
        Pixel::new(bb(0.5), bb(0.5), bb(0.5)).encode()
    })
}
#[allow(non_snake_case)]
fn gen1() {
    use na::{Inv, Iterable, Mat3, Vec3};
    let (xr, yr) = (0.64, 0.33);
    let (xg, yg) = (0.30, 0.60);
    let (xb, yb) = (0.15, 0.06);
    let (xw, yw) = (0.31271, 0.32902);
    let (Xw, Yw, Zw) = (xw / yw, 1., (1. - xw - yw) / yw);
    let (Xr, Xg, Xb) = (xr / yr, xg / yg, xb / yb);
    let (Yr, Yg, Yb) = (1., 1., 1.);
    let (Zr, Zg, Zb) = ((1. - xr - yr) / yr, (1. - xg - yg) / yg, (1. - xb - yb) / yb);
    let mat = Mat3::new(Xr, Xg, Xb, Yr, Yg, Yb, Zr, Zg, Zb);
    let &[Sr, Sg, Sb] = (mat.inv().unwrap() * Vec3::new(Xw, Yw, Zw)).as_array();
    let M = Mat3::new(Sr * Xr, Sg * Xg, Sb * Xb, Sr * Yr, Sg * Yg, Sb * Yb, Sr * Zr, Sg * Zg, Sb * Zb);
    let Mi = M.inv().unwrap();
    for n in M.iter() {
        print!("{:.10e}, ", n);
    }
    println!("");
    for n in Mi.iter() {
        print!("{:.10e}, ", n);
    }
}
fn gen2() {
    fn c(x: f64) -> f64 {
        if x <= 0.04045 { x / 12.92 }
        else { ((x + 0.055) / (1. + 0.055)).powf(2.4) }
    }
    for i in 0..256 {
        if i % 5 == 0 { println!("") }
        print!("{:.10e}, ", c(i as f64 / 255.))
    }
}
#[allow(non_snake_case)]
fn main() {
    let m = 25.;
    let c0 = 0.5;
    let c1 = 3f32.sqrt() / 2.;
    let c2 = 6f32.sqrt() / 3.;
    let c3 = 3f32.sqrt() / 3.;
    let mut v = Vec::new();
    for x in -20..20 {
        for y in -20..20 {
            for z in 0..20 {
                let (xf, yf, zf) = (x as f32 * m, y as f32 * m, z as f32 * m);
                let mut a = xf;
                if y % 2 == 0 { a += m * c0 };
                let mut b = yf * c1;
                if z % 2 == 0 { b += m * c3 };
                let l = zf * c2;
                let lab = Pixel::new(l, a, b);
                let xyz = lab.lab_to_xyz();
                let rgb = xyz.xyz_to_rgb();
                let (r, g, b) = (rgb.x(), rgb.y(), rgb.z());
                if r < 0. || r > 1. || g < 0. || g > 1. || b < 0. || b > 1. { continue }
                let srgb = rgb.encode();
                let (r, g, b) = ((srgb.x() * 255.) as u8, (srgb.y() * 255.) as u8, (srgb.z() * 255.) as u8);
                let dist = |x1, y1, z1, x2, y2, z2| {
                    (x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2) + (z1 - z2) * (z1 - z2)
                };
                let d1: f32 = dist(lab.x(), lab.y(), lab.z(), 0., 0., 0.).sqrt();
                let d2: f32 = dist(lab.x(), lab.y(), lab.z(), 100., 0., 0.).sqrt();
                v.push(((lab.x(), lab.y(), lab.z()), (r, g, b), d1.min(d2)));
            }
        }
    }
    v.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    for &(lab, rgb, d) in &v {
        println!("#{:02X}{:02X}{:02X} = ({:.0}, {:.0}, {:.0}) dist={:.0}",
            rgb.0, rgb.1, rgb.2, lab.0, lab.1, lab.2, d);
    }
    println!("Total of {}", v.len());
    // let p = p.lab_to_xyz().xyz_to_rgb();
    // println!("{:?}", p);
    // let p = p.encode();
    // println!("{:?}", p);
    // println!("#{:02x}{:02x}{:02x}", (p.0.x * 255.) as u8, (p.0.y * 255.) as u8, (p.0.z * 255.) as u8);
}
