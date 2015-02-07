// Copyright © 2014, Peter Atashian

use na::{Mat3, Norm, Vec3};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Copy, Debug)]
#[allow(non_camel_case_types)]
pub struct xyY;
#[derive(Copy, Debug)]
#[allow(non_camel_case_types)]
pub struct sRGB;
#[derive(Copy, Debug)]
pub struct RGB;
#[derive(Copy, Debug)]
pub struct XYZ;
#[derive(Copy, Debug)]
pub struct Lab;
#[derive(Copy, Debug, PartialEq)]
pub struct Pixel<T>(Vec3<f32>);
impl<T> Pixel<T> {
    pub fn new(_1: f32, _2: f32, _3: f32) -> Pixel<T> {
        Pixel(Vec3::new(_1, _2, _3))
    }
    pub fn magnitude(self) -> f32 {
        self.0.norm()
    }
}
impl<T> Add<Pixel<T>> for Pixel<T> {
    type Output = Pixel<T>;
    fn add(self, o: Pixel<T>) -> Pixel<T> {
        Pixel(self.0 + o.0)
    }
}
impl<T> Sub<Pixel<T>> for Pixel<T> {
    type Output = Pixel<T>;
    fn sub(self, o: Pixel<T>) -> Pixel<T> {
        Pixel(self.0 - o.0)
    }
}
impl<T> Mul<Pixel<T>> for Pixel<T> {
    type Output = Pixel<T>;
    fn mul(self, o: Pixel<T>) -> Pixel<T> {
        Pixel(self.0 * o.0)
    }
}
impl<T> Div<Pixel<T>> for Pixel<T> {
    type Output = Pixel<T>;
    fn div(self, o: Pixel<T>) -> Pixel<T> {
        Pixel(self.0 / o.0)
    }
}
pub trait ToRGB {
    fn to_rgb(self) -> Pixel<RGB>;
}
impl ToRGB for Pixel<XYZ> {
    fn to_rgb(self) -> Pixel<RGB> {
        Pixel(Mat3::new(
            3.2410032330, -1.5373989695, -0.4986158820,
            -0.9692242522, 1.8759299837, 0.0415542263,
            0.0556394199, -0.2040112061, 1.0571489772,
        ) * self.0)
    }
}
impl Pixel<RGB> {
    pub fn to_xyz(self) -> Pixel<XYZ> {
        Pixel(Mat3::new(
            0.4123865633, 0.3575914909, 0.1804504912,
            0.2126368217, 0.7151829818, 0.0721801965,
            0.0193306202, 0.1191971636, 0.9503725870,
        ) * self.0)
    }
}
impl ToRGB for Pixel<sRGB> {
    fn to_rgb(self) -> Pixel<RGB> {
        unimplemented!()
    }
}