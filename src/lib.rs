// The MIT License (MIT)
//
// Copyright (c) 2015 Johan Johansson
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! Capture the screen with xlib

#![feature(unique)]
#![allow(dead_code, non_upper_case_globals, non_camel_case_types)]

extern crate x11;
extern crate libc;

use ffi::*;
use x11::xlib::{ self, Display, Window };
use libc::{ c_ulong, c_int };
use std::ptr::{ self, Unique };
use std::slice;

pub mod ffi;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(C, packed)]
pub struct RGB8 {
	r: u8,
	g: u8,
	b: u8,
}

/// From a bitmask for a color in a pixel, calculate the color size in bits and the bitshift
fn mask_size_and_shift(mut mask: c_ulong) -> (c_ulong, u16) {
	let (mut bits, mut shift) = (0, 0);
	let mut prev = 0;
	for _ in 0..(c_ulong::max_value() as f64).log2() as u16 {
		if mask & 1 == 1 {
			bits += 1;
			prev = 1;
		} else if prev == 1 {
			break;
		} else {
			shift += 1;
		}
		mask >>= 1;
	}
	(bits, shift)
}

struct Desktop {
	display: Unique<Display>,
	window: Window,
	width: u32, height: u32,
}
impl Desktop {
	fn new(display: Unique<Display>, screen: c_int) -> Desktop {
		let mut root_window = 0;
		let (mut root_x, mut root_y) = (0, 0);
		let (mut root_width, mut root_height, mut root_border_width) = (0, 0, 0);
		let mut root_pixel_depth = 0;
		if unsafe { xlib::XGetGeometry(*display, xlib::XRootWindow(*display, screen),
			&mut root_window,
			&mut root_x, &mut root_y,
			&mut root_width, &mut root_height, &mut root_border_width,
			&mut root_pixel_depth) == 0 }
		{
			panic!("XGetGeometry failed");
		}

		Desktop{
			display: display,
			window: root_window,
			width: root_width, height: root_height,
		}
	}
}
impl Drop for Desktop {
	fn drop(&mut self) {
		unsafe {
			xlib::XCloseDisplay(*self.display);
		}
	}
}

pub struct Capturer {
	desktop: Desktop,
}
impl Capturer {
	pub fn new() -> Capturer {
		// use the information from the environment variable DISPLAY to create the X connection:	
		let display = unsafe { Unique::new(xlib::XOpenDisplay(ptr::null_mut())) };
		if display.is_null() {
			panic!("Unable to connect X server");
		}

		
		let screen = unsafe { xlib::XDefaultScreen(*display) };
		Capturer{ desktop: Desktop::new(display, screen) }
	}

	pub fn capture_frame(&mut self) -> (Vec<RGB8>, (u32, u32)) {
		let image_ptr = unsafe { xlib::XGetImage(*self.desktop.display, self.desktop.window,
			0, 0,
			self.desktop.width, self.desktop.height,
			AllPlanes, ZPixmap) };
		if image_ptr.is_null() {
			panic!("XGetImage failed");
		}
		let image = unsafe { &mut *image_ptr };

		println!("w, h: {:?}; pad: {}; bpl: {}", (image.width, image.height), image.bitmap_pad, image.bytes_per_line);

		unsafe { if image.depth == 24 && image.bits_per_pixel == 32 &&
			image.red_mask == 0xFF0000 && image.green_mask == 0xFF00 && image.blue_mask == 0xFF
		{
			// It's plain (RGB8 + padding)s in memory
			let raw_img_data = slice::from_raw_parts(image.data as *mut (RGB8, u8),
				image.width as usize * image.height as usize
			).iter()
				.map(|&(pixel, _)| pixel)
				.collect();

			xlib::XFree(image_ptr as *mut _);

			(raw_img_data, (image.width as u32, image.height as u32))
		} else {
			xlib::XFree(image_ptr as *mut _);
			panic!("WRONG LAYOUT")
		} }
	}
}

#[test]
fn test() {
	let mut capturer = Capturer::new();
	println!("Any non-black: {}", capturer.capture_frame().0.iter()
		.any(|p| p.r != 0 || p.g != 0 || p.b != 0));
}