// Copyright © 2014, Peter Atashian

#![allow(dead_code)]
#![feature(core, io, os, path, rand, slicing_syntax)]

extern crate image;
extern crate "kernel32-sys" as kernel32;
extern crate winapi;

use std::cell::{Cell};
use std::num::{Float, ToPrimitive};
use std::ops::{Add, Div, Mul, Sub};
use std::rand::{random};
use wincon::{Attr, CharInfo, ConInfoEx, Console, Rect, Std, Vec2};

mod wincon;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Pixel(f32, f32, f32);
impl Pixel {
    fn gray(&self) -> f32 {
        (self.0 + self.1 + self.2) * (1. / 3.)
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
    fn zero() -> Pixel {
        Pixel(0., 0., 0.)
    }
}
impl<'a> Add<&'a Pixel> for &'a Pixel {
    type Output = Pixel;
    fn add(self, o: &'a Pixel) -> Pixel {
        Pixel(self.0 + o.0, self.1 + o.1, self.2 + o.2)
    }
}
impl<'a> Sub<&'a Pixel> for &'a Pixel {
    type Output = Pixel;
    fn sub(self, o: &'a Pixel) -> Pixel {
        Pixel(self.0 - o.0, self.1 - o.1, self.2 - o.2)
    }
}
impl<'a> Mul<&'a Pixel> for &'a Pixel {
    type Output = Pixel;
    fn mul(self, o: &'a Pixel) -> Pixel {
        Pixel(self.0 * o.0, self.1 * o.1, self.2 * o.2)
    }
}
impl<'a> Div<&'a Pixel> for &'a Pixel {
    type Output = Pixel;
    fn div(self, o: &'a Pixel) -> Pixel {
        Pixel(self.0 / o.0, self.1 / o.1, self.2 / o.2)
    }
}

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
    fn convert(&self, name: &str) {
        let _path = Path::new(name);
        let mut info = self.cinfo;
        self.console.set_info_ex(&info).unwrap();
        let ci = CharInfo::new('X', Attr::new_color(random(), random()));
        let (w, h) = (20, 10);
        let window = info.window();
        let window = Rect::new(
            window.left() + 10, window.top() + 10, window.left() + 10 + w, window.top() + 10 + h,
        );
        let buf = (0..w * h).map(|_| {
            CharInfo::new('X', Attr::new(random::<u16>() & 0xff))
        }).collect::<Vec<_>>();
        self.console.write_output(
            &buf, Vec2::new(w, h), Vec2::new(0, 0), window,
        ).unwrap();
        let mut buf = [0; 100];
        let cin = Console::get(Std::Input).unwrap();
        cin.read(&mut buf).unwrap();
    }
}
impl Drop for Ascii {
    fn drop(&mut self) {
        self.console.set_info_ex(&self.cinfo).unwrap();
    }
}

fn main() {
    let ascii = Ascii::new();
    let args = std::os::args();
    if args.len() < 2 {
        println!("Nothing to do!");
    } else {
        ascii.convert(&args[1]);
    }
}
