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

#![allow(dead_code, non_upper_case_globals)]

extern crate x11;
extern crate libc;

use ffi::*;
use libc::c_int;
use std::ffi::CString;
use std::{slice, ptr};
use x11::{xlib, xrandr};

pub mod ffi;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct Bgr8 {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    _pad: u8,
}

enum Display {
    Address(&'static str),
    Default,
}

#[derive(Clone, Copy)]
enum Screen {
    Specific(c_int),
    Default,
}

#[derive(Clone, Copy)]
enum Window {
    Window(xlib::Window),
    Desktop,
}

struct WindowConnection {
    display: *mut xlib::Display,
    window: xlib::Window,
    width: u32,
    height: u32,
}
impl WindowConnection {
    unsafe fn new(display: Display,
                  screen: Screen,
                  window: Window)
                  -> Result<WindowConnection, ()> {
        let addr = if let Display::Address(address) = display {
            match CString::new(address) {
                Ok(s) => s.as_ptr(),
                Err(_) => return Err(()),
            }
        } else {
            ptr::null()
        };

        let display_ptr = xlib::XOpenDisplay(addr);

        if !display_ptr.is_null() {
            let screen_num = if let Screen::Specific(n) = screen {
                n
            } else {
                xlib::XDefaultScreen(display_ptr)
            };

            let mut window_id = if let Window::Window(id) = window {
                id
            } else {
                xlib::XRootWindow(display_ptr, screen_num)
            };

            let (mut window_width, mut window_height) = (0, 0);

            let geo_result = xlib::XGetGeometry(display_ptr,
                                                window_id,
                                                &mut window_id,
                                                &mut 0,
                                                &mut 0,
                                                &mut window_width,
                                                &mut window_height,
                                                &mut 0,
                                                &mut 0);

            if geo_result != 0 {
                Ok(WindowConnection {
                    display: display_ptr,
                    window: window_id,
                    width: window_width,
                    height: window_height,
                })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}
impl Drop for WindowConnection {
    fn drop(&mut self) {
        unsafe {
            xlib::XCloseDisplay(self.display);
        }
    }
}

/// Possible errors when capturing
#[derive(Debug)]
pub enum CaptureError {
    Fail(&'static str),
}

/// Capture either a region of the total display area,
/// or capture the output of a specific monitor
pub enum CaptureSource {
    Region {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    Monitor(usize),
}

#[derive(Clone, Copy)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// A captured image
///
/// Wrapper that guarantees correct destruction
#[derive(Debug)]
pub struct Image {
    image: *mut xlib::XImage,
}

impl Image {
    pub fn as_slice(&self) -> &[Bgr8] {
        // It's plain (Bgr8 + padding)s in memory
        unsafe {
            slice::from_raw_parts((*self.image).data as *const _,
                                  (*self.image).width as usize * (*self.image).height as usize)
        }
    }

    pub fn get_dimensions(&self) -> (u32, u32) {
        unsafe { ((*self.image).width as u32, (*self.image).height as u32) }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            xlib::XDestroyImage(self.image);
        }
    }
}

pub struct Capturer {
    window_conn: WindowConnection,
    geo: Geometry,
}
impl Capturer {
    pub fn new(capture_src: CaptureSource) -> Result<Capturer, ()> {
        match unsafe { WindowConnection::new(Display::Default, Screen::Default, Window::Desktop) } {
            Ok(conn) => {
                let (x, y, w, h) = match capture_src {
                    CaptureSource::Region { x, y, width, height } => (x, y, width, height),
                    CaptureSource::Monitor(mon_i) => {
                        let mut n_mons = 0;
                        let mons = unsafe {
                            xrandr::XRRGetMonitors(conn.display, conn.window, 1, &mut n_mons)
                        };
                        let mons = unsafe { slice::from_raw_parts_mut(mons, n_mons as usize) };
                        let mon = mons[mon_i];
                        (mon.x, mon.y, mon.width as u32, mon.height as u32)
                    }
                };

                Ok(Capturer {
                    window_conn: conn,
                    geo: Geometry {
                        width: w,
                        height: h,
                        x: x,
                        y: y,
                    },
                })
            }
            Err(_) => Err(()),
        }
    }

    pub fn get_geometry(&self) -> Geometry {
        self.geo
    }

    pub fn capture_frame(&mut self) -> Result<Image, CaptureError> {
        let image_ptr = unsafe {
            xlib::XGetImage(self.window_conn.display,
                            self.window_conn.window,
                            self.geo.x,
                            self.geo.y,
                            self.geo.width,
                            self.geo.height,
                            AllPlanes,
                            ZPixmap)
        };

        if !image_ptr.is_null() {
            let image = unsafe { &mut *image_ptr };

            unsafe {
                if image.depth == 24 && image.bits_per_pixel == 32 &&
                   image.red_mask == 0xFF0000 && image.green_mask == 0xFF00 &&
                   image.blue_mask == 0xFF {

                    let image = Image { image: image_ptr };

                    Ok(image)
                } else {
                    xlib::XDestroyImage(image_ptr as *mut _);

                    Err(CaptureError::Fail("Wrong layout"))
                }
            }
        } else {
            Err(CaptureError::Fail("XGetImage returned null pointer"))
        }
    }
}
