use std::rc::Rc;
use std::slice::IterMut;

use stack::Stack;
use window::Window;
use x::{Connection, WindowId};


pub struct Group {
    connection: Rc<Connection>,
    stack: Stack<WindowId>,
}

impl Group {
    pub fn new(connection: Rc<Connection>) -> Group {
        Group {
            connection: connection,
            stack: Stack::new(),
        }
    }

    pub fn add_window(&mut self, window_id: WindowId) {
        self.stack.push(window_id);
    }

    fn remove_window(&mut self, window_id: &WindowId) {
        self.stack.remove(window_id);
    }

    pub fn find_window_by_id<'a>(&'a mut self, window_id: &WindowId) -> Option<GroupWindow<'a>> {
        self.stack.find_by_value(window_id).map(move |rc| {
                                                    GroupWindow {
                                                        group: self,
                                                        window_id: rc,
                                                    }
                                                })
    }

    pub fn iter_mut<'a>(&'a mut self) -> GroupIter<'a> {
        GroupIter {
            connection: &self.connection,
            inner: self.stack.iter_mut(),
        }
    }

    pub fn get_focused<'a>(&'a mut self) -> Option<GroupWindow<'a>> {
        self.stack.get_focused().map(move |rc| {
                                         GroupWindow {
                                             group: self,
                                             window_id: rc,
                                         }
                                     })
    }

    fn apply_focus_to_window(&mut self) {
        self.stack.get_focused().map(|window_id| self.connection.focus_window(&window_id));
    }

    pub fn focus_next(&mut self) {
        self.stack.focus_next();
        self.apply_focus_to_window();
    }

    pub fn focus_previous(&mut self) {
        self.stack.focus_previous();
        self.apply_focus_to_window();
    }

    pub fn shuffle_next(&mut self, window_id: &WindowId) {
        self.stack.shuffle_next(window_id);
    }

    pub fn shuffle_previous(&mut self, window_id: &WindowId) {
        self.stack.shuffle_previous(window_id);
    }
}


pub struct GroupWindow<'a> {
    group: &'a mut Group,
    window_id: Rc<WindowId>,
}

impl<'a> GroupWindow<'a> {
    pub fn remove_from_group(self) -> WindowId {
        self.group.remove_window(&self.window_id);
        Rc::try_unwrap(self.window_id)
            .expect("Dangling reference to WindowId after removal from group")
    }

    pub fn focus(&mut self) {
        self.group.stack.set_focus(Some(&self.window_id));
        self.group.apply_focus_to_window();
    }

    pub fn shuffle_next(&mut self) {
        self.group.shuffle_next(self.window_id.as_ref());
    }

    pub fn shuffle_previous(&mut self) {
        self.group.shuffle_previous(self.window_id.as_ref());
    }
}

impl<'a> Window for GroupWindow<'a> {
    fn connection(&self) -> &Connection {
        &self.group.connection
    }

    fn id(&self) -> &WindowId {
        self.window_id.as_ref()
    }
}


pub struct GroupIter<'a> {
    connection: &'a Connection,
    inner: IterMut<'a, Rc<WindowId>>,
}

impl<'a> Iterator for GroupIter<'a> {
    type Item = GroupIterItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|window_id| {
                                  GroupIterItem {
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


pub struct GroupIterItem<'a> {
    connection: &'a Connection,
    window_id: &'a WindowId,
}

impl<'a> Window for GroupIterItem<'a> {
    fn connection(&self) -> &Connection {
        self.connection
    }

    fn id(&self) -> &WindowId {
        self.window_id
    }
}
