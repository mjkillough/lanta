#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate x11;

use std::ffi::{ CStr, CString };
use std::os::raw::{c_int, c_char, c_long, c_ulong, c_void};
use std::ptr;
use std::convert::From;

use x11::xlib;

mod debug;
mod keys;
mod layout;
mod window;

use window::Window;
use layout::{Layout, TiledLayout};
use keys::Key;


struct Config {
    keys: Vec<Key>,
    layout: Box<Layout>,
}


pub struct RustWindowManager {
    display: *mut xlib::Display,
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
            xlib::XSetErrorHandler(Some(debug::error_handler_init));
            xlib::XSelectInput(display,
                            root,
                            xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask);
            xlib::XSync(display, 0);

            // If we get this far, then our error handler didn't panic. Set a more useful error handler.
            xlib::XSetErrorHandler(Some(debug::error_handler));
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

    fn get_focused(&self) -> Option<&Window> {
        self.focus.map(|i| &self.stack[i])
    }
}


fn get_wm_protocols(window: &Window) -> Vec<String> {
    let mut atoms: *mut c_ulong = ptr::null_mut();
    let mut count: c_int = 0;
    unsafe {
        xlib::XGetWMProtocols(window.display, window.xwindow, &mut atoms, &mut count);
    }

    if atoms.is_null() {
        error!("XGetWMProtocols returned null pointer");
        return Vec::new();
    }
    if count == 0 {
        return Vec::new();
    }

    let mut pointers: Vec<*mut c_char> = Vec::with_capacity(count as usize);
    let protocols: Vec<String> = unsafe {
        xlib::XGetAtomNames(window.display, atoms, count, pointers.as_mut_ptr());
        pointers.set_len(count as usize);
        pointers.iter()
            .map(|buffer| CStr::from_ptr(*buffer).to_str().unwrap().to_owned())
            .collect()
    };

    unsafe {
        for pointer in pointers.iter() {
            xlib::XFree(*pointer as *mut c_void);
        }
        xlib::XFree(atoms as *mut c_void);
    }

    protocols
}


fn intern_atom(display: *mut xlib::Display, s: &str) -> xlib::Atom {
    // Note: it is important that the CString is bound to a variable to ensure adequate lifetime.
    let cstring = CString::new(s).unwrap();
    let ptr = cstring.as_ptr() as *const c_char;
    unsafe {
        xlib::XInternAtom(display, ptr, 0)
    }
}


fn close_window(wm: &RustWindowManager) {
    if let Some(window) = wm.get_focused() {
        // Does it support WM_DELETE_WINDOW ICCCM protocol?
        // TODO: Avoid converting atoms to strings.
        let protocols = get_wm_protocols(window);
        let has_wm_delete_window = protocols.contains(&"WM_DELETE_WINDOW".to_owned());

        if !has_wm_delete_window {
            error!("Could not close window as it doesn't support WM_DELETE_WINDOW");
        }

        let ATOM_WM_PROTOCOLS = intern_atom(wm.display, "WM_PROTOCOLS");
        let ATOM_WM_DELETE_WINDOW = intern_atom(wm.display, "WM_DELETE_WINDOW");
        debug!("Atoms: {} {}", ATOM_WM_PROTOCOLS, ATOM_WM_DELETE_WINDOW);

        info!("Sending WM_DELETE_WINDOW to {}", window.xwindow);
        let mut client_message = xlib::XClientMessageEvent {
            type_: xlib::ClientMessage,
            serial: 0,
            send_event: 0,
            display: ptr::null_mut(),
            window: window.xwindow,
            message_type: ATOM_WM_PROTOCOLS,
            format: 32,
            data: xlib::ClientMessageData::new(),
        };
        client_message.data.set_long(0, ATOM_WM_DELETE_WINDOW as c_long);
        client_message.data.set_long(1, xlib::CurrentTime as c_long);
        let mut event = xlib::XEvent::from(client_message);
        unsafe {
            xlib::XSendEvent(wm.display, window.xwindow, 0, xlib::NoEventMask, &mut event);
        }
        info!("Sent WM_DELETE_WINDOW to {}", window.xwindow);
    }
}


fn main() {
    env_logger::init().unwrap();

    let keys = vec![Key {
                        mod_mask: xlib::Mod4Mask,
                        keysym: x11::keysym::XK_T,
                        handler: Box::new(close_window),
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
                // Pass through ConfigureRequests unchanged.
                xlib::ConfigureRequest => {
                    let event = xlib::XConfigureRequestEvent::from(event);

                    let mut changes = xlib::XWindowChanges {
                        x: event.x,
                        y: event.y,
                        width: event.width,
                        height: event.height,
                        border_width: event.border_width,
                        sibling: event.above,
                        stack_mode: event.detail,
                    };
                    xlib::XConfigureWindow(wm.display, event.window, event.value_mask as u32, &mut changes);
                }

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
                            (key.handler)(&wm);
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
