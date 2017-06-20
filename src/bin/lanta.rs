#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lanta;

use std::process::Command;

use lanta::{cmd, Lanta, ModKey};
use lanta::errors::*;
use lanta::layout::*;


fn run() -> Result<()> {
    lanta::intiailize_logger()?;

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

    Lanta::new(keys, groups, layouts)?.run();

    Ok(())
}

quick_main!(run);
