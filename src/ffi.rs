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

//! FFI bindings for functions related to SHM, shared memory

use x11::xlib::{ Display, Bool, Visual, XImage, Drawable };
use libc::{ c_int, c_char, c_uint, c_ushort, size_t, c_void, c_ulong, time_t, pid_t, uid_t, gid_t, mode_t };

pub type ShmSeg = c_ulong;

#[repr(C)]
pub struct XShmSegmentInfo {
	pub shmseg: ShmSeg,
	pub shmid: c_int,
	pub shmaddr: *mut c_char,
	pub read_only: Bool,
}

#[link(name = "Xext")]
extern "system" {
	pub fn XShmQueryExtension(display: *mut Display) -> Bool;
	pub fn XShmQueryVersion(display: *mut Display,
		major_ver: *mut c_int, minor_ver: *mut c_int,
		shared_pixmaps: *mut Bool) -> Bool;
	pub fn XShmCreateImage(display: *mut Display, visual: *mut Visual,
		depth: c_uint,
		format: c_int, data: *mut c_char,
		shminfo: *mut XShmSegmentInfo,
		width: c_uint, height: c_uint) -> *mut XImage;
	pub fn XShmAttach(display: *mut Display, shminfo: *mut XShmSegmentInfo) -> Bool;
	pub fn XShmGetImage(display: *mut Display, drawable: Drawable, image: *mut XImage,
		x: c_int, y: c_int,
		plane_mask: c_ulong) -> Bool;
	pub fn XShmDetach(display: *mut Display, shminfo: *mut XShmSegmentInfo) -> Bool;
	fn XShmPixmapFormat(display: *mut Display) -> c_int;
}

pub type key_t = c_int;
pub type shmatt_t = c_ulong;

#[repr(C)]
pub struct ipc_perm {
	pub key: key_t,
	pub uid: uid_t,
	pub gid: gid_t,
	pub cuid: uid_t,
	pub cgid: gid_t,
	pub mode: mode_t,
	pub seq: c_ushort,
}

#[repr(C)]
pub struct shmid_ds {
	pub shm_perm: ipc_perm,
	pub shm_segsz: size_t,
	pub shm_atime: time_t,
	pub shm_dtime: time_t,
	pub shm_ctime: time_t,
	pub shm_cpid: pid_t,
	pub shm_lpid: pid_t,
	pub shm_nattch: shmatt_t,
	shm_unused2: *mut c_void,
	shm_unused3: *mut c_void,
}

pub const IPC_PRIVATE: key_t = 0;
pub const IPC_CREAT: c_int = 0o1000;
pub const IPC_RMID: c_int = 0;

extern "system" {
	pub fn shmget(key: key_t, size: size_t, shm_flag: c_int) -> c_int;
	pub fn shmat(shmid: c_int, shmaddr: *const c_void, shm_flag: c_int) -> *mut c_void;
	pub fn shmdt(shmaddr: *const c_void) -> c_int;
	pub fn shmctl(shmid: c_int, cmd: c_int, buf: *mut shmid_ds) -> c_int;
}

pub const XYBitmap: c_int = 0;
pub const XYPixmap: c_int = 1;
pub const ZPixmap: c_int = 2;

pub const AllPlanes: c_ulong = 0;