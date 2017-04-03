#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate x11;

use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_char, c_long, c_ulong, c_void};
use std::ptr;
use std::convert::From;
use std::rc::Rc;

use x11::xlib;

mod debug;
mod keys;
mod layout;
mod window;
mod x;

use window::Window;
use layout::{Layout, TiledLayout};
use keys::{Key, KeyHandler};
use x::{Connection, Event, WindowId};


struct Config {
    keys: Vec<KeyHandler>,
    layout: Box<Layout>,
}


pub struct RustWindowManager {
    connection: Rc<Connection>,

    config: Config,

    stack: Vec<Window>,
    // Focus is an index into the stack. We could do better and use the borrow checker to ensure
    // that it doesn't point at the wrong data when an item is added/removed from the stack.
    focus: Option<usize>,
}

impl RustWindowManager {
    fn new(config: Config) -> Result<Self, String> {
        let keys: Vec<Key> = config.keys
            .iter()
            .map(|kh| kh.key.clone())
            .collect();
        let connection = Connection::connect(keys)?;
        connection.install_as_wm()?;

        Ok(RustWindowManager {
               connection: Rc::new(connection),

               config: config,

               stack: Vec::new(),
               focus: None,
           })
    }

    fn layout(&mut self) {
        let root_window_id = self.connection.root_window_id();
        let (width, height) = self.connection.get_window_geometry(root_window_id);

        self.config.layout.layout(width, height, &mut self.stack);
    }

    fn get_focused(&self) -> Option<&Window> {
        self.focus.map(|i| &self.stack[i])
    }

    fn run_event_loop(&mut self) {
        let event_loop_connection = self.connection.clone();
        let mut event_loop = event_loop_connection.get_event_loop();
        for event in event_loop {
            match event {
                Event::MapRequest(window_id) => self.on_map_request(window_id),
                Event::DestroyNotify(window_id) => self.on_destroy_notify(window_id),
                Event::KeyPress(key) => self.on_key_press(key),
                Event::EnterNotify(window_id) => self.on_enter_notify(window_id),
            }
        }
        info!("Event loop exiting");
    }

    fn on_map_request(&mut self, window_id: WindowId) {
        self.connection.register_window_events(window_id);
        self.connection.map_window(window_id);

        self.stack.push(Window::new(self.connection.clone(), window_id));
        self.layout();
    }

    fn on_destroy_notify(&mut self, window_id: WindowId) {
        self.stack
            .iter()
            .position(|ref w| w.id == window_id)
            .map(|index| self.stack.remove(index));
        self.layout();
    }

    fn on_key_press(&self, key: Key) {
        self.config
            .keys
            .iter()
            .find(|kh| kh.key == key)
            .map(|kh| (kh.handler)(&self));
    }

    fn on_enter_notify(&mut self, window_id: WindowId) {
        self.focus = self.stack.iter().position(|ref w| w.id == window_id);
    }
}


fn close_window(wm: &RustWindowManager) {
    wm.get_focused().map(|w| w.close());
}


fn main() {
    env_logger::init().unwrap();

    let keys = vec![KeyHandler {
                        key: Key {
                            mod_mask: xlib::Mod4Mask,
                            keysym: x11::keysym::XK_T,
                        },
                        handler: Box::new(close_window),
                    }];
    let layout = Box::new(TiledLayout {});
    let config = Config {
        keys: keys,
        layout: layout,
    };

    let mut wm = RustWindowManager::new(config).unwrap();
    wm.run_event_loop();
}
