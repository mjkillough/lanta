// #![deny(warnings)]

#[macro_use]
extern crate log;
extern crate libc;
extern crate x11;

use std::rc::Rc;

pub mod cmd;
mod debug;
pub mod groups;
pub mod keys;
pub mod layout;
mod stack;
pub mod window;
pub mod x;

use groups::{Group, GroupBuilder};
use keys::{KeyCombo, KeyHandlers};
use layout::Layout;
use stack::Stack;
use x::{Connection, Event, WindowId};


pub struct RustWindowManager {
    connection: Rc<Connection>,
    keys: KeyHandlers,
    groups: Stack<Group>,
}

impl RustWindowManager {
    pub fn new<K>(keys: K,
                  groups: Vec<GroupBuilder>,
                  layouts: Vec<Box<Layout>>)
                  -> Result<Self, String>
        where K: Into<KeyHandlers>
    {
        let keys = keys.into();
        let connection = Rc::new(Connection::connect()?);
        connection.install_as_wm(&keys)?;

        let mut groups = Stack::from(groups
                                         .into_iter()
                                         .map(|group: GroupBuilder| {
                                                  group.build(connection.clone(), layouts.clone())
                                              })
                                         .collect::<Vec<Group>>());

        // Stack guarantees that if it is non-empty, then something will be focused.
        // This captures the case when we have no groups configured - useless in
        // practise, but maybe there'll be some use for it in tests.
        if let Some(group) = groups.focused_mut() {
            // Add all existing windows to the default group.
            let existing_windows = connection.top_level_windows();
            for window in existing_windows {
                group.add_window(window);
            }

            group.activate();
        }

        Ok(RustWindowManager {
               connection: connection.clone(),
               keys: keys,
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

    /// Move the focused window from the active group to another named group.
    ///
    /// If the other named group does not exist, then the window is
    /// (unfortunately) lost.
    pub fn move_focused_to_group<'a, S>(&'a mut self, name: S)
        where S: Into<&'a str>
    {
        let name = name.into();

        // If the group is currently active, then do nothing. This avoids flicker as we
        // unmap/remap.
        if name == self.group().name() {
            return;
        }

        let removed = self.group_mut().remove_focused();
        let new_group_opt = self.groups
            .iter_mut()
            .find(|group| group.name() == name);
        match new_group_opt {
            Some(new_group) => {
                removed.map(|window| new_group.add_window(window));
            }
            None => {
                // It would be nice to put the window back in its group (or avoid taking it out
                // of its group until we've checked the new group exists), but it's difficult
                // to do this while keeping the borrow checker happy.
                error!("Moved window to non-existent group: {}", name);
            }
        }
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
            .enable_window_key_events(&window_id, &self.keys);
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
        self.keys.get(&key).map(move |handler| (handler)(self));
    }

    fn on_enter_notify(&mut self, window_id: WindowId) {
        self.group_mut().focus(&window_id);
    }
}
