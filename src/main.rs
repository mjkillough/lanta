#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate x11;

use std::ffi::CString;
use std::os::raw::{c_int, c_char};
use std::ptr;
use std::convert::From;

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


fn xevent_to_str(event: &xlib::XEvent) -> &str {
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

unsafe fn layout(disp: *mut Display, attrs: &xlib::XWindowAttributes, stack: &Vec<Window>) {
    if stack.len() == 0 {
        return;
    }

    let height = attrs.height / stack.len() as i32;

    for (i, window) in stack.iter().enumerate() {
        let i = i as i32;
        let mut changes: xlib::XWindowChanges = std::mem::zeroed();
        changes.x = 0;
        changes.y = i * height;
        changes.width = attrs.width;
        changes.height = height;
        let flags = xlib::CWX | xlib::CWY | xlib::CWWidth | xlib::CWHeight;
        xlib::XConfigureWindow(disp,
                               *window,
                               flags as u32,
                               &mut changes);
    }
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


        let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
        xlib::XGetWindowAttributes(disp, root, &mut attrs);
        info!("Root window has geometry: {}x{}", attrs.width, attrs.height);

        let mut stack: Vec<Window> = Vec::new();

        loop {
            info!("Getting event...");
            let mut event = xlib::XEvent { pad: [0; 24] };
            xlib::XNextEvent(disp, &mut event);
            info!("Received event: {}", xevent_to_str(&event));

            match event.get_type() {
                xlib::MapRequest => {
                    let event = xlib::XMapRequestEvent::from(event);

                    stack.push(event.window);

                    layout(disp, &attrs, &stack);

                    xlib::XMapWindow(disp, event.window);
                }

                xlib::DestroyNotify => {
                    let event = xlib::XDestroyWindowEvent::from(event);

                    stack.iter().position(|&w| w == event.window).map(|index| stack.remove(index));

                    layout(disp, &attrs, &stack);
                }
                _ => {}
            }

            println!("Stack: {:?}", stack);
        }

    };
    println!("Hello, world!");
}
