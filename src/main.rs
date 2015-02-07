// Copyright © 2014, Peter Atashian

#![allow(dead_code)]
#![feature(collections, core, io, os, path, rand, slicing_syntax)]

extern crate image;
extern crate "kernel32-sys" as kernel32;
extern crate "nalgebra" as na;
extern crate winapi;

use colors::{Pixel, RGB, ToRGB};
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
        for r in 0..256 {
            for g in 0..256 {
                for b in 0..256 {
                    let rgb: Pixel<RGB> = Pixel::new(r as f32 / 255., g as f32 / 255., b as f32 / 255.);
                    let xyz = rgb.to_xyz();
                    let trans = xyz.to_rgb();
                    let log = (trans - rgb).magnitude().log2();
                    if log > -20. { count += 1 }
                }
            }
            self.display(&format!("{}%", r * 100 / 255));
        }
        self.display(&format!("Failures: {}/{}", count, 256 * 256 * 256));
        println!("");
        let rgb: Pixel<RGB> = Pixel::new(1., 1., 1.);
        println!("{:?}", rgb.to_xyz());
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
fn main() {
    let ascii = Ascii::new();
    let args = std::os::args();
    if args.len() < 2 {
        println!("Nothing to do!");
    } else {
        ascii.convert(&args[1]);
    }
}
