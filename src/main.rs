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

mod wincon {
    use kernel32 as k32;
    use winapi as w;
    use std::mem::{size_of_val, zeroed};
    use std::old_io::{IoError, IoResult};
    use std::os::{last_os_error};
    use std::ptr::{self};
    #[derive(Copy, Debug)]
    pub enum Std {
        Input,
        Output,
        Error,
    }
    #[derive(Debug)]
    pub struct Console(w::HANDLE);
    impl Console {
        pub fn get(kind: Std) -> IoResult<Console> {
            let kind = match kind {
                Std::Input => w::STD_INPUT_HANDLE,
                Std::Output => w::STD_OUTPUT_HANDLE,
                Std::Error => w::STD_ERROR_HANDLE,
            };
            let handle = unsafe { k32::GetStdHandle(kind) };
            if handle == w::INVALID_HANDLE_VALUE { Err(IoError::last_error()) }
            else { Ok(Console(handle)) }
        }
        pub fn get_font_info(&self) -> IoResult<FontInfo> {
            let mut info = unsafe { zeroed() };
            match unsafe { k32::GetCurrentConsoleFont(
                self.0, w::FALSE, &mut info as w::PCONSOLE_FONT_INFO,
            ) } {
                0 => Err(IoError::last_error()),
                _ => Ok(FontInfo(info)),
            }
        }
        pub fn get_info_ex(&self) -> IoResult<ConInfoEx> {
            let mut info: w::CONSOLE_SCREEN_BUFFER_INFOEX = unsafe { zeroed() };
            info.cbSize = size_of_val(&info) as w::ULONG;
            match unsafe { k32::GetConsoleScreenBufferInfoEx(
                self.0, &mut info as w::PCONSOLE_SCREEN_BUFFER_INFOEX,
            ) } {
                0 => Err(IoError::last_error()),
                _ => Ok(ConInfoEx(info)),
            }
        }
        pub fn set_info_ex(&self, info: &ConInfoEx) -> IoResult<()> {
            let mut info = info.0;
            info.srWindow.Right += 1;
            info.srWindow.Bottom += 1;
            match unsafe { k32::SetConsoleScreenBufferInfoEx(
                self.0, &mut info as w::PCONSOLE_SCREEN_BUFFER_INFOEX,
            ) } {
                0 => Err(IoError::last_error()),
                _ => Ok(()),
            }
        }
        pub fn write_output(&self, buf: &[CharInfo], size: (w::SHORT, w::SHORT)) {
            unimplemented!()
        }
        pub fn read(&self, buf: &mut [u16]) -> IoResult<u32> {
            let mut read = 0;
            match unsafe { k32::ReadConsoleW(
                self.0, buf.as_mut_ptr() as w::LPVOID, buf.len() as w::DWORD,
                &mut read as w::LPDWORD, ptr::null_mut(),
            ) } {
                0 => Err(IoError::last_error()),
                _ => Ok(read),
            }
        }
    }
    #[derive(Copy, Debug)]
    pub struct ConInfoEx(w::CONSOLE_SCREEN_BUFFER_INFOEX);
    impl ConInfoEx {
        pub fn set_colors(&mut self, colors: &[w::COLORREF; 16]) {
            self.0.ColorTable = *colors;
        }
    }
    #[derive(Copy, Debug)]
    pub struct FontInfo(w::CONSOLE_FONT_INFO);
    impl FontInfo {
        pub fn width(&self) -> w::SHORT { self.0.dwFontSize.X }
        pub fn height(&self) -> w::SHORT { self.0.dwFontSize.Y }
    }
    fn check_bool(b: w::BOOL) {
        if b != w::FALSE { return }
        panic!("{}", last_os_error());
    }
    #[derive(Copy, Debug)]
    pub struct Attributes(w::WORD);
    impl Attributes {
        pub fn new(n: u16) -> Attributes { Attributes(n) }
    }
    #[derive(Copy, Debug)]
    pub struct CharInfo(w::CHAR_INFO);
    impl CharInfo {
        pub fn new(ch: char, at: Attributes) -> CharInfo {
            CharInfo(w::CHAR_INFO {
                Char: ch as u16,
                Attributes: at.0,
            })
        }
    }
}

#[derive(Debug)]
struct Coverage(char, f32);
impl PartialEq for Coverage {
    fn eq(&self, o: &Coverage) -> bool {
        self.1 == o.1
    }
}

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
struct Image {
    pixels: Vec<Cell<Pixel>>,
    width: u32,
    height: u32,
}
impl Image {
    fn get(&self, x: u32, y: u32) -> Pixel {
        self.pixels[(y * self.width + x) as usize].get()
    }
}
#[derive(Debug)]
struct Ascii {
    console: wincon::Console,
    csize: (u32, u32),
    cinfo: wincon::ConInfoEx,
}
impl Ascii {
    fn new() -> Ascii {
        let console = wincon::Console::get(wincon::Std::Output).unwrap();
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
        info.set_colors(&[
            random::<u32>() & 0x00ffffff, random::<u32>() & 0x00ffffff,
            random::<u32>() & 0x00ffffff, random::<u32>() & 0x00ffffff,
            random::<u32>() & 0x00ffffff, random::<u32>() & 0x00ffffff,
            random::<u32>() & 0x00ffffff, random::<u32>() & 0x00ffffff,
            random::<u32>() & 0x00ffffff, random::<u32>() & 0x00ffffff,
            random::<u32>() & 0x00ffffff, random::<u32>() & 0x00ffffff,
            random::<u32>() & 0x00ffffff, random::<u32>() & 0x00ffffff,
            random::<u32>() & 0x00ffffff, random::<u32>() & 0x00ffffff,
        ]);
        self.console.set_info_ex(&info).unwrap();
        let mut buf = [0; 100];
        let cin = wincon::Console::get(wincon::Std::Input).unwrap();
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
