use std::fmt;
use std::collections::HashMap;

use xcb;
use xcb_util::{ewmh, icccm};
use xcb_util::keysyms::KeySymbols;

use debug;
use keys::{KeyCombo, KeyHandlers, ModKey};
use groups::Group;
use stack::Stack;


pub use self::ewmh::StrutPartial;


/// A handle to an X Window.
#[derive(Debug, PartialEq)]
pub struct WindowId(xcb::Window);

impl WindowId {
    fn to_x(&self) -> xcb::Window {
        self.0
    }
}

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WindowType {
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
    Dialog,
    DropdownMenu,
    PopupMenu,
    Tooltip,
    Notification,
    Combo,
    Dnd,
    Normal,
}


macro_rules! atoms {
    ( $( $name:ident ),+ ) => {
        #[allow(non_snake_case)]
        struct InternedAtoms {
            $(
                pub $name: xcb::Atom
            ),*
        }

        impl InternedAtoms {
            pub fn new(conn: &xcb::Connection) -> Result<InternedAtoms, xcb::GenericError> {
                Ok(InternedAtoms {
                    $(
                        $name: Connection::intern_atom(conn, stringify!($atom))?
                    ),*
                })
            }
        }
    };
    // Allow trailing comma:
    ( $( $name:ident ),+ , ) => (atoms!($( $name ),+);)
}


atoms!(
    WM_DELETE_WINDOW,
    WM_PROTOCOLS,
    _NET_NUMBER_OF_DESKTOPS,
    _NET_CURRENT_DESKTOP,
    _NET_DESKTOP_NAMES,
);


pub struct Connection {
    conn: ewmh::Connection,
    root: WindowId,
    screen_idx: i32,
    atoms: InternedAtoms,
    window_type_lookup: HashMap<xcb::Atom, WindowType>,
}


impl Connection {
    /// Opens a connection to the X server, returning a new Connection object.
    pub fn connect() -> Result<Connection, String> {
        let (conn, screen_idx) = xcb::Connection::connect(None).unwrap();
        let conn = ewmh::Connection::connect(conn).map_err(|_| ()).unwrap();
        let root = conn.get_setup()
            .roots()
            .nth(screen_idx as usize)
            .ok_or("Invalid screen")?
            .root();

        let atoms = InternedAtoms::new(&conn).or(Err("Failed to intern atoms"))?;

        let mut lookup = HashMap::new();
        lookup.insert(conn.WM_WINDOW_TYPE_DESKTOP(), WindowType::Desktop);
        lookup.insert(conn.WM_WINDOW_TYPE_DOCK(), WindowType::Dock);
        lookup.insert(conn.WM_WINDOW_TYPE_TOOLBAR(), WindowType::Toolbar);
        lookup.insert(conn.WM_WINDOW_TYPE_MENU(), WindowType::Menu);
        lookup.insert(conn.WM_WINDOW_TYPE_UTILITY(), WindowType::Utility);
        lookup.insert(conn.WM_WINDOW_TYPE_SPLASH(), WindowType::Splash);
        lookup.insert(conn.WM_WINDOW_TYPE_DIALOG(), WindowType::Dialog);
        lookup.insert(
            conn.WM_WINDOW_TYPE_DROPDOWN_MENU(),
            WindowType::DropdownMenu,
        );
        lookup.insert(conn.WM_WINDOW_TYPE_POPUP_MENU(), WindowType::PopupMenu);
        lookup.insert(conn.WM_WINDOW_TYPE_TOOLTIP(), WindowType::Tooltip);
        lookup.insert(conn.WM_WINDOW_TYPE_NOTIFICATION(), WindowType::Notification);
        lookup.insert(conn.WM_WINDOW_TYPE_COMBO(), WindowType::Combo);
        lookup.insert(conn.WM_WINDOW_TYPE_DND(), WindowType::Dnd);
        lookup.insert(conn.WM_WINDOW_TYPE_NORMAL(), WindowType::Normal);

        Ok(Connection {
            conn,
            root: WindowId(root),
            screen_idx,
            atoms,
            window_type_lookup: lookup,
        })
    }

    /// Returns the Atom identifier associated with the atom_name str.
    fn intern_atom(
        conn: &xcb::Connection,
        atom_name: &str,
    ) -> Result<xcb::Atom, xcb::GenericError> {
        Ok(xcb::intern_atom(conn, false, atom_name).get_reply()?.atom())
    }

    fn flush(&self) {
        self.conn.flush();
    }

    /// Installs the Connection as a window manager, by registers for
    /// SubstructureNotify and SubstructureRedirect events on the root window.
    /// If there is already a window manager on the display, then this will
    /// fail.
    pub fn install_as_wm(&self, key_handlers: &KeyHandlers) -> Result<(), String> {
        let values = [
            (
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
            ),
        ];
        xcb::change_window_attributes_checked(&self.conn, self.root.to_x(), &values)
            .request_check()
            .or(Err(
                "Could not register SUBSTRUCTURE_NOTIFY/REDIRECT".to_owned(),
            ))?;

        self.enable_window_key_events(&self.root, key_handlers);

        Ok(())
    }

    /// Returns the ID of the root window.
    pub fn root_window_id(&self) -> &WindowId {
        &self.root
    }

    pub fn update_ewmh_desktops(&self, groups: &Stack<Group>) {
        let group_names = groups.iter().map(|g| g.name());
        ewmh::set_desktop_names(&self.conn, self.screen_idx, group_names);
        ewmh::set_number_of_desktops(&self.conn, self.screen_idx, groups.len() as u32);

        // Matching the current group on name isn't perfect, but it's good enough for
        // EWMH.
        let focused_idx = groups.focused().and_then(|focused| {
            groups.iter().position(|g| g.name() == focused.name())
        });
        match focused_idx {
            Some(idx) => {
                ewmh::set_current_desktop(&self.conn, self.screen_idx, idx as u32);
            }
            None => {
                error!("Invariant: failed to get active group index");
            }
        };
    }

    pub fn top_level_windows(&self) -> Result<Vec<WindowId>, xcb::GenericError> {
        let windows = xcb::query_tree(&self.conn, self.root.to_x())
            .get_reply()?
            .children()
            .iter()
            .map(|w| WindowId(*w))
            .collect();
        Ok(windows)
    }

    /// Queries the WM_PROTOCOLS property of a window, returning a list of the
    /// protocols that it supports.
    // TODO: Have this return a list of atoms, rather than a list of strings.
    // (Perhaps we should
    // have a separate function to convert to a list of strings for debugging?)
    fn get_wm_protocols(&self, window_id: &WindowId) -> Result<Vec<xcb::Atom>, xcb::GenericError> {
        let reply = icccm::get_wm_protocols(&self.conn, window_id.to_x(), self.atoms.WM_PROTOCOLS)
            .get_reply()?;
        Ok(reply.atoms().to_vec())
    }

    pub fn get_window_types(&self, window_id: &WindowId) -> Vec<WindowType> {
        // Filter out any types we don't understand, as that's what the EWMH
        // spec suggests we should do. Don't error if _NET_WM_WINDOW_TYPE
        // is not set - lots of applications don't bother.
        ewmh::get_wm_window_type(&self.conn, window_id.to_x())
            .get_reply()
            .map(|reply| {
                reply
                    .atoms()
                    .iter()
                    .filter_map(|a| self.window_type_lookup.get(a).cloned())
                    .collect()
            })
            .unwrap_or(Vec::new())
    }

    pub fn get_strut_partial(&self, window_id: &WindowId) -> Option<StrutPartial> {
        ewmh::get_wm_strut_partial(&self.conn, window_id.to_x())
            .get_reply()
            .ok()
    }

    /// Closes a window.
    ///
    /// The window will be closed gracefully using the ICCCM WM_DELETE_WINDOW
    /// protocol if it is supported.
    pub fn close_window(&self, window_id: &WindowId) {
        let protocols = self.get_wm_protocols(window_id).unwrap();
        let has_wm_delete_window = protocols.contains(&self.atoms.WM_DELETE_WINDOW);

        // TODO: Use XDestroyWindow to forcefully close windows that do not support
        // WM_DELETE_WINDOW.
        if !has_wm_delete_window {
            panic!("Not implemented: closing windows that don't expose WM_DELETE_WINDOW");
        }

        let data = xcb::ClientMessageData::from_data32(
            [self.atoms.WM_DELETE_WINDOW, xcb::CURRENT_TIME, 0, 0, 0],
        );
        let event =
            xcb::ClientMessageEvent::new(32, window_id.to_x(), self.atoms.WM_PROTOCOLS, data);
        xcb::send_event(
            &self.conn,
            false,
            window_id.to_x(),
            xcb::EVENT_MASK_NO_EVENT,
            &event,
        );
    }

    /// Sets the window's position and size.
    pub fn configure_window(&self, window_id: &WindowId, x: u32, y: u32, width: u32, height: u32) {
        let values = [
            (xcb::CONFIG_WINDOW_X as u16, x),
            (xcb::CONFIG_WINDOW_Y as u16, y),
            (xcb::CONFIG_WINDOW_WIDTH as u16, width),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, height),
        ];
        xcb::configure_window(&self.conn, window_id.to_x(), &values);
    }

    /// Get's the window's width and height.
    pub fn get_window_geometry(&self, window_id: &WindowId) -> (u32, u32) {
        let reply = xcb::get_geometry(&self.conn, window_id.to_x())
            .get_reply()
            .unwrap();
        // Cast as everywhere else uses u32.
        (reply.width() as u32, reply.height() as u32)
    }

    /// Map a window.
    pub fn map_window(&self, window_id: &WindowId) {
        xcb::map_window(&self.conn, window_id.to_x());
    }

    /// Unmap a window.
    pub fn unmap_window(&self, window_id: &WindowId) {
        xcb::unmap_window(&self.conn, window_id.to_x());
    }

    /// Registers for key events.
    pub fn enable_window_key_events(&self, window_id: &WindowId, key_handlers: &KeyHandlers) {
        let key_symbols = KeySymbols::new(&self.conn).expect("Failed to create KeySymbols");
        for key in key_handlers.key_combos() {
            let keycode = key_symbols.get_keycode(key.keysym);
            xcb::grab_key(
                &self.conn,
                false,
                window_id.to_x(),
                key.mod_mask as u16,
                keycode,
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            );
        }
    }

    pub fn enable_window_focus_tracking(&self, window_id: &WindowId) {
        let values = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_ENTER_WINDOW)];
        xcb::change_window_attributes(&self.conn, window_id.to_x(), &values);
    }

    pub fn disable_window_focus_tracking(&self, window_id: &WindowId) {
        let values = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_NO_EVENT)];
        xcb::change_window_attributes(&self.conn, window_id.to_x(), &values);
    }

    pub fn focus_window(&self, window_id: &WindowId) {
        xcb::set_input_focus(
            &self.conn,
            xcb::INPUT_FOCUS_POINTER_ROOT as u8,
            window_id.to_x(),
            xcb::CURRENT_TIME,
        );
        ewmh::set_active_window(&self.conn, self.screen_idx, window_id.to_x());
    }

    pub fn get_event_loop(&self) -> EventLoop {
        EventLoop { connection: self }
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
            // Flush any pending operations that came out of the event we (might
            // have) just yielded.
            self.connection.flush();

            let event = self.connection.conn.wait_for_event().expect(
                "wait_for_event() returned None: IO error?",
            );

            let propagate = match event.response_type() {
                xcb::CONFIGURE_REQUEST => self.on_configure_request(xcb::cast_event(&event)),
                xcb::MAP_REQUEST => self.on_map_request(xcb::cast_event(&event)),
                xcb::DESTROY_NOTIFY => self.on_destroy_notify(xcb::cast_event(&event)),
                xcb::KEY_PRESS => self.on_key_press(xcb::cast_event(&event)),
                xcb::ENTER_NOTIFY => self.on_enter_notify(xcb::cast_event(&event)),
                _ => {
                    debug!("Unhandled event: {}", debug::xcb_event_to_str(&event));
                    None
                }
            };

            if let Some(propagate_event) = propagate {
                return Some(propagate_event);
            }
        }
    }
}

impl<'a> EventLoop<'a> {
    fn on_configure_request(&self, event: &xcb::ConfigureRequestEvent) -> Option<Event> {
        // This request is not interesting for us: grant it unchanged.
        // Build a request with all attributes set, then filter out to only include
        // those from the original request.
        let values = vec![
            (xcb::CONFIG_WINDOW_X as u16, event.x() as u32),
            (xcb::CONFIG_WINDOW_Y as u16, event.y() as u32),
            (xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32),
            (
                xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
                event.border_width() as u32
            ),
            (xcb::CONFIG_WINDOW_SIBLING as u16, event.sibling() as u32),
            (
                xcb::CONFIG_WINDOW_STACK_MODE as u16,
                event.stack_mode() as u32
            ),
        ];
        let filtered_values: Vec<_> = values
            .into_iter()
            .filter(|&(mask, _)| mask & event.value_mask() != 0)
            .collect();
        xcb::configure_window(&self.connection.conn, event.window(), &filtered_values);

        // There's no value in propogating this event.
        None
    }

    fn on_map_request(&self, event: &xcb::MapRequestEvent) -> Option<Event> {
        Some(Event::MapRequest(WindowId(event.window())))
    }

    fn on_destroy_notify(&self, event: &xcb::DestroyNotifyEvent) -> Option<Event> {
        Some(Event::DestroyNotify(WindowId(event.window())))
    }

    fn on_key_press(&self, event: &xcb::KeyPressEvent) -> Option<Event> {
        let key_symbols =
            KeySymbols::new(&self.connection.conn).expect("Failed to create KeySymbols");
        let keysym = key_symbols.key_press_lookup_keysym(event, 0);
        let mod_mask = event.state() as u32 & ModKey::mask_all();
        let key = KeyCombo { mod_mask, keysym };
        Some(Event::KeyPress(key))
    }

    fn on_enter_notify(&self, event: &xcb::EnterNotifyEvent) -> Option<Event> {
        Some(Event::EnterNotify(WindowId(event.event())))
    }
}
