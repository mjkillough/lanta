use std::rc::{Rc, Weak};
use std::slice::IterMut;


pub struct Stack<T>
    where T: PartialEq
{
    vec: Vec<Rc<T>>,
    focus: Option<Weak<T>>,
}

impl<T> Stack<T>
    where T: PartialEq
{
    pub fn new() -> Stack<T> {
        Stack {
            vec: Vec::new(),
            focus: None,
        }
    }

    pub fn push(&mut self, value: T) {
        self.vec.push(Rc::new(value));
    }

    pub fn remove(&mut self, value: &T) {
        self.vec
            .iter()
            .position(|rc| rc.as_ref() == value)
            .map(|index| self.vec.remove(index));
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, Rc<T>> {
        self.vec.iter_mut()
    }

    pub fn find_by_value(&self, value: &T) -> Option<Rc<T>> {
        self.vec
            .iter()
            .find(|rc| rc.as_ref() == value)
            .map(|rc| rc.clone())
    }

    pub fn get_focused(&self) -> Option<Rc<T>> {
        self.focus.clone().and_then(|rc| rc.upgrade())
    }

    pub fn set_focus(&mut self, rc: &Rc<T>) {
        self.focus = Some(Rc::downgrade(&rc));
    }

    pub fn focus_next(&mut self) {
        self.focus = self.focus
            .clone()
            .and_then(|rc| rc.upgrade())
            .and_then(|current| self.vec.iter().position(|rc| rc == &current))
            .map(|current_idx| {
                     let next_idx = (current_idx + 1) % self.vec.len();
                     self.vec[next_idx].clone()
                 })
            .or_else(|| if self.vec.is_empty() {
                         None
                     } else {
                         Some(self.vec[0].clone())
                     })
            .map(|rc| Rc::downgrade(&rc));
    }

    pub fn focus_previous(&mut self) {
        self.focus = self.focus
            .clone()
            .and_then(|rc| rc.upgrade())
            .and_then(|current| self.vec.iter().position(|rc| rc == &current))
            .map(|current_idx| {
                let next_idx = if current_idx == 0 {
                    self.vec.len() - 1
                } else {
                    (current_idx - 1) % self.vec.len()
                };
                self.vec[next_idx].clone()
            })
            .or_else(|| if self.vec.is_empty() {
                         None
                     } else {
                         Some(self.vec[0].clone())
                     })
            .map(|rc| Rc::downgrade(&rc));
    }

    pub fn shuffle_next(&mut self, value: &T) {
        self.vec
            .iter()
            .position(|w| w.as_ref() == value)
            .map(|current_idx| {
                     let next_idx = (current_idx + 1) % self.vec.len();
                     self.vec.swap(current_idx, next_idx);
                 });
    }

    pub fn shuffle_previous(&mut self, value: &T) {
        self.vec
            .iter()
            .position(|w| w.as_ref() == value)
            .map(|current_idx| {
                let previous_idx = if current_idx == 0 {
                    self.vec.len() - 1
                } else {
                    (current_idx - 1) % self.vec.len()
                };
                self.vec.swap(current_idx, previous_idx);
            });
    }
}
