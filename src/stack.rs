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
        self.vec.retain(|rc| rc.as_ref() != value);
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

    pub fn set_focus(&mut self, item: Option<&Rc<T>>) {
        self.focus = item.map(|rc| Rc::downgrade(rc));
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
            .map(|current_idx| if current_idx == (self.vec.len() - 1) {
                     let item = self.vec.remove(current_idx);
                     self.vec.insert(0, item);
                 } else {
                     let next_idx = current_idx + 1;
                     self.vec.swap(current_idx, next_idx);
                 });
    }

    pub fn shuffle_previous(&mut self, value: &T) {
        self.vec
            .iter()
            .position(|w| w.as_ref() == value)
            .map(|current_idx| if current_idx == 0 {
                     let item = self.vec.remove(0);
                     self.vec.push(item);
                 } else {
                     let previous_idx = current_idx - 1;
                     self.vec.swap(current_idx, previous_idx);
                 });
    }
}


#[cfg(test)]
mod test {

    use std::fmt::Debug;
    use std::rc::Rc;

    use super::Stack;

    fn assert_stack_contents<T>(stack: &Stack<T>, vec: Vec<T>)
        where T: Copy + Debug + PartialEq
    {
        let vec: Vec<Rc<T>> = vec.iter().map(|v| Rc::new(*v)).collect();
        assert_eq!(stack.vec, vec);
    }

    #[test]
    fn test_push() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        assert_stack_contents(&stack, vec![2]);
    }

    #[test]
    fn test_remove() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(3);
        stack.push(4);
        stack.remove(&3);
        assert_stack_contents(&stack, vec![2, 4]);
    }

    #[test]
    fn test_iter_mut() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        let mut iter = stack.iter_mut();
        assert_eq!(iter.next(), Some(&mut Rc::new(2)));
        assert_eq!(iter.next(), Some(&mut Rc::new(3)));
        assert_eq!(iter.next(), Some(&mut Rc::new(4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_find_by_value() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        assert_eq!(stack.find_by_value(&2), Some(Rc::new(2)));
        assert_eq!(stack.find_by_value(&3), None);
    }

    #[test]
    fn test_set_get_focus() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.get_focused(), None);
        {
            // Extra scope is important: two holds a reference to the Rc, which must go out of
            // scope before .get_focused() returns None. (Sad).
            let two = stack.find_by_value(&2).unwrap();
            stack.set_focus(Some(&two));
            assert_eq!(stack.get_focused(), Some(Rc::new(2)));
        }
        stack.remove(&2);
        assert_eq!(stack.get_focused(), None);
    }

    #[test]
    fn test_focus_next() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);

        // If nothing is focused, focus_next() should pick the first item in the stack.
        assert_eq!(stack.get_focused(), None);
        stack.focus_next();
        assert_eq!(stack.get_focused(), Some(Rc::new(2)));
        stack.focus_next();
        assert_eq!(stack.get_focused(), Some(Rc::new(3)));
        stack.focus_next();
        assert_eq!(stack.get_focused(), Some(Rc::new(4)));
        stack.focus_next();
        assert_eq!(stack.get_focused(), Some(Rc::new(2)));
    }

    #[test]
    fn test_focus_previous() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);

        // If nothing is focused, focus_previous() should pick the first item in the stack.
        assert_eq!(stack.get_focused(), None);
        stack.focus_previous();
        assert_eq!(stack.get_focused(), Some(Rc::new(2)));
        stack.focus_previous();
        assert_eq!(stack.get_focused(), Some(Rc::new(4)));
        stack.focus_previous();
        assert_eq!(stack.get_focused(), Some(Rc::new(3)));
        stack.focus_previous();
        assert_eq!(stack.get_focused(), Some(Rc::new(2)));
    }

    #[test]
    fn test_shuffle_next() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);

        assert_stack_contents(&stack, vec![2, 3, 4]);
        stack.shuffle_next(&2);
        assert_stack_contents(&stack, vec![3, 2, 4]);
        stack.shuffle_next(&2);
        assert_stack_contents(&stack, vec![3, 4, 2]);
        stack.shuffle_next(&2);
        assert_stack_contents(&stack, vec![2, 3, 4]);
    }

    #[test]
    fn test_shuffle_previous() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);

        assert_stack_contents(&stack, vec![2, 3, 4]);
        stack.shuffle_previous(&2);
        assert_stack_contents(&stack, vec![3, 4, 2]);
        stack.shuffle_previous(&2);
        assert_stack_contents(&stack, vec![3, 2, 4]);
        stack.shuffle_previous(&2);
        assert_stack_contents(&stack, vec![2, 3, 4]);
    }
}
