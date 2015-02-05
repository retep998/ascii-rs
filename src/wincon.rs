// Copyright © 2014, Peter Atashian

use kernel32 as k32;
use winapi::*;
use std::mem::{size_of_val, zeroed};
use std::old_io::{IoError, IoResult};
use std::ptr::{self};
#[derive(Copy, Debug)]
pub enum Std {
    Input,
    Output,
    Error,
}
#[derive(Debug)]
pub struct Console(HANDLE);
impl Console {
    pub fn get(kind: Std) -> IoResult<Console> {
        let kind = match kind {
            Std::Input => STD_INPUT_HANDLE,
            Std::Output => STD_OUTPUT_HANDLE,
            Std::Error => STD_ERROR_HANDLE,
        };
        let handle = unsafe { k32::GetStdHandle(kind) };
        if handle == INVALID_HANDLE_VALUE { Err(IoError::last_error()) }
        else { Ok(Console(handle)) }
    }
    pub fn get_font_info(&self) -> IoResult<FontInfo> {
        let mut info = unsafe { zeroed() };
        match unsafe { k32::GetCurrentConsoleFont(
            self.0, FALSE, &mut info as PCONSOLE_FONT_INFO,
        ) } {
            0 => Err(IoError::last_error()),
            _ => Ok(FontInfo(info)),
        }
    }
    pub fn get_info_ex(&self) -> IoResult<ConInfoEx> {
        let mut info: CONSOLE_SCREEN_BUFFER_INFOEX = unsafe { zeroed() };
        info.cbSize = size_of_val(&info) as ULONG;
        match unsafe { k32::GetConsoleScreenBufferInfoEx(
            self.0, &mut info as PCONSOLE_SCREEN_BUFFER_INFOEX,
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
            self.0, &mut info as PCONSOLE_SCREEN_BUFFER_INFOEX,
        ) } {
            0 => Err(IoError::last_error()),
            _ => Ok(()),
        }
    }
    pub fn write_output(
        &self, buf: &[CharInfo], size: Vec2, index: Vec2, mut region: Rect,
    ) -> IoResult<Rect> {
        match unsafe { k32::WriteConsoleOutputW(
            self.0, buf.as_ptr() as *const CHAR_INFO, size.0, index.0,
            &mut region.0 as PSMALL_RECT,
        ) } {
            0 => Err(IoError::last_error()),
            _ => Ok(region),
        }
    }
    pub fn read(&self, buf: &mut [u16]) -> IoResult<u32> {
        let mut read = 0;
        match unsafe { k32::ReadConsoleW(
            self.0, buf.as_mut_ptr() as LPVOID, buf.len() as DWORD,
            &mut read as LPDWORD, ptr::null_mut(),
        ) } {
            0 => Err(IoError::last_error()),
            _ => Ok(read),
        }
    }
}
#[derive(Copy, Debug)]
pub struct ConInfoEx(CONSOLE_SCREEN_BUFFER_INFOEX);
impl ConInfoEx {
    pub fn set_colors(&mut self, colors: &[COLORREF; 16]) {
        self.0.ColorTable = *colors;
    }
    pub fn window(&self) -> Rect { Rect(self.0.srWindow) }
}
#[derive(Copy, Debug)]
pub struct FontInfo(CONSOLE_FONT_INFO);
impl FontInfo {
    pub fn width(&self) -> i16 { self.0.dwFontSize.X }
    pub fn height(&self) -> i16 { self.0.dwFontSize.Y }
}
#[derive(Copy, Debug)]
pub struct Attr(WORD);
impl Attr {
    pub fn new(n: u16) -> Attr { Attr(n) }
    pub fn new_color(fg: u16, bg: u16) -> Attr {
        let (fg, bg) = (fg & 0xf, bg & 0xf);
        Attr(fg | (bg << 4))
    }
}
#[derive(Copy, Debug)]
pub struct CharInfo(CHAR_INFO);
impl CharInfo {
    pub fn new(ch: char, at: Attr) -> CharInfo {
        CharInfo(CHAR_INFO {
            Char: ch as u16,
            Attributes: at.0,
        })
    }
}
#[derive(Copy, Debug)]
pub struct Vec2(COORD);
impl Vec2 {
    pub fn new(x: i16, y: i16) -> Vec2 {
        Vec2(COORD { X: x, Y: y })
    }
}
#[derive(Copy, Debug)]
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