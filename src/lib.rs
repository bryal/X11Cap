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
use x11::xlib;
use libc::c_int;
use std::ffi::CString;
use std::ptr::{self, Unique};
use std::slice;

pub mod ffi;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(C, packed)]
pub struct RGB8 {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    _pad: u8,
}

pub enum Display {
    Address(&'static str),
    Default,
}

#[derive(Clone, Copy)]
pub enum Screen {
    Specific(c_int),
    Default,
}

#[derive(Clone, Copy)]
pub enum Window {
    Window(xlib::Window),
    Desktop,
}

struct WindowConnection {
    display: Unique<xlib::Display>,
    window: xlib::Window,
    width: u32,
    height: u32,
}
impl WindowConnection {
    fn new(display: Display, screen: Screen, window: Window) -> Result<WindowConnection, ()> {
        let display_ptr = unsafe {
            Unique::new(xlib::XOpenDisplay(if let Display::Address(address) = display {
                match CString::new(address) {
                    Ok(s) => s.as_ptr(),
                    Err(_) => return Err(()),
                }
            } else {
                ptr::null()
            }))
        };

        if !display_ptr.is_null() {
            let screen_num = if let Screen::Specific(n) = screen {
                n
            } else {
                unsafe { xlib::XDefaultScreen(*display_ptr) }
            };

            let mut window_id = if let Window::Window(id) = window {
                id
            } else {
                unsafe { xlib::XRootWindow(*display_ptr, screen_num) }
            };

            let (mut window_width, mut window_height) = (0, 0);

            if unsafe {
                xlib::XGetGeometry(*display_ptr,
                                   window_id,
                                   &mut window_id,
                                   &mut 0,
                                   &mut 0,
                                   &mut window_width,
                                   &mut window_height,
                                   &mut 0,
                                   &mut 0) != 0
            } {
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
            xlib::XCloseDisplay(*self.display);
        }
    }
}

/// Possible errors when capturing
#[derive(Debug)]
pub enum CaptureError {
    Fail(&'static str),
}

pub struct Capturer {
    window_conn: WindowConnection,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}
impl Capturer {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Result<Capturer, ()> {
        match WindowConnection::new(Display::Default, Screen::Default, Window::Desktop) {
            Ok(conn) => {
                Ok(Capturer {
                    window_conn: conn,
                    width: width,
                    height: height,
                    x: x,
                    y: y,
                })
            }
            Err(_) => Err(()),
        }
    }

    pub fn capture_frame(&mut self) -> Result<(Vec<RGB8>, (u32, u32)), CaptureError> {
        let image_ptr = unsafe {
            xlib::XGetImage(*self.window_conn.display,
                            self.window_conn.window,
                            self.x,
                            self.y,
                            self.width,
                            self.height,
                            AllPlanes,
                            ZPixmap)
        };

        if !image_ptr.is_null() {
            let image = unsafe { &mut *image_ptr };

            unsafe {
                if image.depth == 24 && image.bits_per_pixel == 32 &&
                   image.red_mask == 0xFF0000 && image.green_mask == 0xFF00 &&
                   image.blue_mask == 0xFF {

                    let (width, height) = (image.width, image.height);

                    // It's plain (RGB8 + padding)s in memory
                    let raw_img_data = slice::from_raw_parts(image.data as *mut RGB8,
                                                             width as usize * height as usize)
                        .to_vec();

                    xlib::XDestroyImage(image_ptr);

                    Ok((raw_img_data, (width as u32, height as u32)))
                } else {
                    xlib::XDestroyImage(image_ptr as *mut _);

                    Err(CaptureError::Fail("WRONG LAYOUT"))
                }
            }
        } else {
            Err(CaptureError::Fail("XGetImage returned null pointer"))
        }
    }
}
