use std::fmt;

use stack::Stack;
use x::{Connection, WindowId};

use super::Viewport;


pub trait LayoutClone {
    fn clone_box(&self) -> Box<Layout>;
}

impl<T> LayoutClone for T
where
    T: 'static + Layout + Clone,
{
    fn clone_box(&self) -> Box<Layout> {
        Box::new(self.clone())
    }
}


pub trait Layout: LayoutClone {
    fn name(&self) -> &str;
    fn layout(&self, connection: &Connection, viewport: &Viewport, stack: &Stack<WindowId>);
}

impl Clone for Box<Layout> {
    fn clone(&self) -> Box<Layout> {
        self.clone_box()
    }
}

impl fmt::Debug for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Layout {{ \"{}\" }}", self.name())
    }
}


#[derive(Clone)]
pub struct TiledLayout {
    name: String,
    padding: u32,
}

impl TiledLayout {
    pub fn new<S: Into<String>>(name: S, padding: u32) -> Box<Layout> {
        Box::new(TiledLayout {
            name: name.into(),
            padding,
        })
    }
}

impl Layout for TiledLayout {
    fn name(&self) -> &str {
        &self.name
    }

    fn layout(&self, connection: &Connection, viewport: &Viewport, stack: &Stack<WindowId>) {
        if stack.len() == 0 {
            return;
        }

        let tile_height = ((viewport.height - self.padding) / stack.len() as u32) - self.padding;

        for (i, window_id) in stack.iter().enumerate() {
            connection.disable_window_tracking(window_id);
            connection.map_window(window_id);
            connection.configure_window(
                window_id,
                viewport.x + self.padding,
                viewport.y + self.padding + (i as u32 * (tile_height + self.padding)),
                viewport.width - (self.padding * 2),
                tile_height,
            );
            connection.enable_window_tracking(window_id);
        }
    }
}


#[derive(Clone)]
pub struct StackLayout {
    name: String,
    padding: u32,
}

impl StackLayout {
    pub fn new<S: Into<String>>(name: S, padding: u32) -> Box<Layout> {
        Box::new(StackLayout {
            name: name.into(),
            padding,
        })
    }
}

impl Layout for StackLayout {
    fn name(&self) -> &str {
        &self.name
    }

    fn layout(&self, connection: &Connection, viewport: &Viewport, stack: &Stack<WindowId>) {
        if stack.len() == 0 {
            return;
        }

        // A non-empty `Stack` is guaranteed to have something focused.
        let focused_id = stack.focused().unwrap();

        for window_id in stack.iter() {
            if focused_id == window_id {
                continue;
            }
            connection.disable_window_tracking(window_id);
            connection.unmap_window(window_id);
            connection.enable_window_tracking(window_id);
        }

        connection.disable_window_tracking(focused_id);
        connection.map_window(focused_id);
        connection.configure_window(
            focused_id,
            viewport.x + self.padding,
            viewport.y + self.padding,
            viewport.width - (self.padding * 2),
            viewport.height - (self.padding * 2),
        );
        connection.enable_window_tracking(focused_id);
    }
}
