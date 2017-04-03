use std::rc::Rc;

use x::{Connection, WindowId};


/// A trait implemented by any objects that allow control over a window on the screen.
pub trait Window {
    fn connection(&self) -> &Connection;
    fn id(&self) -> &WindowId;

    /// Sets the window's position and size.
    fn configure(&self, x: i32, y: i32, width: i32, height: i32) {
        self.connection().configure_window(self.id(), x, y, width, height);
    }

    /// Closes the window.
    fn close(&self) {
        self.connection().close_window(&self.id());
    }
}
