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

// 	// this routine determines which types of input are allowed in the input.
// 	xlib::XSelectInput(display, window,
// 		xlib::ExposureMask | xlib::ButtonPressMask | xlib::KeyPressMask);

// 	// create the Graphics Context
// 	let graphics_context = xlib::XCreateGC(display, window, 0, ptr::null_mut());

// 	xlib::XMapWindow(display, window);
// 	xlib::XFlush(display);

// 	std::thread::sleep_ms(1000);

// 	// it is good programming practice to return system resources to the system...
// 	xlib::XFreeGC(display, graphics_context);
// 	xlib::XDestroyWindow(display, window);
// 	xlib::XCloseDisplay(display);

// 	}
// }

#[test]
fn test_shm() {
	use x11::xlib;
	use std::{ ptr, mem };
	use ffi::*;
	use x11::xlib::Bool;
	use libc::{ c_uint, size_t, c_char, c_void, c_int };

	unsafe {

	// use the information from the environment variable DISPLAY to create the X connection:	
	let display = xlib::XOpenDisplay(ptr::null_mut());
	if display.is_null() {
		panic!("Unable to connect X server");
	}

	println!("MIT-SHM available: {}", XShmQueryExtension(display) != 0);
	let (mut major_ver, mut minor_ver) = (0, 0);
	let mut shared_pixmaps: Bool = 0;
	XShmQueryVersion(display, &mut major_ver, &mut minor_ver, &mut shared_pixmaps);
	println!("SHM: major: {}, minor: {}, shared pixmaps: {}", major_ver, minor_ver, shared_pixmaps);

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

	let mut shm_segment_info = mem::zeroed();
	let image_ptr = XShmCreateImage(display, xlib::XDefaultVisual(display, screen_num),
		xlib::XDefaultDepth(display, screen_num) as c_uint,
		ZPixmap, ptr::null_mut(),
		&mut shm_segment_info,
		root_width, root_height);
	if image_ptr.is_null() {
		panic!("XShmCreateImage returned null pointer");
	}
	let image = &mut *image_ptr;

	shm_segment_info.shmid = shmget(IPC_PRIVATE,
		(image.bytes_per_line * image.height) as size_t,
		IPC_CREAT | 0777);
	if shm_segment_info.shmid == -1 {
		panic!("shmget failed");
	}

	shm_segment_info.shmaddr = shmat(shm_segment_info.shmid, ptr::null_mut(), 0) as *mut c_char;
	if shm_segment_info.shmaddr.is_null() {
		panic!("XShmCreateImage returned null pointer");
	}
	image.data = shm_segment_info.shmaddr;

	shm_segment_info.read_only = 0;

	if XShmAttach(display, &mut shm_segment_info) == 0 {
		panic!("XShmAttach failed");
	}

	for _ in 0..10 {
		if XShmGetImage(display, root_window, image, root_x, root_y, AllPlanes) == 0 {
			panic!("XShmGetImage failed");
		}

		println!("width {}, height {}, xoffset {}, format {}, byte_order {}, bitmap_unity {}, \
			bitmap_bit_order {}, bitmap_pad {}, depth {}, bytes_per_line {}, bits_per_pixel {}, \
			red_mask {:b}, green_mask {:b}, blue_mask {:b}",
			image.width, image.height, image.xoffset, image.format, image.byte_order,
			image.bitmap_unity, image.bitmap_bit_order, image.bitmap_pad, image.depth,
			image.bytes_per_line, image.bits_per_pixel,
			image.red_mask, image.green_mask, image.blue_mask);

		// Factor to multiply color value with for it to fit in a u8
		let (red_size, red_shift) = mask_size_and_shift(image.red_mask);
		let (green_size, green_shift) = mask_size_and_shift(image.red_mask);
		let (blue_size, blue_shift) = mask_size_and_shift(image.red_mask);
		let (red_mask, green_mask, blue_mask) = (image.red_mask, image.green_mask, image.blue_mask);
		let masked_pixel_to_rgb: Box<Fn(c_ulong) -> RGB8> =
			if red_size == 8 && green_size == 8 && blue_size == 8
		{

			Box::new(move |pixel| RGB8{
				r: ((pixel & red_mask) >> red_shift) as u8,
				g: ((pixel & green_mask) >> green_shift) as u8,
				b: ((pixel & blue_mask) >> blue_shift) as u8,
			})
		} else {
			let red_size_factor = 8.0 / red_size as f32;
			let green_size_factor = 8.0 / green_size as f32;
			let blue_size_factor = 8.0 / blue_size as f32;
			Box::new(move |pixel| RGB8{
				r: (((pixel & red_mask) >> red_shift) as f32 * red_size_factor) as u8,
				g: (((pixel & green_mask) >> green_shift) as f32 * green_size_factor) as u8,
				b: (((pixel & blue_mask) >> blue_shift) as f32 * blue_size_factor) as u8,
			})
		};

		let mut pixel_buf = Vec::with_capacity(image.width as usize * image.height as usize);
		for row in 0..image.height {
			for col in 0..image.width {
				pixel_buf.push(masked_pixel_to_rgb(xlib::XGetPixel(image, col, row)));
			}
		}

		println!("Tot color: {:?}", pixel_buf.iter()
			.fold((0, 0, 0), |(r, g, b), p| (r + p.r as u64, g + p.g as u64, b + p.b as u64))
		);
	}

	XShmDetach(display, &mut shm_segment_info);
	xlib::XDestroyImage(image);
	shmdt(shm_segment_info.shmaddr as *mut c_void);
	shmctl(shm_segment_info.shmid, IPC_RMID, ptr::null_mut());

	xlib::XCloseDisplay(display);

	}
}