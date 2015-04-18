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

use x11::xlib::{ Display, Bool, Visual, XImage, Drawable };
use libc::{ c_int, c_char, c_uint, c_ushort, size_t, c_void, c_ulong, time_t, pid_t, uid_t, gid_t, mode_t };

type ShmSeg = c_ulong;

#[repr(C)]
struct XShmSegmentInfo {
	shmseg: ShmSeg,
	shmid: c_int,
	shmaddr: *mut c_char,
	read_only: Bool,
}

#[link(name = "Xext")]
extern "system" {
	fn XShmQueryExtension(display: *mut Display) -> Bool;
	fn XShmQueryVersion(display: *mut Display,
		major_ver: *mut c_int, minor_ver: *mut c_int,
		shared_pixmaps: *mut Bool) -> Bool;
	fn XShmCreateImage(display: *mut Display, visual: *mut Visual,
		depth: c_uint,
		format: c_int, data: *mut c_char,
		shminfo: *mut XShmSegmentInfo,
		width: c_uint, height: c_uint) -> *mut XImage;
	fn XShmAttach(display: *mut Display, shminfo: *mut XShmSegmentInfo) -> Bool;
	fn XShmGetImage(display: *mut Display, drawable: Drawable, image: *mut XImage,
		x: c_int, y: c_int,
		plane_mask: c_ulong) -> Bool;
	fn XShmDetach(display: *mut Display, shminfo: *mut XShmSegmentInfo) -> Bool;
}

type key_t = c_int;
type shmatt_t = c_ulong;

#[repr(C)]
struct ipc_perm {
	key: key_t,
	uid: uid_t,
	gid: gid_t,
	cuid: uid_t,
	cgid: gid_t,
	mode: mode_t,
	seq: c_ushort,
}

#[repr(C)]
struct shmid_ds {
	shm_perm: ipc_perm,
	shm_segsz: size_t,
	shm_atime: time_t,
	shm_dtime: time_t,
	shm_ctime: time_t,
	shm_cpid: pid_t,
	shm_lpid: pid_t,
	shm_nattch: shmatt_t,
	shm_unused2: *mut c_void,
	shm_unused3: *mut c_void,
}

const IPC_PRIVATE: key_t = 0;
const IPC_CREAT: c_int = 0o1000;
const IPC_RMID: c_int = 0;

extern "system" {
	fn shmget(key: key_t, size: size_t, shm_flag: c_int) -> c_int;
	fn shmat(shmid: c_int, shmaddr: *const c_void, shm_flag: c_int) -> *mut c_void;
	fn shmdt(shmaddr: *const c_void) -> c_int;
	fn shmctl(shmid: c_int, cmd: c_int, buf: *mut shmid_ds) -> c_int;
}

const XYBitmap: c_int = 0;
const XYPixmap: c_int = 1;
const ZPixmap: c_int = 2;

const AllPlanes: c_ulong = 0;

#[test]
fn test_create_window() {
	use x11::xlib;
	use std::ptr;

	unsafe {

	// use the information from the environment variable DISPLAY to create the X connection:	
	let display = match xlib::XOpenDisplay(ptr::null_mut()) {
		null if null.is_null() => panic!("Unable to connect X server"),
		display => display,
	};

	let screen_num = xlib::XDefaultScreen(display);
	let (black_pixel, white_pixel) = (xlib::XBlackPixel(display, screen_num),
		xlib::XWhitePixel(display, screen_num));

	// Once the display is initialized, create the window
	let (x, y) = (0, 0);
	let (width, height, border_width) = (400, 500, 5);
	let (border_color, background_color) = (black_pixel, white_pixel);
	let window = xlib::XCreateSimpleWindow(display,
		xlib::XRootWindow(display, screen_num),
		x, y,
		width, height, border_width,
		border_color, background_color);

	// this routine determines which types of input are allowed in the input.
	xlib::XSelectInput(display, window,
		xlib::ExposureMask | xlib::ButtonPressMask | xlib::KeyPressMask);

	// create the Graphics Context
	let graphics_context = xlib::XCreateGC(display, window, 0, ptr::null_mut());

	xlib::XMapWindow(display, window);
	xlib::XFlush(display);

	// std::thread::sleep_ms(1000);

	// it is good programming practice to return system resources to the system...
	xlib::XFreeGC(display, graphics_context);
	xlib::XDestroyWindow(display, window);
	xlib::XCloseDisplay(display);

	}
}

#[test]
fn test_shm() {
	use x11::xlib;
	use std::{ ptr, mem };

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
	let image = XShmCreateImage(display, xlib::XDefaultVisual(display, screen_num),
		xlib::XDefaultDepth(display, screen_num) as c_uint,
		ZPixmap, ptr::null_mut(),
		&mut shm_segment_info,
		root_width, root_height);
	if image.is_null() {
		panic!("XShmCreateImage returned null pointer");
	}

	shm_segment_info.shmid = shmget(IPC_PRIVATE,
		((*image).bytes_per_line * (*image).height) as size_t,
		IPC_CREAT | 0777);
	if shm_segment_info.shmid == -1 {
		panic!("shmget failed");
	}

	shm_segment_info.shmaddr = shmat(shm_segment_info.shmid, ptr::null_mut(), 0) as *mut c_char;
	if shm_segment_info.shmaddr.is_null() {
		panic!("XShmCreateImage returned null pointer");
	}
	(*image).data = shm_segment_info.shmaddr;

	shm_segment_info.read_only = 0;

	if XShmAttach(display, &mut shm_segment_info) == 0 {
		panic!("XShmAttach failed");
	}

	if XShmGetImage(display, root_window, image, root_x, root_y, AllPlanes) == 0 {
		panic!("XShmGetImage failed");
	}

	XShmDetach(display, &mut shm_segment_info);
	shmdt(shm_segment_info.shmaddr as *mut c_void);
	shmctl(shm_segment_info.shmid, IPC_RMID, ptr::null_mut());

	xlib::XCloseDisplay(display);

	}
}