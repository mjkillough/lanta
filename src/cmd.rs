use std::rc::Rc;

use super::RustWindowManager;


pub type Command = Rc<Fn(&mut RustWindowManager)>;


/// Lazy-functions which return a `Command` to do the requested action.
// TODO: Consider offering non-lazy versions and then having simple lazy
// wrappers for them.
pub mod lazy {

    use std::process;
    use std::rc::Rc;
    use std::sync::Mutex;

    use super::Command;
    use window::Window;

    /// Closes the currently focused window.
    pub fn close_focused_window() -> Command {
        Rc::new(|ref mut wm| { wm.group_mut().get_focused().map(|w| w.close()); })
    }

    /// Moves the focus to the next window in the current group's stack.
    pub fn focus_next() -> Command {
        Rc::new(|ref mut wm| { wm.group_mut().focus_next(); })
    }

    /// Moves the focus to the previous window in the current group's stack.
    pub fn focus_previous() -> Command {
        Rc::new(|ref mut wm| { wm.group_mut().focus_previous(); })
    }

    /// Shuffles the focused window to the next position in the current group's
    /// stack.
    pub fn shuffle_next() -> Command {
        Rc::new(|ref mut wm| { wm.group_mut().shuffle_next(); })
    }

    /// Shuffles the focused window to the previous position in the current
    /// group's stack.
    pub fn shuffle_previous() -> Command {
        Rc::new(|ref mut wm| { wm.group_mut().shuffle_previous(); })
    }

    /// Cycles to the next layout of the current group.
    pub fn layout_next() -> Command {
        Rc::new(|ref mut wm| { wm.group_mut().layout_next(); })
    }

    /// Spawns the specified command.
    ///
    /// The returned `Command` will spawn the `Command` each time it is called.
    pub fn spawn(command: process::Command) -> Command {
        let mutex = Mutex::new(command);
        Rc::new(move |_| {
                    let mut command = mutex.lock().unwrap();
                    info!("Spawning: {:?}", *command);
                    command.spawn().unwrap();
                })
    }

    /// Switches to the group specified by name.
    pub fn switch_group(name: &'static str) -> Command {
        Rc::new(move |wm| wm.switch_group(name))
    }

    /// Moves the focused window on the active group to another group.
    pub fn move_window_to_group(name: &'static str) -> Command {
        Rc::new(move |wm| wm.move_focused_to_group(name))
    }
}
