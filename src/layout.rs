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
}

impl TiledLayout {
    pub fn new(name: String) -> Box<Layout> {
        Box::new(TiledLayout { name })
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

        let tile_height = viewport.height / stack.len() as u32;

        for (i, window) in stack.enumerate() {
            window.without_focus_tracking(|window| {
                window.map();
                window.configure(
                    viewport.x,
                    viewport.y + (i as u32 * tile_height),
                    viewport.width,
                    tile_height,
                );
            });
        }
    }
}


#[derive(Clone)]
pub struct StackLayout {
    name: String,
}

impl StackLayout {
    pub fn new(name: String) -> Box<Layout> {
        Box::new(StackLayout { name })
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
                window.without_focus_tracking(|window| window.unmap());
            }
        }
        focused.map(|window| {
            window.without_focus_tracking(|window| {
                window.map();
                window.configure(viewport.x, viewport.y, viewport.width, viewport.height);
            })
        });
    }
}
