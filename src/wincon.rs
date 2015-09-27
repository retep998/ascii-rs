// Copyright © 2014, Peter Atashian

use kernel32 as k32;
use winapi::*;
use std::mem::{size_of_val, zeroed};
use std::io::{Error, Result};
use std::path::Path;
use std::ptr::{self};
#[derive(Clone, Copy, Debug)]
pub enum Std {
    Input,
    Output,
    Error,
}
#[derive(Debug)]
pub struct Console(HANDLE);
impl Console {
    pub fn get(kind: Std) -> Result<Console> {
        let kind = match kind {
            Std::Input => STD_INPUT_HANDLE,
            Std::Output => STD_OUTPUT_HANDLE,
            Std::Error => STD_ERROR_HANDLE,
        };
        let handle = unsafe { k32::GetStdHandle(kind) };
        if handle == INVALID_HANDLE_VALUE { Err(Error::last_os_error()) }
        else { Ok(Console(handle)) }
    }
    pub fn get_font_info(&self) -> Result<FontInfo> {
        let mut info = unsafe { zeroed() };
        match unsafe { k32::GetCurrentConsoleFont(
            self.0, FALSE, &mut info as PCONSOLE_FONT_INFO,
        ) } {
            0 => Err(Error::last_os_error()),
            _ => Ok(FontInfo(info)),
        }
    }
    pub fn get_info_ex(&self) -> Result<ConInfoEx> {
        let mut info: CONSOLE_SCREEN_BUFFER_INFOEX = unsafe { zeroed() };
        info.cbSize = size_of_val(&info) as u32;
        match unsafe { k32::GetConsoleScreenBufferInfoEx(
            self.0, &mut info as PCONSOLE_SCREEN_BUFFER_INFOEX,
        ) } {
            0 => Err(Error::last_os_error()),
            _ => {
                info.srWindow.Right += 1;
                info.srWindow.Bottom += 1;
                Ok(ConInfoEx(info))
            },
        }
    }
    pub fn set_info_ex(&self, info: &ConInfoEx) -> Result<()> {
        let mut info = info.0;
        match unsafe { k32::SetConsoleScreenBufferInfoEx(
            self.0, &mut info as PCONSOLE_SCREEN_BUFFER_INFOEX,
        ) } {
            0 => Err(Error::last_os_error()),
            _ => Ok(()),
        }
    }
    pub fn write_output(
        &self, buf: &[CharInfo], size: Vec2, index: Vec2, mut region: Rect,
    ) -> Result<Rect> {
        match unsafe { k32::WriteConsoleOutputW(
            self.0, buf.as_ptr() as *const CHAR_INFO, size.0, index.0,
            &mut region.0 as PSMALL_RECT,
        ) } {
            0 => Err(Error::last_os_error()),
            _ => Ok(region),
        }
    }
    pub fn write_output_chars(&self, buf: &[u16], pos: Vec2) -> Result<u32> {
        let mut written = 0;
        match unsafe { k32::WriteConsoleOutputCharacterW(
            self.0, buf.as_ptr(), buf.len() as u32, pos.0, &mut written as LPDWORD,
        )} {
            0 => Err(Error::last_os_error()),
            _ => Ok(written),
        }
    }
    pub fn read(&self, buf: &mut [u16]) -> Result<u32> {
        let mut read = 0;
        match unsafe { k32::ReadConsoleW(
            self.0, buf.as_mut_ptr() as LPVOID, buf.len() as u32,
            &mut read as LPDWORD, ptr::null_mut(),
        ) } {
            0 => Err(Error::last_os_error()),
            _ => Ok(read),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ConInfoEx(CONSOLE_SCREEN_BUFFER_INFOEX);
impl ConInfoEx {
    pub fn set_colors(&mut self, colors: &[COLORREF; 16]) {
        self.0.ColorTable = *colors;
    }
    pub fn window(&self) -> Rect { Rect(self.0.srWindow) }
    pub fn cursor(&self) -> Vec2 { Vec2(self.0.dwCursorPosition) }
}
#[derive(Clone, Copy, Debug)]
pub struct FontInfo(CONSOLE_FONT_INFO);
impl FontInfo {
    pub fn width(&self) -> i16 { self.0.dwFontSize.X }
    pub fn height(&self) -> i16 { self.0.dwFontSize.Y }
}
#[derive(Clone, Copy, Debug)]
pub struct Attr(WORD);
impl Attr {
    pub fn new(n: u16) -> Attr { Attr(n) }
    pub fn new_color(fg: u16, bg: u16) -> Attr {
        let (fg, bg) = (fg & 0xf, bg & 0xf);
        Attr(fg | (bg << 4))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CharInfo(CHAR_INFO);
impl CharInfo {
    pub fn new(ch: char, at: Attr) -> CharInfo {
        CharInfo(CHAR_INFO {
            Char: (ch as u32) as u16,
            Attributes: at.0,
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Vec2(COORD);
impl Vec2 {
    pub fn new(x: i16, y: i16) -> Vec2 {
        Vec2(COORD { X: x, Y: y })
    }
    pub fn x(&self) -> i16 { self.0.X }
    pub fn y(&self) -> i16 { self.0.Y }
}
#[derive(Clone, Copy, Debug)]
pub struct Rect(SMALL_RECT);
impl Rect {
    pub fn new(left: i16, top: i16, right: i16, bottom: i16) -> Rect {
        Rect(SMALL_RECT {
            Left: left, Top: top, Right: right, Bottom: bottom,
        })
    }
    pub fn left(&self) -> i16 { self.0.Left }
    pub fn top(&self) -> i16 { self.0.Top }
    pub fn right(&self) -> i16 { self.0.Right }
    pub fn bottom(&self) -> i16 { self.0.Bottom }
}
#[derive(Clone, Copy, Debug)]
pub struct Color(COLORREF);
impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
    }
}
