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
use stack::Stack;
use window::Window;
use x::{Connection, Event, WindowId};


pub struct Config {
    pub keys: KeyHandlers,
}


pub struct RustWindowManager {
    connection: Rc<Connection>,

    config: Config,

    groups: Stack<Group>,
}

impl RustWindowManager {
    pub fn new(config: Config) -> Result<Self, String> {
        let connection = Connection::connect()?;
        connection.install_as_wm(&config.keys)?;
        let connection = Rc::new(connection);

        let mut groups = Stack::new();
        groups.push(Group::new("g1", connection.clone()));
        groups.push(Group::new("g2", connection.clone()));

        Ok(RustWindowManager {
               connection: connection.clone(),

               config: config,

               groups: groups,
           })
    }

    pub fn group(&self) -> &Group {
        self.groups.focused().expect("No active group!")
    }

    pub fn group_mut(&mut self) -> &mut Group {
        self.groups.focused_mut().expect("No active group!")
    }

    pub fn switch_group<'a, S>(&'a mut self, name: S)
        where S: Into<&'a str>
    {
        let name = name.into();
        self.group_mut().deactivate();
        self.groups.focus(|group| group.name() == name);
        self.group_mut().activate();
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
            .enable_window_key_events(&window_id, &self.config.keys);
        self.connection.enable_window_focus_tracking(&window_id);
        self.connection.map_window(&window_id);

        self.group_mut().add_window(window_id);
    }

    fn on_destroy_notify(&mut self, window_id: WindowId) {
        // Remove the window from whichever Group it is in.
        let group_opt = self.groups
            .iter_mut()
            .find(|group| group.contains(&window_id));
        match group_opt {
            Some(group) => {
                group.remove_window(&window_id);
            }
            None => {
                error!("on_destroy_notify: window {} is not in any group",
                       window_id)
            }
        }
    }

    fn on_key_press(&mut self, key: KeyCombo) {
        self.config
            .keys
            .get(&key)
            .map(move |handler| (handler)(self));
    }

    fn on_enter_notify(&mut self, window_id: WindowId) {
        self.group_mut().focus(&window_id);
    }
}


pub fn close_window(wm: &mut RustWindowManager) {
    wm.group_mut().get_focused().map(|w| w.close());
}

pub fn focus_next(wm: &mut RustWindowManager) {
    wm.group_mut().focus_next();
}

pub fn focus_previous(wm: &mut RustWindowManager) {
    wm.group_mut().focus_previous();
}

pub fn shuffle_next(wm: &mut RustWindowManager) {
    wm.group_mut().shuffle_next();
}

pub fn shuffle_previous(wm: &mut RustWindowManager) {
    wm.group_mut().shuffle_previous();
}

pub fn layout_next(wm: &mut RustWindowManager) {
    wm.group_mut().layout_next();
}

pub fn spawn_command(command: Command) -> KeyHandler {
    let mutex = Mutex::new(command);
    Rc::new(move |wm| {
                let mut command = mutex.lock().unwrap();
                command.spawn().unwrap();
            })
}

pub fn switch_group(name: &'static str) -> KeyHandler {
    Rc::new(move |wm| wm.switch_group(name))
}
