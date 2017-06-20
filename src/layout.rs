use std::fmt;

use groups::{GroupIter, GroupWindow};
use window::Window;
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
    fn layout(&self, viewport: &Viewport, focused: Option<GroupWindow>, stack: GroupIter);
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

    fn layout(&self, viewport: &Viewport, _: Option<GroupWindow>, stack: GroupIter) {
        if stack.len() == 0 {
            return;
        }

        let tile_height = ((viewport.height - self.padding) / stack.len() as u32) - self.padding;

        for (i, window) in stack.enumerate() {
            window.without_tracking(|window| {
                window.map();
                window.configure(
                    viewport.x + self.padding,
                    viewport.y + self.padding + (i as u32 * (tile_height + self.padding)),
                    viewport.width - (self.padding * 2),
                    tile_height,
                );
            });
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

    fn layout(&self, viewport: &Viewport, focused: Option<GroupWindow>, stack: GroupIter) {
        if stack.len() == 0 {
            return;
        }

        {
            let unfocused = stack.filter(|window| {
                focused
                    .as_ref()
                    .map_or(true, |focused_window| window.id() != focused_window.id())
            });
            for window in unfocused {
                window.without_tracking(|window| window.unmap());
            }
        }
        focused.map(|window| {
            window.without_tracking(|window| {
                window.map();
                window.configure(
                    viewport.x + self.padding,
                    viewport.y + self.padding,
                    viewport.width - (self.padding * 2),
                    viewport.height - (self.padding * 2),
                );
            })
        });
    }
}
