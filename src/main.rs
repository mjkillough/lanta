#![deny(warnings)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate x11;

use std::rc::Rc;

mod debug;
mod groups;
mod keys;
mod layout;
mod window;
mod x;

use layout::{Layout, TiledLayout};
use keys::{KeyCombo, KeyHandlers, ModKey};
use x::{Connection, Event, WindowId};
use window::Window;
use groups::{Group, GroupWindow};


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
        let connection = Rc::new(connection);

        Ok(RustWindowManager {
               connection: connection.clone(),

               config: config,

               group: Group::new(connection.clone()),
           })
    }

    fn layout(&mut self) {
        let root_window_id = self.connection.root_window_id();
        let (width, height) = self.connection.get_window_geometry(&root_window_id);

        self.config.layout.layout(width, height, self.group.iter_mut());
    }

    fn get_focused(&mut self) -> Option<GroupWindow> {
        self.group.get_focused()
    }

    fn focus_next(&mut self) {
        self.group.focus_next();
    }

    fn focus_previous(&mut self) {
        self.group.focus_previous();
    }

    fn run_event_loop(&mut self) {
        let event_loop_connection = self.connection.clone();
        let event_loop = event_loop_connection.get_event_loop();
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
        self.connection.register_window_events(&window_id, &self.config.keys);
        self.connection.map_window(&window_id);

        self.group.add_window(window_id);
        self.layout();
    }

    fn on_destroy_notify(&mut self, window_id: WindowId) {
        self.group.find_window_by_id(&window_id).map(|w| w.remove_from_group());
        self.layout();
    }

    fn on_key_press(&mut self, key: KeyCombo) {
        self.config
            .keys
            .get(&key)
            .map(move |handler| (handler)(self));
    }

    fn on_enter_notify(&mut self, window_id: WindowId) {
        self.group.find_window_by_id(&window_id).map(|mut w| w.focus());
    }
}


fn close_window(wm: &mut RustWindowManager) {
    wm.get_focused().map(|w| w.close());
}

fn focus_next(wm: &mut RustWindowManager) {
    wm.focus_next();
}

fn focus_previous(wm: &mut RustWindowManager) {
    wm.focus_previous();
}

fn main() {
    env_logger::init().unwrap();

    let keys = KeyHandlers::new(vec![(KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_t),
                                      Rc::new(close_window)),
                                     (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_y),
                                      Rc::new(focus_next)),
                                     (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_u),
                                      Rc::new(focus_previous))]);

    let layout = Box::new(TiledLayout {});
    let config = Config {
        keys: keys,
        layout: layout,
    };

    let mut wm = RustWindowManager::new(config).unwrap();
    wm.run_event_loop();
}
