use std::rc::Rc;

use super::Viewport;
use crate::layout::Layout;
use crate::stack::Stack;
use crate::x::{Connection, WindowId};

#[derive(Clone)]
pub struct GroupBuilder {
    name: String,
    default_layout: String,
}

impl GroupBuilder {
    pub fn new<S1, S2>(name: S1, default_layout: S2) -> GroupBuilder
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        GroupBuilder {
            name: name.into(),
            default_layout: default_layout.into(),
        }
    }

    pub fn build(self, connection: Rc<Connection>, layouts: Vec<Box<dyn Layout>>) -> Group {
        let mut layouts_stack = Stack::from(layouts);
        layouts_stack.focus(|layout| layout.name() == self.default_layout);

        Group {
            connection,
            name: self.name.clone(),
            active: false,
            stack: Stack::new(),
            layouts: layouts_stack,
            viewport: Viewport::default(),
        }
    }
}

pub struct Group {
    name: String,
    connection: Rc<Connection>,
    active: bool,
    stack: Stack<WindowId>,
    layouts: Stack<Box<dyn Layout>>,
    viewport: Viewport,
}

impl Group {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn activate(&mut self, viewport: Viewport) {
        info!("Activating group: {}", self.name());
        self.active = true;
        self.viewport = viewport;
        self.perform_layout();
    }

    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.perform_layout();
    }

    pub fn deactivate(&mut self) {
        info!("Deactivating group: {}", self.name());
        for window_id in self.stack.iter() {
            self.connection.disable_window_tracking(window_id);
            self.connection.unmap_window(window_id);
            self.connection.enable_window_tracking(window_id);
        }
        self.active = false;
    }

    fn perform_layout(&mut self) {
        if !self.active {
            return;
        }

        if let Some(layout) = self.layouts.focused() {
            layout.layout(&self.connection, &self.viewport, &self.stack)
        }

        // Tell X to focus the focused window for this group, or to unset
        // it's focus if we have no windows.
        match self.stack.focused() {
            Some(window_id) => self.connection.focus_window(window_id),
            None => self.connection.focus_nothing(),
        }
    }

    pub fn add_window(&mut self, window_id: WindowId) {
        info!("Adding window to group {}: {}", self.name(), window_id);
        self.stack.push(window_id);
        self.perform_layout();
    }

    pub fn remove_window(&mut self, window_id: &WindowId) -> WindowId {
        info!("Removing window from group {}: {}", self.name(), window_id);
        let removed = self.stack.remove(|w| w == window_id);
        self.perform_layout();
        removed
    }

    pub fn remove_focused(&mut self) -> Option<WindowId> {
        info!(
            "Removing focused window from group {}: {:?}",
            self.name(),
            self.stack.focused()
        );
        let removed = self.stack.remove_focused();
        self.perform_layout();
        removed.map(|window| {
            self.connection.disable_window_tracking(&window);
            self.connection.unmap_window(&window);
            self.connection.enable_window_tracking(&window);
            window
        })
    }

    pub fn contains(&self, window_id: &WindowId) -> bool {
        self.stack.iter().any(|w| w == window_id)
    }

    pub fn focus(&mut self, window_id: &WindowId) {
        info!("Focusing window in group {}: {}", self.name(), window_id);
        self.stack.focus(|id| id == window_id);
        self.perform_layout();
    }

    pub fn close_focused(&self) {
        if let Some(window_id) = self.stack.focused() {
            self.connection.close_window(window_id);
        }
    }

    pub fn focus_next(&mut self) {
        self.stack.focus_next();
        info!(
            "Focusing next window in group {}: {:?}",
            self.name(),
            self.stack.focused()
        );
        self.perform_layout();
    }

    pub fn focus_previous(&mut self) {
        self.stack.focus_previous();
        info!(
            "Focusing previous window in group {}: {:?}",
            self.name(),
            self.stack.focused()
        );
        self.perform_layout();
    }

    pub fn shuffle_next(&mut self) {
        info!(
            "Shuffling focused window to next position in group {}: {:?}",
            self.name(),
            self.stack.focused()
        );
        self.stack.shuffle_next();
        self.perform_layout();
    }

    pub fn shuffle_previous(&mut self) {
        info!(
            "Shuffling focused window to previous position in group {}: {:?}",
            self.name(),
            self.stack.focused()
        );
        self.stack.shuffle_previous();
        self.perform_layout();
    }

    pub fn layout_next(&mut self) {
        self.layouts.focus_next();
        info!(
            "Switching to next layout in group {}: {:?}",
            self.name(),
            self.layouts.focused()
        );
        self.perform_layout();
    }

    pub fn layout_previous(&mut self) {
        self.layouts.focus_next();
        info!(
            "Switching to previous layout in group {}: {:?}",
            self.name(),
            self.layouts.focused()
        );
        self.layouts.focus_previous();
        self.perform_layout();
    }
}
