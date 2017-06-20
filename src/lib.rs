// #![deny(warnings)]

extern crate fern;
#[macro_use]
extern crate log;
extern crate libc;
extern crate time;
extern crate xcb;
extern crate xcb_util;
extern crate xdg;

use std::cmp;
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
use x::{Connection, Event, StrutPartial, WindowId, WindowType};


/// Initializes a logger using the default configuration.
///
/// Outputs to stdout and $XDG_DATA/lanta/lanta.log by default.
/// You should feel free to initialize your own logger, instead of using this.
pub fn intiailize_logger() {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("lanta")
        .expect("Could not create xdg BaseDirectories");
    let log_path = xdg_dirs
        .place_data_file("lanta.log")
        .expect("Could not create log file");

    let logger_config = fern::DispatchConfig {
        format: Box::new(
            |msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
                format!("[{}] [{}] {}", time::now().rfc3339(), level, msg)
            },
        ),
        output: vec![
            fern::OutputConfig::stdout(),
            fern::OutputConfig::file(&log_path),
        ],
        level: log::LogLevelFilter::Trace,
    };
    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
}


#[derive(Clone, Copy, Debug, Default)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}


struct Dock {
    window_id: WindowId,
    strut_partial: Option<StrutPartial>,
}


#[derive(Default)]
struct Docks {
    vec: Vec<Dock>,
}

impl Docks {
    pub fn add(&mut self, conn: &Connection, window_id: WindowId) {
        let strut_partial = conn.get_strut_partial(&window_id);
        self.vec.push(Dock {
            window_id,
            strut_partial,
        });
    }

    pub fn remove(&mut self, window_id: &WindowId) {
        self.vec.retain(|d| &d.window_id != window_id);
    }

    /// Figure out the usable area of the screen based on the STRUT_PARTIAL of
    /// all docks.
    pub fn viewport(&self, screen_width: u32, screen_height: u32) -> Viewport {
        let (left, right, top, bottom) = self.vec
            .iter()
            .filter_map(|o| o.strut_partial.as_ref())
            .fold((0, 0, 0, 0), |(left, right, top, bottom), s| {
                // We don't bother looking at the start/end members of the
                // StrutPartial - treating it more like a Strut.
                (
                    cmp::max(left, s.left()),
                    cmp::max(right, s.right()),
                    cmp::max(top, s.top()),
                    cmp::max(bottom, s.bottom()),
                )
            });
        let viewport = Viewport {
            x: left,
            y: top,
            width: screen_width - left - right,
            height: screen_height - top - bottom,
        };
        debug!("Calculated Viewport as {:?}", viewport);
        viewport
    }
}


pub struct RustWindowManager {
    connection: Rc<Connection>,
    keys: KeyHandlers,
    groups: Stack<Group>,
    docks: Docks,
}

impl RustWindowManager {
    pub fn new<K>(
        keys: K,
        groups: Vec<GroupBuilder>,
        layouts: Vec<Box<Layout>>,
    ) -> Result<Self, String>
    where
        K: Into<KeyHandlers>,
    {
        let keys = keys.into();
        let connection = Rc::new(Connection::connect()?);
        connection.install_as_wm(&keys)?;

        let groups = Stack::from(
            groups
                .into_iter()
                .map(|group: GroupBuilder| {
                    group.build(connection.clone(), layouts.clone())
                })
                .collect::<Vec<Group>>(),
        );

        let mut wm = RustWindowManager {
            connection: connection.clone(),
            keys: keys,
            groups: groups,
            docks: Docks::default(),
        };

        // Learn about existing top-level windows.
        let existing_windows = connection.top_level_windows().unwrap();
        for window in existing_windows {
            wm.manage_window(window);
        }
        let viewport = wm.viewport();
        wm.group_mut().activate(viewport);
        wm.connection.update_ewmh_desktops(&wm.groups);

        Ok(wm)
    }

    fn viewport(&self) -> Viewport {
        let (width, height) = self.connection
            .get_window_geometry(self.connection.root_window_id());
        self.docks.viewport(width, height)
    }

    pub fn group(&self) -> &Group {
        self.groups.focused().expect("No active group!")
    }

    pub fn group_mut(&mut self) -> &mut Group {
        self.groups.focused_mut().expect("No active group!")
    }

    pub fn switch_group<'a, S>(&'a mut self, name: S)
    where
        S: Into<&'a str>,
    {
        let name = name.into();

        // If we're already on this group, do nothing.
        if self.group().name() == name {
            return;
        }

        self.group_mut().deactivate();
        self.groups.focus(|group| group.name() == name);
        let viewport = self.viewport();
        self.group_mut().activate(viewport);
        self.connection.update_ewmh_desktops(&self.groups);
    }

    /// Move the focused window from the active group to another named group.
    ///
    /// If the other named group does not exist, then the window is
    /// (unfortunately) lost.
    pub fn move_focused_to_group<'a, S>(&'a mut self, name: S)
    where
        S: Into<&'a str>,
    {
        let name = name.into();

        // If the group is currently active, then do nothing. This avoids flicker as we
        // unmap/remap.
        if name == self.group().name() {
            return;
        }

        let removed = self.group_mut().remove_focused();
        let new_group_opt = self.groups.iter_mut().find(|group| group.name() == name);
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

    pub fn manage_window(&mut self, window_id: WindowId) {
        debug!("Managing window: {}", window_id);

        // If we are already managing the window, then do nothing.
        // This may result in some undesireableness, in that we don't move
        // it to the currently focused group when it's remapped, but it shouldn't
        // be too bad.
        let already_managed = self.groups.iter().any(|g| g.contains(&window_id));
        if already_managed {
            warn!(
                "Asked to manage window that's already managed: {}",
                window_id
            );
            return;
        }

        let window_types = self.connection.get_window_types(&window_id);
        let dock = window_types.contains(&WindowType::Dock);

        self.connection
            .enable_window_key_events(&window_id, &self.keys);

        if dock {
            self.docks.add(&self.connection, window_id);
            let viewport = self.viewport();
            self.group_mut().update_viewport(viewport);
        } else {
            self.connection.enable_window_tracking(&window_id);
            self.group_mut().add_window(window_id);
        }
    }

    pub fn unmanage_window(&mut self, window_id: WindowId) {
        debug!("Unmanaging window: {}", window_id);

        // Remove the window from whichever Group it is in. Special case for
        // docks which aren't in any group.
        self.groups
            .iter_mut()
            .find(|group| group.contains(&window_id))
            .map(|group| group.remove_window(&window_id));
        self.docks.remove(&window_id);

        // The viewport may have changed.
        let viewport = self.viewport();
        self.group_mut().update_viewport(viewport);
    }

    pub fn run_event_loop(&mut self) {
        let event_loop_connection = self.connection.clone();
        let event_loop = event_loop_connection.get_event_loop();
        for event in event_loop {
            match event {
                Event::MapRequest(window_id) => self.on_map_request(window_id),
                Event::UnmapNotify(window_id) => self.on_unmap_notify(window_id),
                Event::DestroyNotify(window_id) => self.on_destroy_notify(window_id),
                Event::KeyPress(key) => self.on_key_press(key),
                Event::EnterNotify(window_id) => self.on_enter_notify(window_id),
            }
        }
        info!("Event loop exiting");
    }

    fn on_map_request(&mut self, window_id: WindowId) {
        self.connection.map_window(&window_id);
        self.manage_window(window_id);
    }

    fn on_unmap_notify(&mut self, window_id: WindowId) {
        // We only receive an unmap notify event when the window is actually
        // unmapped by its application. When our layouts unmap windows, they
        // (should) do it by disabling event tracking first.
        self.unmanage_window(window_id);
    }

    fn on_destroy_notify(&mut self, window_id: WindowId) {
        self.unmanage_window(window_id);
    }

    fn on_key_press(&mut self, key: KeyCombo) {
        self.keys.get(&key).map(move |handler| (handler)(self));
    }

    fn on_enter_notify(&mut self, window_id: WindowId) {
        self.group_mut().focus(&window_id);
    }
}
