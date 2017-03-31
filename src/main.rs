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


#[derive(Debug)]
struct Win {
    display: *mut Display,
    xwindow: Window,
}


impl Win {
    // Does this really need to be &mut self? It feels light it ought to be as it is actually
    // modifying the underlying window, even if we're not actually modifying the value as far as
    // Rust is concerned.
    fn position(&mut self, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
        let mut changes = xlib::XWindowChanges {
            x: x,
            y: y,
            width: width,
            height: height,

            // Ignored:
            border_width: 0,
            sibling: 0,
            stack_mode: 0,
        };
        let flags = (xlib::CWX | xlib::CWY | xlib::CWWidth | xlib::CWHeight) as u32;

        unsafe {
            xlib::XConfigureWindow(self.display, self.xwindow, flags, &mut changes);
        };

        Ok(())
    }

    fn grab_keys(&mut self, keys: &[Key]) {
        for key in keys.iter() {
            unsafe {
                let keycode = xlib::XKeysymToKeycode(self.display, key.keysym as u64) as i32;
                xlib::XGrabKey(self.display,
                               keycode,
                               key.mod_mask,
                               self.xwindow,
                               0,
                               xlib::GrabModeAsync,
                               xlib::GrabModeAsync);
            }
        }
    }
}


trait Layout {
    fn layout(&self, width: i32, height: i32, stack: &mut [Win]) -> Result<(), String>;
}


struct TiledLayout;

impl Layout for TiledLayout {
    fn layout(&self, width: i32, height: i32, stack: &mut [Win]) -> Result<(), String> {
        if stack.len() == 0 {
            return Ok(());
        }

        let tile_height = height / stack.len() as i32;

        for (i, window) in stack.iter_mut().enumerate() {
            window.position(0, i as i32 * tile_height, width, tile_height)?;
        }

        Ok(())
    }
}



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


fn key_handler() {
    info!("Key press!");
}


struct Key {
    mod_mask: std::os::raw::c_uint,
    keysym: std::os::raw::c_uint,
    handler: Box<Fn()>,
}


impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
               "Key {{ mod_mask: {}, keysym: {} }}",
               self.mod_mask,
               self.keysym)
    }
}


struct Config {
    keys: Vec<Key>,
    layout: Box<Layout>,
}


struct RustWindowManager {
    display: *mut Display,
    root: Window, // Could be Win?

    config: Config,

    stack: Vec<Win>,
    // Focus is an index into the stack. We could do better and use the borrow checker to ensure
    // that it doesn't point at the wrong data when an item is added/removed from the stack.
    focus: Option<usize>
}

impl RustWindowManager {
    fn new(config: Config) -> Result<Self, String> {
        let (display, root) = unsafe {
            let display: *mut Display = XOpenDisplay(ptr::null_mut());
            if display.is_null() {
                return Err("XOpenDisplay() returned null".to_owned());
            }

            let root: Window = XDefaultRootWindow(display);
            if root == 0 {
                return Err("XDefaultRootWindow returned 0".to_owned());
            }

            (display, root)
        };

        // Install ourselves as the WM.
        unsafe {
            // It's tricky to get state from the error handler to here, so we install a special handler
            // while becoming the WM that panics on error.
            XSetErrorHandler(Some(error_handler_init));
            xlib::XSelectInput(display,
                            root,
                            xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask);
            xlib::XSync(display, 0);

            // If we get this far, then our error handler didn't panic. Set a more useful error handler.
            XSetErrorHandler(Some(error_handler));
        }

        Ok(RustWindowManager {
            display: display,
            root: root,

            config: config,

            stack: Vec::new(),
            focus: None,
        })
    }

    fn layout(&mut self) -> Result<(), String> {
        let (width, height) = unsafe {
            let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
            xlib::XGetWindowAttributes(self.display, self.root, &mut attrs);
            info!("Root window has geometry: {}x{}", attrs.width, attrs.height);
            (attrs.width, attrs.height)
        };

        self.config.layout.layout(width, height, &mut self.stack)
    }
}


fn main() {
    env_logger::init().unwrap();

    let keys = vec![Key {
                        mod_mask: xlib::Mod4Mask,
                        keysym: x11::keysym::XK_T,
                        handler: Box::new(key_handler),
                    }];
    let layout = Box::new(TiledLayout {});
    let config = Config { keys: keys, layout: layout };

    let mut wm = RustWindowManager::new(config).unwrap();

    unsafe {
        loop {
            let mut event = xlib::XEvent { pad: [0; 24] };
            xlib::XNextEvent(wm.display, &mut event);
            info!("Received event: {}", xevent_to_str(&event));

            match event.get_type() {
                xlib::MapRequest => {
                    let event = xlib::XMapRequestEvent::from(event);
                    let mut window = Win {
                        display: wm.display,
                        xwindow: event.window,
                    };
                    window.grab_keys(&wm.config.keys);

                    xlib::XSelectInput(wm.display, window.xwindow, xlib::EnterWindowMask);

                    wm.stack.push(window);

                    wm.layout();

                    // TODO: Make this into a method on Win?
                    xlib::XMapWindow(wm.display, event.window);

                }

                xlib::DestroyNotify => {
                    let event = xlib::XDestroyWindowEvent::from(event);

                    wm.stack.iter().position(|ref w| w.xwindow == event.window).map(|index| wm.stack.remove(index));

                    wm.layout();
                }

                xlib::KeyPress => {
                    let event = xlib::XKeyEvent::from(event);

                    for key in wm.config.keys.iter() {
                        let keycode = xlib::XKeysymToKeycode(wm.display, key.keysym as u64) as u32;
                        debug!("KeyPress: state={}, keycode={}", event.state, event.keycode);

                        // TODO: Allow extra mod keys to be pressed at the same time. (Add test!)
                        if (event.state & key.mod_mask != 0) && event.keycode == keycode {
                            info!("KeyPress matches key: {}", key);
                            (key.handler)();
                            break;
                        }
                    }

                }

                xlib::EnterNotify => {
                    let event = xlib::XEnterWindowEvent::from(event);

                    wm.focus = wm.stack.iter().position(|ref w| w.xwindow == event.window);
                    debug!("EnterNotify: {:?}", wm.focus);
                }

                _ => {}
            }

            println!("Stack: {:?}", wm.stack);
        }

    };
    println!("Hello, world!");
}
