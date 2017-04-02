use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use x11::xlib;


// Error handler used during setup, which simply checks for the BadAccess error
// which indicates that another WM is already running.
pub unsafe extern "C" fn error_handler_init(display: *mut xlib::Display, err: *mut xlib::XErrorEvent) -> c_int {
    if (*err).error_code == xlib::BadAccess {
        panic!("Another WM is already running");
    }
    0
}


// Actual error handler used during normal operation.
pub unsafe extern "C" fn error_handler(display: *mut xlib::Display, err: *mut xlib::XErrorEvent) -> c_int {
    let buffer_size: usize = 1024;
    let mut buffer = Vec::<u8>::with_capacity(buffer_size);
    // XXX the docs say this returns the error text in the 'current locale'. We're
    // being extremely naughty and assuming this is UTF-8.
    // We're also assuming the return value of XGetErrorText is the actual length
    // of the string.
    let len: c_int = xlib::XGetErrorText(display,
                                         (*err).error_code as i32,
                                         buffer.as_mut_ptr() as *mut c_char,
                                         buffer_size as i32);
    buffer.truncate(len as usize);
    let error_text = CString::new(buffer).unwrap().into_string().unwrap();
    error!("Received X error: request={}, error_code=({}, {}), resource_id={}",
           (*err).request_code,
           (*err).error_code,
           error_text,
           (*err).resourceid);
    0
}


pub fn xevent_to_str(event: &xlib::XEvent) -> &str {
    match event.get_type() {
        2 => "KeyPress",
        3 => "KeyRelease",
        4 => "ButtonPress",
        5 => "ButtonRelease",
        6 => "MotionNotify",
        7 => "EnterNotify",
        8 => "LeaveNotify",
        9 => "FocusIn",
        10 => "FocusOut",
        11 => "KeymapNotify",
        12 => "Expose",
        13 => "GraphicsExpose",
        14 => "NoExpose",
        15 => "VisibilityNotify",
        16 => "CreateNotify",
        17 => "DestroyNotify",
        18 => "UnmapNotify",
        19 => "MapNotify",
        20 => "MapRequest",
        21 => "ReparentNotify",
        22 => "ConfigureNotify",
        23 => "ConfigureRequest",
        24 => "GravityNotify",
        25 => "ResizeRequest",
        26 => "CirculateNotify",
        27 => "CirculateRequest",
        28 => "PropertyNotify",
        29 => "SelectionClear",
        30 => "SelectionRequest",
        31 => "SelectionNotify",
        32 => "ColormapNotify",
        33 => "ClientMessage",
        34 => "MappingNotify",
        35 => "GenericEvent",
        36 => "LASTEvent",
        _ => {
            error!("Unknown XEvent type: {}", event.get_type());
            "Unknown"
        }
    }
}
