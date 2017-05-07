use std::fmt;

use groups::{GroupIter, GroupWindow};
use window::Window;


pub trait LayoutClone {
    fn clone_box(&self) -> Box<Layout>;
}

impl<T> LayoutClone for T
    where T: 'static + Layout + Clone
{
    fn clone_box(&self) -> Box<Layout> {
        Box::new(self.clone())
    }
}


pub trait Layout: LayoutClone {
    fn name(&self) -> &str;
    fn layout(&self, width: i32, height: i32, focused: Option<GroupWindow>, stack: GroupIter);
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

    fn layout(&self, width: i32, height: i32, _: Option<GroupWindow>, stack: GroupIter) {
        if stack.len() == 0 {
            return;
        }

        let tile_height = height / stack.len() as i32;

        for (i, window) in stack.enumerate() {
            window.without_focus_tracking(|window| {
                                              window.map();
                                              window.configure(0,
                                                               i as i32 * tile_height,
                                                               width,
                                                               tile_height);
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

    fn layout(&self, width: i32, height: i32, focused: Option<GroupWindow>, stack: GroupIter) {
        if stack.len() == 0 {
            return;
        }


        for window in stack {
            window.without_focus_tracking(|window| window.unmap());
        }
        focused.map(|window| {
                        window.without_focus_tracking(|window| {
                                                          window.map();
                                                          window.configure(0, 0, width, height);
                                                      })
                    });
    }
}
