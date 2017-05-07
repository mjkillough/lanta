use std;
use std::ffi;
use std::fmt;
use std::os::raw::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_void};
use std::ptr;
use std::slice;

use x11::xlib;

use debug;
use keys::{KeyCombo, KeyHandlers, ModKey};


/// A handle to an X Window.
#[derive(Debug, PartialEq)]
pub struct WindowId(xlib::Window);

impl WindowId {
    fn to_x(&self) -> xlib::Window {
        self.0
    }
}

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[allow(non_snake_case)]
struct InternedAtoms {
    WM_DELETE_WINDOW: xlib::Atom,
    WM_PROTOCOLS: xlib::Atom,
}

pub struct Connection {
    display: *mut xlib::Display,
    root: WindowId,
    atoms: InternedAtoms,
}

// TODO: Implement Drop so that we can call XCloseDisplay?

impl Connection {
    /// Opens a connection to the X server, returning a new Connection object.
    pub fn connect() -> Result<Connection, String> {
        let (display, root) = unsafe {
            let display: *mut xlib::Display = xlib::XOpenDisplay(ptr::null_mut());
            let root: xlib::Window = xlib::XDefaultRootWindow(display);
            (display, root)
        };

        if display.is_null() {
            return Err("XOpenDisplay returned null pointer".to_owned());
        }
        if root == 0 {
            return Err("XDefaultRootWindow returned 0".to_owned());
        }

        Ok(Connection {
               display: display,
               root: WindowId(root),
               atoms: InternedAtoms {
                   WM_PROTOCOLS: Self::intern_atom(display, "WM_PROTOCOLS"),
                   WM_DELETE_WINDOW: Self::intern_atom(display, "WM_DELETE_WINDOW"),
               },
           })
    }

    /// Returns the Atom identifier associated with the atom_name str.
    fn intern_atom(display: *mut xlib::Display, atom_name: &str) -> xlib::Atom {
        // Note: the CString is bound to a variable to ensure adequate lifetime.
        let cstring = ffi::CString::new(atom_name).unwrap();
        let ptr = cstring.as_ptr() as *const c_char;
        unsafe { xlib::XInternAtom(display, ptr, 0) }
    }

    /// Installs the Connection as a window manager, by registers for
    /// SubstructureNotify and SubstructureRedirect events on the root window.
    /// If there is already a window manager on the display, then this will
    /// fail.
    pub fn install_as_wm(&self, key_handlers: &KeyHandlers) -> Result<(), String> {
        unsafe {
            // It's tricky to get state from the error handler to here, so we install a
            // special handler while becoming the WM that panics on error.
            xlib::XSetErrorHandler(Some(debug::error_handler_init));
            xlib::XSelectInput(self.display,
                               self.root.to_x(),
                               xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask);
            xlib::XSync(self.display, 0);

            // If we get this far, then our error handler didn't panic. Set a more useful
            // error handler.
            xlib::XSetErrorHandler(Some(debug::error_handler));
        }

        self.enable_window_key_events(&self.root, key_handlers);

        Ok(())
    }

    /// Returns the ID of the root window.
    pub fn root_window_id(&self) -> &WindowId {
        &self.root
    }

    pub fn top_level_windows(&self) -> Vec<WindowId> {
        let mut root: xlib::Window = 0;
        let mut parent: xlib::Window = 0;
        let mut children: *mut xlib::Window = ptr::null_mut();
        let mut num_children: c_uint = 0;
        unsafe {
            xlib::XQueryTree(self.display,
                             self.root.to_x(),
                             &mut root,
                             &mut parent,
                             &mut children,
                             &mut num_children);
        }

        if children.is_null() || num_children == 0 {
            return vec![];
        }

        let slice = unsafe { slice::from_raw_parts(children, num_children as usize) };
        let vec: Vec<WindowId> = slice.iter().map(|id| WindowId(*id)).collect();

        unsafe {
            xlib::XFree(children as *mut c_void);
        }

        vec
    }

    /// Queries the WM_PROTOCOLS property of a window, returning a list of the
    /// protocols that it supports.
    // TODO: Have this return a list of atoms, rather than a list of strings.
    // (Perhaps we should
    // have a separate function to convert to a list of strings for debugging?)
    pub fn get_wm_protocols(&self, window_id: &WindowId) -> Vec<String> {
        let mut atoms: *mut c_ulong = ptr::null_mut();
        let mut count: c_int = 0;
        unsafe {
            xlib::XGetWMProtocols(self.display, window_id.to_x(), &mut atoms, &mut count);
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
            xlib::XGetAtomNames(self.display, atoms, count, pointers.as_mut_ptr());
            pointers.set_len(count as usize);
            pointers
                .iter()
                .map(|buffer| {
                         ffi::CStr::from_ptr(*buffer)
                             .to_str()
                             .unwrap()
                             .to_owned()
                     })
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

    // TODO: This, as it'd make the WM_DELETE_WINDOW code a little clearer.
    // pub fn supports_protocol(&self, window_id, WindowId, atom: Atom) -> bool;

    /// Closes a window.
    ///
    /// The window will be closed gracefully using the ICCCM WM_DELETE_WINDOW
    /// protocol if it is supported.
    pub fn close_window(&self, window_id: &WindowId) {
        let protocols = self.get_wm_protocols(window_id);
        let has_wm_delete_window = protocols.contains(&"WM_DELETE_WINDOW".to_owned());

        // TODO: Use XDestroyWindow to forcefully close windows that do not support
        // WM_DELETE_WINDOW.
        if !has_wm_delete_window {
            panic!("Not implemented: closing windows that don't expose WM_DELETE_WINDOW");
        }

        let mut client_message = xlib::XClientMessageEvent {
            type_: xlib::ClientMessage,
            serial: 0,
            send_event: 0,
            display: ptr::null_mut(),
            window: window_id.to_x(),
            message_type: self.atoms.WM_PROTOCOLS,
            format: 32,
            data: xlib::ClientMessageData::new(),
        };
        client_message
            .data
            .set_long(0, self.atoms.WM_DELETE_WINDOW as c_long);
        client_message
            .data
            .set_long(1, xlib::CurrentTime as c_long);
        let mut event = xlib::XEvent::from(client_message);
        unsafe {
            xlib::XSendEvent(self.display,
                             window_id.to_x(),
                             0,
                             xlib::NoEventMask,
                             &mut event);
        }
    }

    /// Sets the window's position and size.
    pub fn configure_window(&self, window_id: &WindowId, x: i32, y: i32, width: i32, height: i32) {
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
        let flags = xlib::CWX | xlib::CWY | xlib::CWWidth | xlib::CWHeight;

        unsafe {
            xlib::XConfigureWindow(self.display, window_id.to_x(), flags as u32, &mut changes);
        };
    }

    /// Get's the window's width and height.
    pub fn get_window_geometry(&self, window_id: &WindowId) -> (i32, i32) {
        unsafe {
            let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
            xlib::XGetWindowAttributes(self.display, window_id.to_x(), &mut attrs);

            (attrs.width, attrs.height)
        }
    }

    /// Map a window.
    pub fn map_window(&self, window_id: &WindowId) {
        unsafe {
            xlib::XMapWindow(self.display, window_id.to_x());
        }
    }

    /// Unmap a window.
    pub fn unmap_window(&self, window_id: &WindowId) {
        unsafe {
            xlib::XUnmapWindow(self.display, window_id.to_x());
        }
    }

    /// Registers for key events.
    pub fn enable_window_key_events(&self, window_id: &WindowId, key_handlers: &KeyHandlers) {
        unsafe {
            for key in key_handlers.key_combos() {
                let keycode = xlib::XKeysymToKeycode(self.display, key.keysym as u64) as i32;
                xlib::XGrabKey(self.display,
                               keycode,
                               key.mod_mask,
                               window_id.to_x(),
                               0,
                               xlib::GrabModeAsync,
                               xlib::GrabModeAsync);
            }
        }
    }

    pub fn enable_window_focus_tracking(&self, window_id: &WindowId) {
        unsafe {
            xlib::XSelectInput(self.display, window_id.to_x(), xlib::EnterWindowMask);
        }
    }

    pub fn disable_window_focus_tracking(&self, window_id: &WindowId) {
        unsafe {
            xlib::XSelectInput(self.display, window_id.to_x(), 0);
        }
    }

    pub fn focus_window(&self, window_id: &WindowId) {
        self.map_window(&window_id);
        unsafe {
            xlib::XSetInputFocus(self.display,
                                 window_id.to_x(),
                                 xlib::RevertToPointerRoot,
                                 xlib::CurrentTime);
            // TODO: _NET_ACTIVE_WINDOW
        }
    }

    pub fn get_event_loop(&self) -> EventLoop {
        EventLoop { connection: &self }
    }
}


/// Events received from the `EventLoop`.
pub enum Event {
    MapRequest(WindowId),
    DestroyNotify(WindowId),
    KeyPress(KeyCombo),
    EnterNotify(WindowId),
}


/// An iterator that yields events from the X event loop.
///
/// Use `Connection::get_event_loop()` to get one.
pub struct EventLoop<'a> {
    connection: &'a Connection,
}

impl<'a> Iterator for EventLoop<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            unsafe {
                let mut event = std::mem::zeroed();
                xlib::XNextEvent(self.connection.display, &mut event);

                let event = match event.get_type() {
                    xlib::ConfigureRequest => {
                        self.on_configure_request(xlib::XConfigureRequestEvent::from(event))
                    }
                    // TODO: move most of these handlers up to the window manager. The
                    // only one that out to stay here is the ConfigureRequest, as we're
                    // not going to do much with it!
                    xlib::MapRequest => self.on_map_request(xlib::XMapRequestEvent::from(event)),
                    xlib::DestroyNotify => {
                        self.on_destroy_notify(xlib::XDestroyWindowEvent::from(event))
                    }
                    xlib::KeyPress => self.on_key_press(xlib::XKeyPressedEvent::from(event)),
                    xlib::EnterNotify => self.on_enter_notify(xlib::XEnterWindowEvent::from(event)),
                    _ => None,
                };

                if let Some(event) = event {
                    return Some(event);
                }
            }
        }
    }
}

impl<'a> EventLoop<'a> {
    fn on_configure_request(&self, event: xlib::XConfigureRequestEvent) -> Option<Event> {
        // This request is not interesting for us: grant it unchanged.
        let mut changes = xlib::XWindowChanges {
            x: event.x,
            y: event.y,
            width: event.width,
            height: event.height,
            border_width: event.border_width,
            sibling: event.above,
            stack_mode: event.detail,
        };

        unsafe {
            xlib::XConfigureWindow(self.connection.display,
                                   event.window,
                                   event.value_mask as u32,
                                   &mut changes);
        }

        // There's no value in propogating this event.
        None
    }

    fn on_map_request(&self, event: xlib::XMapRequestEvent) -> Option<Event> {
        Some(Event::MapRequest(WindowId(event.window)))
    }

    fn on_destroy_notify(&self, event: xlib::XDestroyWindowEvent) -> Option<Event> {
        Some(Event::DestroyNotify(WindowId(event.window)))
    }

    fn on_key_press(&self, event: xlib::XKeyPressedEvent) -> Option<Event> {
        let mod_mask = event.state & ModKey::mask_all();
        let keysym = unsafe {
            xlib::XKeycodeToKeysym(self.connection.display, event.keycode as c_uchar, 0)
        } as c_uint;
        let key = KeyCombo {
            mod_mask: mod_mask,
            keysym: keysym,
        };

        Some(Event::KeyPress(key))
    }

    fn on_enter_notify(&self, event: xlib::XEnterWindowEvent) -> Option<Event> {
        Some(Event::EnterNotify(WindowId(event.window)))
    }
}
