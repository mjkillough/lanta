use std::rc::Rc;

use crate::Lanta;
use crate::Result;

pub type Command = Rc<dyn Fn(&mut Lanta) -> Result<()>>;

/// Lazy-functions which return a `Command` to do the requested action.
// TODO: Consider offering non-lazy versions and then having simple lazy
// wrappers for them.
pub mod lazy {

    use std::process;
    use std::rc::Rc;
    use std::sync::Mutex;

    use failure::ResultExt;

    use super::Command;

    /// Closes the currently focused window.
    pub fn close_focused_window() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().close_focused();
            Ok(())
        })
    }

    /// Moves the focus to the next window in the current group's stack.
    pub fn focus_next() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().focus_next();
            Ok(())
        })
    }

    /// Moves the focus to the previous window in the current group's stack.
    pub fn focus_previous() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().focus_previous();
            Ok(())
        })
    }

    /// Shuffles the focused window to the next position in the current group's
    /// stack.
    pub fn shuffle_next() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().shuffle_next();
            Ok(())
        })
    }

    /// Shuffles the focused window to the previous position in the current
    /// group's stack.
    pub fn shuffle_previous() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().shuffle_previous();
            Ok(())
        })
    }

    /// Cycles to the next layout of the current group.
    pub fn layout_next() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().layout_next();
            Ok(())
        })
    }

    /// Spawns the specified command.
    ///
    /// The returned `Command` will spawn the `Command` each time it is called.
    pub fn spawn(command: process::Command) -> Command {
        let mutex = Mutex::new(command);
        Rc::new(move |_| {
            let mut command = mutex.lock().unwrap();
            info!("Spawning: {:?}", *command);
            command
                .spawn()
                .with_context(|_| format!("Could not spawn command: {:?}", *command))?;
            Ok(())
        })
    }

    /// Switches to the group specified by name.
    pub fn switch_group(name: &'static str) -> Command {
        Rc::new(move |wm| {
            wm.switch_group(name);
            Ok(())
        })
    }

    /// Moves the focused window on the active group to another group.
    pub fn move_window_to_group(name: &'static str) -> Command {
        Rc::new(move |wm| {
            wm.move_focused_to_group(name);
            Ok(())
        })
    }
}
