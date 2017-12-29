use std::fmt;

use stack::Stack;
use Viewport;
use x::{Connection, WindowId};

mod stack;
mod tiled;

pub use self::stack::StackLayout;
pub use self::tiled::TiledLayout;


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
