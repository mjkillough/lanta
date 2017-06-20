#[macro_use]
extern crate log;
extern crate x11;

#[macro_use]
extern crate lanta;

use std::process::Command;

use lanta::cmd;
use lanta::RustWindowManager;
use lanta::groups::GroupBuilder;
use lanta::keys::ModKey;
use lanta::layout::{StackLayout, TiledLayout};
pub use x11::keysym;


fn main() {
    lanta::intiailize_logger();

    let modkey = ModKey::Control;
    let shift = ModKey::Shift;
    let mut keys = keys![
        ([modkey], XK_w, cmd::lazy::close_focused_window()),
        ([modkey], XK_j, cmd::lazy::focus_next()),
        ([modkey], XK_k, cmd::lazy::focus_previous()),
        ([modkey, shift], XK_j, cmd::lazy::shuffle_next()),
        ([modkey, shift], XK_k, cmd::lazy::shuffle_previous()),
        ([modkey], XK_Tab, cmd::lazy::layout_next()),
        ([modkey], XK_Return, cmd::lazy::spawn(Command::new("urxvt"))),
        ([modkey], XK_c, cmd::lazy::spawn(Command::new("chrome"))),
        ([modkey], XK_v, cmd::lazy::spawn(Command::new("code"))),
        (
            [modkey],
            XK_q,
            cmd::lazy::spawn(Command::new("change-wallpaper"))
        ),
    ];

    let padding = 20;
    let layouts = vec![
        StackLayout::new("stack-padded", padding),
        StackLayout::new("stack", 0),
        TiledLayout::new("tiled", padding),
    ];

    let groups = groups!{
        keys,
        [
            ([modkey], XK_a, "chrome", "stack"),
            ([modkey], XK_s, "code", "stack"),
            ([modkey], XK_d, "term", "tiled"),
            ([modkey], XK_f, "misc", "tiled"),
        ]
    };

    let mut wm = RustWindowManager::new(keys, groups, layouts).unwrap();
    info!("Started WM, entering event loop.");
    wm.run_event_loop();
}
