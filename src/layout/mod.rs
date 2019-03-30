use std::fmt;

use crate::stack::Stack;
use crate::x::{Connection, WindowId};
use crate::Viewport;

mod stack;
mod tiled;

pub use self::stack::StackLayout;
pub use self::tiled::TiledLayout;

pub trait LayoutClone {
    fn clone_box(&self) -> Box<dyn Layout>;
}

impl<T> LayoutClone for T
where
    T: 'static + Layout + Clone,
{
    fn clone_box(&self) -> Box<dyn Layout> {
        Box::new(self.clone())
    }
}

pub trait Layout: LayoutClone {
    fn name(&self) -> &str;
    fn layout(&self, connection: &Connection, viewport: &Viewport, stack: &Stack<WindowId>);
}

impl Clone for Box<dyn Layout> {
    fn clone(&self) -> Box<dyn Layout> {
        self.clone_box()
    }
}

impl fmt::Debug for dyn Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Layout {{ \"{}\" }}", self.name())
    }
}
