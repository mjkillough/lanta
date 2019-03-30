use crate::layout::Layout;
use crate::stack::Stack;
use crate::x::{Connection, WindowId};
use crate::Viewport;

#[derive(Clone)]
pub struct StackLayout {
    name: String,
    padding: u32,
}

impl StackLayout {
    pub fn new<S: Into<String>>(name: S, padding: u32) -> StackLayout {
        StackLayout {
            name: name.into(),
            padding,
        }
    }
}

impl Layout for StackLayout {
    fn name(&self) -> &str {
        &self.name
    }

    fn layout(&self, connection: &Connection, viewport: &Viewport, stack: &Stack<WindowId>) {
        if stack.is_empty() {
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
