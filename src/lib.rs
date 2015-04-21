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

#![allow(dead_code, non_upper_case_globals, non_camel_case_types)]
#![feature(step_by)]

extern crate x11;
extern crate libc;

use libc::c_ulong;

pub mod ffi;

#[derive(Debug, Eq, PartialEq)]
struct RGB8 {
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

// #[test]
// fn test_create_window() {
// 	use x11::xlib;
// 	use std::ptr;
// 	use ffi::*;

// 	unsafe {

// 	// use the information from the environment variable DISPLAY to create the X connection:	
// 	let display = match xlib::XOpenDisplay(ptr::null_mut()) {
// 		null if null.is_null() => panic!("Unable to connect X server"),
// 		display => display,
// 	};

// 	let screen_num = xlib::XDefaultScreen(display);
// 	let (black_pixel, white_pixel) = (xlib::XBlackPixel(display, screen_num),
// 		xlib::XWhitePixel(display, screen_num));

// 	// Once the display is initialized, create the window
// 	let (x, y) = (0, 0);
// 	let (width, height, border_width) = (400, 500, 5);
// 	let (border_color, background_color) = (black_pixel, white_pixel);
// 	let window = xlib::XCreateSimpleWindow(display,
// 		xlib::XRootWindow(display, screen_num),
// 		x, y,
// 		width, height, border_width,
// 		border_color, background_color);


// 	xlib::XMapWindow(display, window);
// 	xlib::XFlush(display);

// 	std::thread::sleep_ms(1000);

// 	let image_ptr = xlib::XGetImage(display, window,
// 		0, 0,
// 		width, height,
// 		AllPlanes, ZPixmap);
// 	if image_ptr.is_null() {
// 		panic!("XGetImage failed");
// 	}
// 	let image = &mut *image_ptr;

// 	println!("width {}, height {}, depth {}, bytes_per_line {}, bits_per_pixel {}",
// 		image.width, image.height, image.depth, image.bytes_per_line, image.bits_per_pixel);

// 	for row in 0..image.height {
// 		for col in 0..image.width {
// 			let pixel = xlib::XGetPixel(image, col, row);
// 			if pixel != 0 {
// 				println!("{:b}", pixel);
// 			}
// 		}
// 	}

// 	std::thread::sleep_ms(1000);

// 	// it is good programming practice to return system resources to the system...
// 	xlib::XDestroyWindow(display, window);
// 	xlib::XCloseDisplay(display);

// 	}
// }

#[test]
fn test_capture() {
	use x11::xlib;
	use std::ptr;
	use ffi::*;

	unsafe {

	// use the information from the environment variable DISPLAY to create the X connection:	
	let display = xlib::XOpenDisplay(ptr::null_mut());
	if display.is_null() {
		panic!("Unable to connect X server");
	}

	let screen_num = xlib::XDefaultScreen(display);
	println!("Screen: {}", screen_num);

	let mut root_window = 0;
	let (mut root_x, mut root_y) = (0, 0);
	let (mut root_width, mut root_height, mut root_border_width) = (0, 0, 0);
	let mut root_pixel_depth = 0;
	if xlib::XGetGeometry(display, xlib::XRootWindow(display, screen_num),
		&mut root_window,
		&mut root_x, &mut root_y,
		&mut root_width, &mut root_height, &mut root_border_width,
		&mut root_pixel_depth) == 0
	{
		panic!("XGetGeometry failed");
	}
	println!("root window: {}; x, y: {:?}; w, h, bw: {:?}; d: {:?}",
		root_window, (root_x, root_y), (root_width, root_height, root_border_width),
		root_pixel_depth);

	println!("AllPlanes: {}, ZPixmap: {}\n", AllPlanes, ZPixmap);

	let image_ptr = xlib::XGetImage(display, root_window,
		0, 0,
		root_width, root_height,
		AllPlanes, ZPixmap);
	if image_ptr.is_null() {
		panic!("XGetImage failed");
	}
	let image = &mut *image_ptr;

	println!("width {}, height {}, depth {}, bytes_per_line {}, bits_per_pixel {}",
		image.width, image.height, image.depth, image.bytes_per_line, image.bits_per_pixel);
	
	let mut pixel_buf = Vec::with_capacity((image.height as usize * image.width as usize) / 9);
	for y in (0..image.height).step_by(3) {
		for x in (0..image.width).step_by(3) {
			let mut color = xlib::XColor{ pixel: xlib::XGetPixel(image_ptr, x, y),
				red: 0, green: 0, blue: 0,
				flags: 0, pad: 0,
			};
			xlib::XQueryColor(display, xlib::XDefaultColormap(display, screen_num), &mut color);
			pixel_buf.push(RGB8{
				r: (color.red / 256) as u8,
				g: (color.green / 256) as u8,
				b: (color.blue / 256) as u8
			});
		}
	}

	let n_pixels = (image.height as u64 * image.width as u64) / 9;
	let (r_tot, g_tot, b_tot) = pixel_buf.iter()
		.fold((0, 0, 0), |(r, g, b), p| (r + p.r as u64, g + p.g as u64, b + p.b as u64));
	println!("Avg color: {:?}", (r_tot/n_pixels, g_tot/n_pixels, b_tot/n_pixels));

	xlib::XFree(image_ptr as *mut _);
	xlib::XCloseDisplay(display);

	}
}