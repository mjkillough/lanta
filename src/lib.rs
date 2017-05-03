// #![deny(warnings)]

#[macro_use]
extern crate log;
extern crate libc;
extern crate x11;

use std::process::Command;
use std::rc::Rc;
use std::sync::Mutex;

mod debug;
pub mod groups;
pub mod keys;
pub mod layout;
mod stack;
pub mod window;
pub mod x;

use groups::{Group, GroupWindow};
use keys::{KeyCombo, KeyHandler, KeyHandlers, ModKey};
use window::Window;
use x::{Connection, Event, WindowId};


pub struct Config {
    pub keys: KeyHandlers,
}


pub struct RustWindowManager {
    connection: Rc<Connection>,

    config: Config,

    group: Group,
}

impl RustWindowManager {
    pub fn new(config: Config) -> Result<Self, String> {
        let connection = Connection::connect()?;
        connection.install_as_wm()?;
        let connection = Rc::new(connection);

        Ok(RustWindowManager {
               connection: connection.clone(),

               config: config,

               group: Group::new(connection.clone()),
           })
    }

    pub fn get_focused(&mut self) -> Option<GroupWindow> {
        self.group.get_focused()
    }

    pub fn focus_next(&mut self) {
        self.group.focus_next();
    }

    pub fn focus_previous(&mut self) {
        self.group.focus_previous();
    }

    pub fn shuffle_next(&mut self) {
        self.group.shuffle_next();
    }

    pub fn shuffle_previous(&mut self) {
        self.group.shuffle_previous();
    }

    pub fn run_event_loop(&mut self) {
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
        self.connection
            .register_window_events(&window_id, &self.config.keys);
        self.connection.map_window(&window_id);

        self.group.add_window(window_id);
    }

    fn on_destroy_notify(&mut self, window_id: WindowId) {
        self.group.remove_window(&window_id);
    }

    fn on_key_press(&mut self, key: KeyCombo) {
        self.config
            .keys
            .get(&key)
            .map(move |handler| (handler)(self));
    }

    fn on_enter_notify(&mut self, window_id: WindowId) {
        self.group.focus(&window_id);
    }
}


pub fn close_window(wm: &mut RustWindowManager) {
    wm.get_focused().map(|w| w.close());
}

pub fn focus_next(wm: &mut RustWindowManager) {
    wm.focus_next();
}

pub fn focus_previous(wm: &mut RustWindowManager) {
    wm.focus_previous();
}

pub fn shuffle_next(wm: &mut RustWindowManager) {
    wm.shuffle_next();
}

pub fn shuffle_previous(wm: &mut RustWindowManager) {
    wm.shuffle_previous();
}

pub fn spawn_command(command: Command) -> KeyHandler {
    let mutex = Mutex::new(command);
    Rc::new(move |wm| {
                let mut command = mutex.lock().unwrap();
                command.spawn().unwrap();
            })
}
