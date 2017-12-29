use std::fmt;
use std::collections::HashMap;

use xcb;
use xcb_util::{ewmh, icccm};
use xcb_util::keysyms::KeySymbols;

use errors::*;
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


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WindowState {
    Modal,
    Sticky,
    MaximizedVert,
    MaximizedHorz,
    Shaded,
    SkipTaskbar,
    SkipPager,
    Hidden,
    Fullscreen,
    Above,
    Below,
    DemandsAttention,
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
            pub fn new(conn: &xcb::Connection) -> Result<InternedAtoms> {
                Ok(InternedAtoms {
                    $(
                        $name: Connection::intern_atom(conn, stringify!($name))?
                    ),*
                })
            }
        }
    };
    // Allow trailing comma:
    ( $( $name:ident ),+ , ) => (atoms!($( $name ),+);)
}


atoms!(WM_DELETE_WINDOW, WM_PROTOCOLS,);


pub struct Connection {
    conn: ewmh::Connection,
    root: WindowId,
    screen_idx: i32,
    atoms: InternedAtoms,
    window_type_lookup: HashMap<xcb::Atom, WindowType>,
    window_state_lookup: HashMap<xcb::Atom, WindowState>,
}


impl Connection {
    /// Opens a connection to the X server, returning a new Connection object.
    pub fn connect() -> Result<Connection> {
        let (conn, screen_idx) =
            xcb::Connection::connect(None).chain_err(|| "Failed to connect to X server")?;
        let conn = ewmh::Connection::connect(conn).map_err(|(e, _)| e)?;
        let root = conn.get_setup()
            .roots()
            .nth(screen_idx as usize)
            .ok_or("Invalid screen")?
            .root();

        let atoms = InternedAtoms::new(&conn).or(Err("Failed to intern atoms"))?;

        let mut types = HashMap::new();
        types.insert(conn.WM_WINDOW_TYPE_DESKTOP(), WindowType::Desktop);
        types.insert(conn.WM_WINDOW_TYPE_DOCK(), WindowType::Dock);
        types.insert(conn.WM_WINDOW_TYPE_TOOLBAR(), WindowType::Toolbar);
        types.insert(conn.WM_WINDOW_TYPE_MENU(), WindowType::Menu);
        types.insert(conn.WM_WINDOW_TYPE_UTILITY(), WindowType::Utility);
        types.insert(conn.WM_WINDOW_TYPE_SPLASH(), WindowType::Splash);
        types.insert(conn.WM_WINDOW_TYPE_DIALOG(), WindowType::Dialog);
        types.insert(
            conn.WM_WINDOW_TYPE_DROPDOWN_MENU(),
            WindowType::DropdownMenu,
        );
        types.insert(conn.WM_WINDOW_TYPE_POPUP_MENU(), WindowType::PopupMenu);
        types.insert(conn.WM_WINDOW_TYPE_TOOLTIP(), WindowType::Tooltip);
        types.insert(conn.WM_WINDOW_TYPE_NOTIFICATION(), WindowType::Notification);
        types.insert(conn.WM_WINDOW_TYPE_COMBO(), WindowType::Combo);
        types.insert(conn.WM_WINDOW_TYPE_DND(), WindowType::Dnd);
        types.insert(conn.WM_WINDOW_TYPE_NORMAL(), WindowType::Normal);

        let mut state = HashMap::new();
        state.insert(conn.WM_STATE_MODAL(), WindowState::Modal);
        state.insert(conn.WM_STATE_STICKY(), WindowState::Sticky);
        state.insert(conn.WM_STATE_MAXIMIZED_VERT(), WindowState::MaximizedVert);
        state.insert(conn.WM_STATE_MAXIMIZED_HORZ(), WindowState::MaximizedHorz);
        state.insert(conn.WM_STATE_SHADED(), WindowState::Shaded);
        state.insert(conn.WM_STATE_SKIP_TASKBAR(), WindowState::SkipTaskbar);
        state.insert(conn.WM_STATE_SKIP_PAGER(), WindowState::SkipPager);
        state.insert(conn.WM_STATE_HIDDEN(), WindowState::Hidden);
        state.insert(conn.WM_STATE_FULLSCREEN(), WindowState::Fullscreen);
        state.insert(conn.WM_STATE_ABOVE(), WindowState::Above);
        state.insert(conn.WM_STATE_BELOW(), WindowState::Below);
        state.insert(
            conn.WM_STATE_DEMANDS_ATTENTION(),
            WindowState::DemandsAttention,
        );

        Ok(Connection {
            conn,
            root: WindowId(root),
            screen_idx,
            atoms,
            window_type_lookup: types,
            window_state_lookup: state,
        })
    }

    /// Returns the Atom identifier associated with the atom_name str.
    fn intern_atom(conn: &xcb::Connection, atom_name: &str) -> Result<xcb::Atom> {
        Ok(xcb::intern_atom(conn, false, atom_name).get_reply()?.atom())
    }

    fn flush(&self) {
        self.conn.flush();
    }

    /// Installs the Connection as a window manager, by registers for
    /// SubstructureNotify and SubstructureRedirect events on the root window.
    /// If there is already a window manager on the display, then this will
    /// fail.
    pub fn install_as_wm(&self, key_handlers: &KeyHandlers) -> Result<()> {
        let values = [
            (
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
            ),
        ];
        xcb::change_window_attributes_checked(&self.conn, self.root.to_x(), &values)
            .request_check()
            .or(Err("Could not register SUBSTRUCTURE_NOTIFY/REDIRECT"
                .to_owned()))?;

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
        let focused_idx = groups
            .focused()
            .and_then(|focused| groups.iter().position(|g| g.name() == focused.name()));
        match focused_idx {
            Some(idx) => {
                ewmh::set_current_desktop(&self.conn, self.screen_idx, idx as u32);
            }
            None => {
                error!("Invariant: failed to get active group index");
            }
        };
    }

    pub fn top_level_windows(&self) -> Result<Vec<WindowId>> {
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
    fn get_wm_protocols(&self, window_id: &WindowId) -> Result<Vec<xcb::Atom>> {
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

    pub fn get_window_states(&self, window_id: &WindowId) -> Vec<WindowState> {
        // EWMH states to ignore any we don't understand.
        // Don't error if no window states set.
        ewmh::get_wm_state(&self.conn, window_id.to_x())
            .get_reply()
            .map(|reply| {
                reply
                    .atoms()
                    .iter()
                    .filter_map(|a| self.window_state_lookup.get(a).cloned())
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
        let has_wm_delete_window = self.get_wm_protocols(window_id)
            .map(|protocols| protocols.contains(&self.atoms.WM_DELETE_WINDOW))
            .unwrap_or(false);

        if has_wm_delete_window {
            info!("Closing window {} using WM_DELETE", window_id);
            let data = xcb::ClientMessageData::from_data32([
                self.atoms.WM_DELETE_WINDOW,
                xcb::CURRENT_TIME,
                0,
                0,
                0,
            ]);
            let event =
                xcb::ClientMessageEvent::new(32, window_id.to_x(), self.atoms.WM_PROTOCOLS, data);
            xcb::send_event(
                &self.conn,
                false,
                window_id.to_x(),
                xcb::EVENT_MASK_NO_EVENT,
                &event,
            );
        } else {
            info!("Closing window {} using xcb::destroy_window()", window_id);
            xcb::destroy_window(&self.conn, window_id.to_x());
        }
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
    ///
    /// If it fails to register any of the keys, it will log an error and continue.
    pub fn enable_window_key_events(&self, window_id: &WindowId, key_handlers: &KeyHandlers) {
        let key_symbols = KeySymbols::new(&self.conn);
        for key in key_handlers.key_combos() {
            match key_symbols.get_keycode(key.keysym).next() {
                Some(keycode) => {
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
                None => {
                    error!(
                        "Failed to get keycode for keysym {} - could not register handler on {}",
                        key.keysym, window_id
                    );
                }
            }
        }
    }

    pub fn enable_window_tracking(&self, window_id: &WindowId) {
        let values = [
            (
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_ENTER_WINDOW | xcb::EVENT_MASK_STRUCTURE_NOTIFY,
            ),
        ];
        xcb::change_window_attributes(&self.conn, window_id.to_x(), &values);
    }

    pub fn disable_window_tracking(&self, window_id: &WindowId) {
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

    /// Unsets EWMH's _NET_ACTIVE_WINDOW to indicate there is no active window.
    pub fn focus_nothing(&self) {
        ewmh::set_active_window(&self.conn, self.screen_idx, xcb::NONE);
    }

    pub fn get_event_loop(&self) -> EventLoop {
        EventLoop { connection: self }
    }
}


/// Events received from the `EventLoop`.
pub enum Event {
    MapRequest(WindowId),
    UnmapNotify(WindowId),
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

            let event = self.connection
                .conn
                .wait_for_event()
                .expect("wait_for_event() returned None: IO error?");

            unsafe {
                let propagate = match event.response_type() {
                    xcb::CONFIGURE_REQUEST => self.on_configure_request(xcb::cast_event(&event)),
                    xcb::MAP_REQUEST => self.on_map_request(xcb::cast_event(&event)),
                    xcb::UNMAP_NOTIFY => self.on_unmap_notify(xcb::cast_event(&event)),
                    xcb::DESTROY_NOTIFY => self.on_destroy_notify(xcb::cast_event(&event)),
                    xcb::KEY_PRESS => self.on_key_press(xcb::cast_event(&event)),
                    xcb::ENTER_NOTIFY => self.on_enter_notify(xcb::cast_event(&event)),
                    _ => None,
                };

                if let Some(propagate_event) = propagate {
                    return Some(propagate_event);
                }
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
                event.border_width() as u32,
            ),
            (xcb::CONFIG_WINDOW_SIBLING as u16, event.sibling() as u32),
            (
                xcb::CONFIG_WINDOW_STACK_MODE as u16,
                event.stack_mode() as u32,
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

    fn on_unmap_notify(&self, event: &xcb::UnmapNotifyEvent) -> Option<Event> {
        // Ignore UnmapNotify events that come from our SUBSTRUCTURE_NOTIFY mask
        // on the root window. We are interested only in the events that come from
        // the windows themselves, which allows our `Connection::disable_window_tracking()`
        // to stop us seeing unwanted UnmapNotify events.
        if event.event() != self.connection.root_window_id().to_x() {
            Some(Event::UnmapNotify(WindowId(event.window())))
        } else {
            None
        }
    }

    fn on_destroy_notify(&self, event: &xcb::DestroyNotifyEvent) -> Option<Event> {
        Some(Event::DestroyNotify(WindowId(event.window())))
    }

    fn on_key_press(&self, event: &xcb::KeyPressEvent) -> Option<Event> {
        let key_symbols = KeySymbols::new(&self.connection.conn);
        let keysym = key_symbols.press_lookup_keysym(event, 0);
        let mod_mask = event.state() as u32 & ModKey::mask_all();
        let key = KeyCombo { mod_mask, keysym };
        Some(Event::KeyPress(key))
    }

    fn on_enter_notify(&self, event: &xcb::EnterNotifyEvent) -> Option<Event> {
        Some(Event::EnterNotify(WindowId(event.event())))
    }
}
