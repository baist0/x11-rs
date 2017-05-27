// x11-rs: Rust bindings for X11 libraries
// The X11 libraries are available under the MIT license.
// These bindings are public domain.
//
// Stas Baibak created this example
//

use std::os::raw::{ c_char, c_int, c_uint, c_ulong };
use xlib::{ Bool, Display, Drawable, GC, Pixmap, Visual, XImage };

//
// functions
//

x11_link! { Xext, xext, ["libXext.so.6", "libXext.so"], 10,
  pub fn XShmQueryExtension (_1: *mut Display) -> Bool,
  pub fn XShmGetEventBase (_1: *mut Display) -> c_int,
  pub fn XShmQueryVersion (_4: *mut Display, _3: *mut c_int, _2: *mut c_int, _1: *mut Bool) -> Bool,
  pub fn XShmPixmapFormat (_1: *mut Display) -> c_int,
  pub fn XShmGetImage (_6: *mut Display, _5: Drawable, _4: *mut XImage, _3: c_int, _2: c_int, _1: c_ulong) -> Bool,
  pub fn XShmAttach (_2: *mut Display, _1: *mut XShmSegmentInfo) -> c_int,
  pub fn XShmDetach (_2: *mut Display, _1: *mut XShmSegmentInfo) -> c_int,
  // dpy, d, data, shminfo, width, height, depth
  pub fn XShmCreatePixmap(_7: *mut Display, _6: Drawable, _5: *mut c_char, _4: *mut XShmSegmentInfo, _3: c_uint, _2: c_uint, _1: c_uint) -> Pixmap,
  // spy, visual, depth, format, data, shminfo, width, height
  pub fn XShmCreateImage(_8: *mut Display, _7: *mut Visual, _6: c_uint, _5: c_int, _4: *mut c_char, _3: *mut XShmSegmentInfo, _2: c_uint, _1: c_uint) -> *mut XImage,
  // dpy, d, gc, image, src_x, src_y, dst_x, dst_y, src_width, src_height, send_event
  pub fn XShmPutImage(_11: *mut Display, _10: Drawable, _9: GC, _8: *mut XImage, _7: c_int, _6: c_int, _5: c_int, _4: c_int, _3: c_uint, _2: c_uint, _1: c_int) -> c_int,
variadic:
globals:
}

//
// types
//

pub type ShmSeg = c_ulong;

//
// structures
//

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XShmCompletionEvent {
  pub _type: c_int,           // of event
  pub serial: c_ulong,        // # of last request processed by server
  pub send_event: Bool,       // true if this came frome a SendEvent request
  pub display: *mut Display,  // Display the event was read from
  pub drawable: Drawable,     // drawable of request
  pub major_code: c_int,      // ShmReqCode
  pub minor_code: c_int,      // X_ShmPutImage
  pub shmseg: ShmSeg,         // the ShmSeg used in the request
  pub offset: c_ulong         // the offset into ShmSeg used in the request
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XShmSegmentInfo {
  pub shmseg: ShmSeg,        // resource id
  pub shmid: c_int,          // kernel id
  pub shmaddr: *mut c_char,  // address in client
  pub readOnly: Bool         // how the server should attach it
}
