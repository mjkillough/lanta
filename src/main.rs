#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate x11;

use std::os::raw::{c_int, c_char};
use std::ptr;
use std::convert::From;

use x11::xlib;
use x11::xlib::{BadAccess, Display, XErrorEvent, XOpenDisplay, XDefaultRootWindow,
                XSetErrorHandler};


mod debug;
mod keys;
mod layout;
mod window;

use window::Window;
use layout::{Layout, TiledLayout};
use keys::Key;


fn key_handler() {
    info!("Key press!");
}


struct Config {
    keys: Vec<Key>,
    layout: Box<Layout>,
}


struct RustWindowManager {
    display: *mut Display,
    root: xlib::Window,

    config: Config,

    stack: Vec<Window>,
    // Focus is an index into the stack. We could do better and use the borrow checker to ensure
    // that it doesn't point at the wrong data when an item is added/removed from the stack.
    focus: Option<usize>
}

impl RustWindowManager {
    fn new(config: Config) -> Result<Self, String> {
        let (display, root) = unsafe {
            let display: *mut xlib::Display = xlib::XOpenDisplay(ptr::null_mut());
            if display.is_null() {
                return Err("XOpenDisplay() returned null".to_owned());
            }

            let root: xlib::Window = xlib::XDefaultRootWindow(display);
            if root == 0 {
                return Err("XDefaultRootWindow returned 0".to_owned());
            }

            (display, root)
        };

        // Install ourselves as the WM.
        unsafe {
            // It's tricky to get state from the error handler to here, so we install a special handler
            // while becoming the WM that panics on error.
            XSetErrorHandler(Some(debug::error_handler_init));
            xlib::XSelectInput(display,
                            root,
                            xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask);
            xlib::XSync(display, 0);

            // If we get this far, then our error handler didn't panic. Set a more useful error handler.
            XSetErrorHandler(Some(debug::error_handler));
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
            info!("Received event: {}", debug::xevent_to_str(&event));

            match event.get_type() {
                xlib::MapRequest => {
                    let event = xlib::XMapRequestEvent::from(event);
                    let mut window = Window {
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
