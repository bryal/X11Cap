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

extern crate x11;

#[test]
fn test() {
	use x11::xlib;
	use std::{ ptr, thread };

	unsafe {

	println!("Init\n");

	// use the information from the environment variable DISPLAY to create the X connection:	
	let display = match xlib::XOpenDisplay(ptr::null_mut()) {
		null if null.is_null() => panic!("Unable to connect X server"),
		display => display,
	};

	let screen_num = xlib::XDefaultScreen(display);
	let (black_pixel, white_pixel) = (xlib::XBlackPixel(display, screen_num),
		xlib::XWhitePixel(display, screen_num));
	println!("Screen: {}, Black: {}, White: {}", screen_num, black_pixel, white_pixel);

	// Once the display is initialized, create the window
	let (x, y) = (0, 0);
	let (width, height, border_width) = (400, 500, 5);
	let (border_color, background_color) = (black_pixel, white_pixel);
	let window = xlib::XCreateSimpleWindow(display,
		xlib::XRootWindow(display, screen_num),
		x, y,
		width, height, border_width,
		border_color, background_color);
	println!("Window: {}", window);

	// this routine determines which types of input are allowed in the input.
	xlib::XSelectInput(display, window,
		xlib::ExposureMask | xlib::ButtonPressMask | xlib::KeyPressMask);

	// create the Graphics Context
	let graphics_context = xlib::XCreateGC(display, window, 0, ptr::null_mut());

	xlib::XMapWindow(display, window);
	xlib::XFlush(display);

	thread::sleep_ms(3000);

	println!("Close");
	// it is good programming practice to return system resources to the system...
	xlib::XFreeGC(display, graphics_context);
	xlib::XDestroyWindow(display, window);
	xlib::XCloseDisplay(display);

	}
}