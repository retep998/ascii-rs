// Copyright © 2014, Peter Atashian

#![allow(dead_code)]
#![feature(collections, core, io, os, path, rand, slicing_syntax)]

extern crate image;
extern crate "kernel32-sys" as kernel32;
extern crate "nalgebra" as na;
extern crate winapi;

use colors::{Pixel, RGB};
use std::num::{Float, ToPrimitive};
use std::old_io::stdio::{stdin};
use std::rand::{random};
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
        let csize = (finfo.width().to_u32().unwrap(), finfo.height().to_u32().unwrap());
        let cinfo = console.get_info_ex().unwrap();
        Ascii {
            console: console,
            csize: csize,
            cinfo: cinfo,
        }
    }
    fn draw(&self) {
        let info = self.console.get_info_ex().unwrap();
        let win = info.window();
        let win = Rect::new(
            win.left(), win.top(), win.right(), win.bottom(),
        );
        let (w, h) = (win.right() - win.left(), win.bottom() - win.top());
        let buf = (0..w * h).map(|_| {
            CharInfo::new('X', Attr::new(random::<u16>() & 0xff))
        }).collect::<Vec<_>>();
        self.console.write_output(
            &buf, Vec2::new(w, h), Vec2::new(0, 0), win,
        ).unwrap();
    }
    fn convert(&self, name: &str) {
        self.display("Loading image");
        let img = image::open(&Path::new(name)).unwrap();
        self.display("Converting to RGBA");
        let img = img.to_rgba();
        let mut count = 0;
        for r in 1..256 {
            for g in 1..256 {
                for b in 1..256 {
                    let srgb = Pixel::new(r as f32 / 255., g as f32 / 255., b as f32 / 255.);
                    let rgb = Pixel::from_srgb(r as u8, g as u8, b as u8);
                    let trans = rgb.to_xyz().to_rgb().to_srgb();
                    let diff = (trans - srgb).magnitude().log2();
                    if diff > -10. { count += 1 }
                }
            }
            self.display(&format!("{}%", r * 100 / 255));
        }
        self.display(&format!("Failures: {}/{}", count, 255 * 255 * 255));
        stdin().read_line().unwrap();
    }
    fn display(&self, s: &str) {
        let mut buf = s.chars().map(|c| (c as u32).to_u16().unwrap()).collect::<Vec<_>>();
        let info = self.console.get_info_ex().unwrap();
        let win = info.window();
        let width = win.right() - win.left();
        buf.resize(width.to_uint().unwrap(), 32);
        let pos = info.cursor();
        self.console.write_output_chars(&buf, pos).unwrap();
    }
}
impl Drop for Ascii {
    fn drop(&mut self) {
        self.console.set_info_ex(&self.cinfo).unwrap();
    }
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
    let ascii = Ascii::new();
    let args = std::os::args();
    if args.len() < 2 {
        println!("Nothing to do!");
    } else {
        ascii.convert(&args[1]);
    }
}
