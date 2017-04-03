use std::rc::Rc;
use std::slice::IterMut;

use window::Window;
use x::{Connection, WindowId};


pub struct Group {
    connection: Rc<Connection>,
    stack: Vec<WindowId>,
    focus: Option<usize>,
}

impl Group {
    pub fn new(connection: Rc<Connection>) -> Group {
        Group {
            connection: connection,
            stack: Vec::new(),
            focus: None,
        }
    }

    pub fn add_window(&mut self, window_id: WindowId) {
        self.stack.push(window_id);
    }

    pub fn find_window_by_id<'a>(&'a mut self, window_id: WindowId) -> Option<GroupWindow<'a>> {
        self.stack
            .iter()
            .position(|w| *w == window_id)
            .map(move |idx| {
                     GroupWindow {
                         group: self,
                         idx: idx,
                     }
                 })
    }

    pub fn get_focused<'a>(&'a mut self) -> Option<GroupWindow<'a>> {
        self.focus.map(move |idx| {
                           GroupWindow {
                               group: self,
                               idx: idx,
                           }
                       })
    }

    pub fn iter_mut<'a>(&'a mut self) -> GroupIter<'a> {
        GroupIter {
            connection: &self.connection,
            inner: self.stack.iter_mut(),
        }
    }
}


pub struct GroupWindow<'a> {
    group: &'a mut Group,
    idx: usize,
}

impl<'a> GroupWindow<'a> {
    pub fn remove_from_group(self) {
        self.group.stack.remove(self.idx);
    }

    pub fn focus(&mut self) {
        self.group.focus = Some(self.idx);
    }
}

impl<'a> Window for GroupWindow<'a> {
    fn connection(&self) -> &Connection {
        &self.group.connection
    }

    fn id(&self) -> &WindowId {
        &self.group.stack[self.idx]
    }
}


pub struct GroupIter<'a> {
    connection: &'a Connection,
    inner: IterMut<'a, WindowId>,
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
