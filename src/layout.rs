use std::rc::Rc;
use std::slice::IterMut;

use window::Window;


pub trait Layout {
    fn layout(&self, width: i32, height: i32, stack: IterMut<Rc<Window>>);
}


pub struct TiledLayout;

impl Layout for TiledLayout {
    fn layout(&self, width: i32, height: i32, stack: IterMut<Rc<Window>>) {
        if stack.len() == 0 {
            return;
        }

        let tile_height = height / stack.len() as i32;

        for (i, window) in stack.enumerate() {
            window.configure(0, i as i32 * tile_height, width, tile_height);
        }
    }
}
