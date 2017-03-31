#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate x11;

use std::ffi::CString;
use std::os::raw::{c_int, c_char};
use std::ptr;

use x11::xlib;
use x11::xlib::{BadAccess, Display, Window, XErrorEvent, XOpenDisplay, XDefaultRootWindow,
                XSetErrorHandler};


// Error handler used during setup, which simply checks for the BadAccess error
// which indicates that another WM is already running.
unsafe extern "C" fn error_handler_init(disp: *mut Display, err: *mut XErrorEvent) -> c_int {
    if (*err).error_code == BadAccess {
        panic!("Another WM is already running");
    }
    0
}

// Actual error handler used during normal operation.
unsafe extern "C" fn error_handler(disp: *mut Display, err: *mut XErrorEvent) -> c_int {
    let buffer_size: usize = 1024;
    let mut buffer = Vec::<u8>::with_capacity(buffer_size);
    // XXX the docs say this returns the error text in the 'current locale'. We're
    // being extremely naughty and assuming this is UTF-8.
    // We're also assuming the return value of XGetErrorText is the actual length
    // of the string.
    let len: c_int = xlib::XGetErrorText(disp,
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


fn main() {
    env_logger::init().unwrap();

    unsafe {
        let disp: *mut Display = XOpenDisplay(ptr::null_mut());
        assert!(!disp.is_null());
        let root: Window = XDefaultRootWindow(disp);
        assert!(root != 0);

        XSetErrorHandler(Some(error_handler_init));
        xlib::XSelectInput(disp,
                           root,
                           xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask);
        xlib::XSync(disp, 0);

        // If we get this far, then our panicing error handler didn't complain!
        info!("We are now the WM");

        XSetErrorHandler(Some(error_handler));

    };
    println!("Hello, world!");
}
