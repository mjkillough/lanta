use std::rc::Rc;

use x::{Connection, WindowId};


pub struct Window {
    connection: Rc<Connection>,
    pub id: WindowId,
}

impl Window {
    pub fn new(connection: Rc<Connection>, id: WindowId) -> Window {
        Window {
            connection: connection,
            id: id,
        }
    }

    /// Sets the window's position and size.
    pub fn configure(&self, x: i32, y: i32, width: i32, height: i32) {
        self.connection.configure_window(self.id, x, y, width, height);
    }

    /// Closes the window.
    pub fn close(&self) {
        self.connection.close_window(self.id);
    }
}
