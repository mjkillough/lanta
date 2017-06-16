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

    /// Returns the number of elements in the stack.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Adds an element to the stack (at the end) and focuses it.
    pub fn push(&mut self, value: T) {
        self.vec.push(value);
        self.focus = Some(self.vec.len() - 1);
    }

    pub fn remove<P>(&mut self, p: P) -> T
    where
        P: FnMut(&T) -> bool,
    {
        let position = self.vec
            .iter()
            .position(p)
            .expect("No element in stack matches predicate");
        let element = self.vec.remove(position);
        // Focus might now be pointing at the wrong element. If we
        self.fix_focus_after_removal(position);
        element
    }

    /// Removes the focused element (if any) and returns it.
    pub fn remove_focused(&mut self) -> Option<T> {
        let element = self.focus.map(|idx| self.vec.remove(idx));
        if let Some(removed_idx) = self.focus {
            self.fix_focus_after_removal(removed_idx);
        }
        element
    }

    /// Updates the focus index into the vector, as it may be pointing at the
    /// wrong element after a removal.
    fn fix_focus_after_removal(&mut self, removed_idx: usize) {
        self.focus = self.focus
            .and_then(|idx| if self.vec.is_empty() { None } else { Some(idx) })
            .map(|idx| {
                if idx > removed_idx || (idx == removed_idx && idx == self.vec.len()) {
                    idx - 1
                } else {
                    idx
                }
            });
    }

    pub fn iter(&self) -> Iter<T> {
        self.vec.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.vec.iter_mut()
    }

    pub fn focused(&self) -> Option<&T> {
        self.focus.and_then(|idx| {
            let item = self.vec.get(idx);
            // `focus` should always point to a valid element of `vec`. If not, we've
            // broken an invariant. Allow execution to continue, as it should be possible
            // for the user to work around this in most cases.
            if item.is_none() {
                error!(
                    "Stack's focus index ({}) is greater than it's length ({})",
                    idx,
                    self.vec.len()
                );
            }
            item
        })
    }

    pub fn focused_mut(&mut self) -> Option<&mut T> {
        self.focus.and_then(move |idx| {
            let item = self.vec.get_mut(idx);
            // `focus` should always point to a valid element of `vec`. If not, we've
            // broken an invariant. Allow execution to continue, as it should be possible
            // for the user to work around this in most cases.
            if item.is_none() {
                error!("Stack's focus index ({}) is greater than it's length", idx);
            }
            item
        })
    }

    /// Focuses the first element in the stack that matches the predicate.
    ///
    /// Panics if no element matches the predicate.
    pub fn focus<P>(&mut self, p: P)
    where
        P: FnMut(&T) -> bool,
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

    #[test]
    fn test_from() {
        let vec = vec![1, 2, 3];
        let stack = Stack::from(vec.clone());
        assert_eq!(stack.vec, vec);
        assert_eq!(stack.focused(), Some(&vec[0]));
    }

    #[test]
    fn test_push() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        assert_eq!(stack.vec, vec![2]);
        assert_eq!(stack.focused(), Some(&2));
        stack.push(3);
        assert_eq!(stack.focused(), Some(&3));
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
    fn test_remove_updates_focus() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        stack.focus_previous();
        assert_eq!(stack.focused(), Some(&3));

        // Remove element before focused element.
        let mut stack1 = stack.clone();
        stack1.remove(|v| v == &2);
        assert_eq!(stack1.focused(), Some(&3));

        // Remove focused element.
        let mut stack2 = stack.clone();
        stack2.remove(|v| v == &3);
        assert_eq!(stack2.focused(), Some(&4));

        // Remove element after focused element.
        let mut stack3 = stack.clone();
        stack3.remove(|v| v == &4);
        assert_eq!(stack3.focused(), Some(&3));

        // Remove focused element where it is also the final element.
        let mut stack4 = stack.clone();
        stack4.focus_next();
        assert_eq!(stack4.focused(), Some(&4));
        stack4.remove(|v| v == &4);
        assert_eq!(stack4.focused(), Some(&3));

        // Remove only element.
        let mut stack5 = Stack::<u8>::new();
        stack5.push(2);
        assert_eq!(stack5.focused(), Some(&2));
        stack5.remove(|v| v == &2);
        assert_eq!(stack5.focus, None);
    }

    #[test]
    fn test_remove_focused() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.focused(), Some(&3));

        let element = stack.remove_focused();
        assert_eq!(element, Some(3));
        assert_eq!(stack.focused(), Some(&2));
        assert_eq!(stack.vec, vec![2]);
    }

    #[test]
    fn test_remove_focused_when_no_focus() {
        let mut stack = Stack::<u8>::new();
        assert_eq!(stack.focused(), None);
        let element = stack.remove_focused();
        assert_eq!(element, None);
        assert_eq!(stack.focused(), None);
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
        assert_eq!(stack.focused(), Some(&3));
        stack.remove(|v| v == &3);
        // A non-empty stack should always have something focused.
        assert_eq!(stack.focused(), Some(&2));
        stack.remove(|v| v == &2);
        assert_eq!(stack.focused(), None);
    }

    #[test]
    fn test_focus_next() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        assert_eq!(stack.focused(), Some(&4));

        stack.focus_next();
        assert_eq!(stack.focused(), Some(&2));
        stack.focus_next();
        assert_eq!(stack.focused(), Some(&3));
        stack.focus_next();
        assert_eq!(stack.focused(), Some(&4));
    }

    #[test]
    fn test_focus_previous() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        assert_eq!(stack.focused(), Some(&4));

        stack.focus_previous();
        assert_eq!(stack.focused(), Some(&3));
        stack.focus_previous();
        assert_eq!(stack.focused(), Some(&2));
        stack.focus_previous();
        assert_eq!(stack.focused(), Some(&4));
    }

    #[test]
    fn test_shuffle_next() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        assert_eq!(stack.focused(), Some(&4));

        assert_eq!(stack.vec, vec![2, 3, 4]);
        stack.shuffle_next();
        assert_eq!(stack.vec, vec![4, 2, 3]);
        stack.shuffle_next();
        assert_eq!(stack.vec, vec![2, 4, 3]);
        stack.shuffle_next();
        assert_eq!(stack.vec, vec![2, 3, 4]);
    }

    #[test]
    fn test_shuffle_previous() {
        let mut stack = Stack::<u8>::new();
        stack.push(2);
        stack.push(3);
        stack.push(4);
        assert_eq!(stack.focused(), Some(&4));

        assert_eq!(stack.vec, vec![2, 3, 4]);
        stack.shuffle_previous();
        assert_eq!(stack.vec, vec![2, 4, 3]);
        stack.shuffle_previous();
        assert_eq!(stack.vec, vec![4, 2, 3]);
        stack.shuffle_previous();
        assert_eq!(stack.vec, vec![2, 3, 4]);
    }
}
