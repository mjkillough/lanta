use std::slice::{Iter, IterMut};


#[derive(Clone)]
pub struct Stack<T> {
    vec: Vec<T>,
    focus: Option<usize>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack {
            vec: Vec::new(),
            focus: None,
        }
    }

    pub fn push(&mut self, value: T) {
        self.vec.push(value);
        self.ensure_focus();
    }

    pub fn remove<P>(&mut self, p: P) -> T
        where P: FnMut(&T) -> bool
    {
        let position = self.vec
            .iter()
            .position(p)
            .expect("No element in stack matches predicate");
        let element = self.vec.remove(position);
        // We might have removed the focus element. A non-empty stack should always
        // focus something.
        self.ensure_focus();
        element
    }

    pub fn iter(&self) -> Iter<T> {
        self.vec.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.vec.iter_mut()
    }

    pub fn focused(&self) -> Option<&T> {
        self.focus.and_then(|idx| self.vec.get(idx))
    }

    pub fn focused_mut(&mut self) -> Option<&mut T> {
        self.focus.and_then(move |idx| self.vec.get_mut(idx))
    }

    /// Focuses the first element in the stack that matches the predicate.
    ///
    /// Panics if no element matches the predicate.
    pub fn focus<P>(&mut self, p: P)
        where P: FnMut(&T) -> bool
    {
        let position = self.vec
            .iter()
            .position(p)
            .expect("No element in stack matches predicate");
        self.focus = Some(position);
    }

    fn ensure_focus(&mut self) {
        self.focus = self.focus
            .or(if self.vec.is_empty() { None } else { Some(0) });
    }

    fn next_index(&self, index: usize) -> usize {
        (index + 1) % self.vec.len()
    }

    fn previous_index(&self, index: usize) -> usize {
        if index == 0 {
            self.vec.len() - 1
        } else {
            (index - 1) % self.vec.len()
        }
    }

    pub fn focus_next(&mut self) {
        self.focus = self.focus.map(|idx| self.next_index(idx));
    }

    pub fn focus_previous(&mut self) {
        self.focus = self.focus.map(|idx| self.previous_index(idx));
    }

    pub fn shuffle_next(&mut self) {
        if let Some(current_idx) = self.focus {
            let next_idx = self.next_index(current_idx);
            if next_idx == 0 {
                let item = self.vec.remove(current_idx);
                self.vec.insert(0, item);
            } else {
                self.vec.swap(current_idx, next_idx);
            }
            self.focus = Some(next_idx);
        }
    }

    pub fn shuffle_previous(&mut self) {
        if let Some(current_idx) = self.focus {
            let prev_idx = self.previous_index(current_idx);
            if prev_idx == self.vec.len() - 1 {
                let item = self.vec.remove(current_idx);
                self.vec.push(item);
            } else {
                self.vec.swap(current_idx, prev_idx);
            }
            self.focus = Some(prev_idx);
        }
    }
}

impl<T> From<Vec<T>> for Stack<T> {
    fn from(vec: Vec<T>) -> Self {
        let mut stack = Stack {
            vec: vec,
            focus: None,
        };
        stack.ensure_focus();
        stack
    }
}


#[cfg(test)]
mod test {


    use super::Stack;
    use std::fmt::Debug;
    use std::rc::Rc;


    #[test]
    fn test_push() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        assert_eq!(stack.vec, vec![2]);
    }

    #[test]
    fn test_remove() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        stack.remove(|v| v == &3);
        assert_eq!(stack.vec, vec![2, 4]);
    }

    #[test]
    fn test_iter() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        let mut iter = stack.iter();
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_set_get_focus() {
        let mut stack = Stack::<u8>::new();
        assert_eq!(stack.focused(), None);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.focused(), Some(&2));
        stack.remove(|v| v == &2);
        // A non-empty stack should always have something focused.
        assert_eq!(stack.focused(), Some(&3));
        stack.remove(|v| v == &3);
        assert_eq!(stack.focused(), None);
    }

    #[test]
    fn test_focus_next() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        assert_eq!(stack.focused(), Some(&2));

        stack.focus_next();
        assert_eq!(stack.focused(), Some(&3));
        stack.focus_next();
        assert_eq!(stack.focused(), Some(&4));
        stack.focus_next();
        assert_eq!(stack.focused(), Some(&2));
    }

    #[test]
    fn test_focus_previous() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        assert_eq!(stack.focused(), Some(&2));

        stack.focus_previous();
        assert_eq!(stack.focused(), Some(&4));
        stack.focus_previous();
        assert_eq!(stack.focused(), Some(&3));
        stack.focus_previous();
        assert_eq!(stack.focused(), Some(&2));
    }

    #[test]
    fn test_shuffle_next() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        assert_eq!(stack.focused(), Some(&2));

        assert_eq!(stack.vec, vec![2, 3, 4]);
        stack.shuffle_next();
        assert_eq!(stack.vec, vec![3, 2, 4]);
        stack.shuffle_next();
        assert_eq!(stack.vec, vec![3, 4, 2]);
        stack.shuffle_next();
        assert_eq!(stack.vec, vec![2, 3, 4]);
    }

    #[test]
    fn test_shuffle_previous() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        assert_eq!(stack.focused(), Some(&2));

        assert_eq!(stack.vec, vec![2, 3, 4]);
        stack.shuffle_previous();
        assert_eq!(stack.vec, vec![3, 4, 2]);
        stack.shuffle_previous();
        assert_eq!(stack.vec, vec![3, 2, 4]);
        stack.shuffle_previous();
        assert_eq!(stack.vec, vec![2, 3, 4]);
    }
}
