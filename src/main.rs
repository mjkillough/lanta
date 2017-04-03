#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate x11;

use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_char, c_long, c_ulong, c_void};
use std::ptr;
use std::convert::From;
use std::rc::{Rc, Weak};
use std::slice::IterMut;

use x11::xlib;

mod debug;
mod keys;
mod layout;
mod window;
mod x;

use window::Window;
use layout::{Layout, TiledLayout};
use keys::{KeyCombo, KeyHandler, KeyHandlers, ModKey};
use x::{Connection, Event, WindowId};


struct Group {
    stack: Vec<Rc<Window>>,
    focus: Option<Weak<Window>>,
}

impl Group {
    fn new() -> Group {
        Group {
            stack: Vec::new(),
            focus: None,
        }
    }

    fn add_window(&mut self, window: Window) {
        self.stack.push(Rc::new(window));
    }

    fn find_window_by_id(&self, window_id: WindowId) -> Option<Rc<Window>> {
        self.stack
            .iter()
            .find(|w| w.id == window_id)
            .map(|rc| rc.clone())
    }

    fn remove_window(&mut self, window: &Window) {
        self.stack
            .iter()
            .position(|w| w.id == window.id)
            .map(|i| self.stack.remove(i));
    }

    fn focus(&mut self, window: Rc<Window>) {
        self.focus = Some(Rc::downgrade(&window));
    }

    fn unfocus(&mut self) {
        self.focus = None;
    }

    fn get_focused(&self) -> Option<Rc<Window>> {
        self.focus.clone().and_then(|rc| rc.upgrade())
    }

    fn iter_mut(&mut self) -> IterMut<Rc<Window>> {
        self.stack.iter_mut()
    }
}


struct Config {
    layout: Box<Layout>,
    keys: KeyHandlers,
}


pub struct RustWindowManager {
    connection: Rc<Connection>,

    config: Config,

    group: Group,
}

impl RustWindowManager {
    fn new(config: Config) -> Result<Self, String> {
        let connection = Connection::connect()?;
        connection.install_as_wm()?;

        Ok(RustWindowManager {
               connection: Rc::new(connection),

               config: config,

               group: Group::new(),
           })
    }

    fn layout(&mut self) {
        let root_window_id = self.connection.root_window_id();
        let (width, height) = self.connection.get_window_geometry(root_window_id);

        self.config.layout.layout(width, height, self.group.iter_mut());
    }

    fn get_focused(&self) -> Option<Rc<Window>> {
        self.group.get_focused()
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
        self.connection.register_window_events(window_id, &self.config.keys);
        self.connection.map_window(window_id);

        let window = Window::new(self.connection.clone(), window_id);
        self.group.add_window(window);
        self.layout();
    }

    fn on_destroy_notify(&mut self, window_id: WindowId) {
        self.group.find_window_by_id(window_id)
            .map(|w| self.group.remove_window(&w));
        self.layout();
    }

    fn on_key_press(&self, key: KeyCombo) {
        self.config.keys.dispatch(&key, &self);
    }

    fn on_enter_notify(&mut self, window_id: WindowId) {
        self.group.find_window_by_id(window_id)
            .map(|w| self.group.focus(w));
    }
}


fn close_window(wm: &RustWindowManager) {
    wm.get_focused().map(|w| w.close());
}


fn main() {
    env_logger::init().unwrap();

    let keys = KeyHandlers::new(vec![(KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_t),
                                      Box::new(close_window))]);

    let layout = Box::new(TiledLayout {});
    let config = Config {
        keys: keys,
        layout: layout,
    };

    let mut wm = RustWindowManager::new(config).unwrap();
    wm.run_event_loop();
}
