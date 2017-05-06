use layout::{Layout, StackLayout, TiledLayout};

use stack::Stack;
use std::rc::Rc;
use std::slice::Iter;
use window::Window;
use x::{Connection, WindowId};


pub struct Group {
    name: String,
    connection: Rc<Connection>,
    stack: Stack<WindowId>,
    layouts: Stack<Box<Layout>>,
}

impl Group {
    pub fn new<S>(name: S, connection: Rc<Connection>) -> Group
        where S: Into<String>
    {
        let mut layouts = Stack::new();
        layouts.push(Box::new(StackLayout {}) as Box<Layout>);
        layouts.push(Box::new(TiledLayout {}) as Box<Layout>);

        Group {
            name: name.into(),
            connection: connection,
            stack: Stack::new(),
            layouts: layouts,
        }
    }

    fn layout(&mut self) {
        let (width, height) = self.connection
            .get_window_geometry(&self.connection.root_window_id());

        let focused = self.stack
            .focused()
            .map(|window_id| {
                     GroupWindow {
                         connection: &self.connection,
                         window_id: &window_id,
                     }
                 });

        self.layouts
            .focused()
            .map(|l| l.layout(width, height, focused, self.iter()));
    }

    // pub fn activate
    // pub fn deactivate

    pub fn add_window(&mut self, window_id: WindowId) {
        self.stack.push(window_id);
        self.layout();
    }

    pub fn remove_window(&mut self, window_id: &WindowId) -> WindowId {
        let removed = self.stack.remove(|w| w == window_id);
        self.layout();
        removed
    }

    pub fn focus(&mut self, window_id: &WindowId) {
        self.stack.focus(|id| id == window_id);
        self.apply_focus_to_window();
    }

    fn iter<'a>(&'a self) -> GroupIter<'a> {
        GroupIter {
            connection: &self.connection,
            inner: self.stack.iter(),
        }
    }

    pub fn get_focused<'a>(&'a self) -> Option<GroupWindow<'a>> {
        self.stack
            .focused()
            .map(move |ref id| {
                     GroupWindow {
                         connection: &self.connection,
                         window_id: &id,
                     }
                 })
    }

    fn apply_focus_to_window(&mut self) {
        self.stack
            .focused()
            .map(|window_id| self.connection.focus_window(&window_id));
        self.layout();
    }

    pub fn focus_next(&mut self) {
        self.stack.focus_next();
        self.apply_focus_to_window();
    }

    pub fn focus_previous(&mut self) {
        self.stack.focus_previous();
        self.apply_focus_to_window();
    }

    pub fn shuffle_next(&mut self) {
        self.stack.shuffle_next();
        self.layout();
    }

    pub fn shuffle_previous(&mut self) {
        self.stack.shuffle_previous();
        self.layout();
    }

    pub fn layout_next(&mut self) {
        self.layouts.focus_next();
        self.layout();
    }

    pub fn layout_previous(&mut self) {
        self.layouts.focus_previous();
        self.layout();
    }
}


pub struct GroupIter<'a> {
    connection: &'a Connection,
    inner: Iter<'a, WindowId>,
}

impl<'a> Iterator for GroupIter<'a> {
    type Item = GroupWindow<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|window_id| {
                     GroupWindow {
                         connection: self.connection,
                         window_id: window_id,
                     }
                 })
    }
}

impl<'a> ExactSizeIterator for GroupIter<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}


pub struct GroupWindow<'a> {
    connection: &'a Connection,
    window_id: &'a WindowId,
}

impl<'a> Window for GroupWindow<'a> {
    fn connection(&self) -> &Connection {
        self.connection
    }

    fn id(&self) -> &WindowId {
        self.window_id
    }
}
