use x::{Connection, WindowId};


/// A trait implemented by any objects that allow control over a window on the
/// screen.
pub trait Window {
    fn connection(&self) -> &Connection;
    fn id(&self) -> &WindowId;

    fn without_focus_tracking<'a, F>(&'a self, func: F)
        where F: Fn(&'a Self)
    {
        self.connection()
            .disable_window_focus_tracking(self.id());
        (func)(self);
        self.connection().enable_window_focus_tracking(self.id());
    }

    /// Maps the window.
    fn map(&self) {
        debug!("Mapping window: {}", self.id());
        self.connection().map_window(self.id());
    }

    /// Unmaps the window.
    fn unmap(&self) {
        debug!("Unmapping window: {}", self.id());
        self.connection().unmap_window(self.id());
    }

    /// Sets the window's position and size.
    fn configure(&self, x: i32, y: i32, width: i32, height: i32) {
        debug!("Configuring window: {} (x={}, y={}, width={}, height={})",
               self.id(),
               x,
               y,
               width,
               height);
        self.connection()
            .configure_window(self.id(), x, y, width, height);
    }

    /// Closes the window.
    fn close(&self) {
        info!("Closing window: {}", self.id());
        self.connection().close_window(&self.id());
    }
}
