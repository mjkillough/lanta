use x11::xlib;

use keys::Key;


#[derive(Debug)]
pub struct Window {
    // TODO: Consider making these private.
    pub display: *mut xlib::Display,
    pub xwindow: xlib::Window,
}


impl Window {
    // Does this really need to be &mut self? It feels light it ought to be as it is actually
    // modifying the underlying window, even if we're not actually modifying the value as far as
    // Rust is concerned.
    pub fn position(&mut self, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
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

    pub fn grab_keys(&mut self, keys: &[Key]) {
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
