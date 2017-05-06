extern crate env_logger;
extern crate x11;

extern crate lanta;

use std::process::Command;

use lanta::cmd;
use lanta::RustWindowManager;
use lanta::groups::GroupBuilder;
use lanta::keys::ModKey;
use lanta::layout::{StackLayout, TiledLayout};
use x11::keysym;


fn main() {
    env_logger::init().unwrap();

    let modkey = ModKey::Mod4;
    let mut keys =
        vec![(vec![modkey], keysym::XK_t, cmd::lazy::close_focused_window()),
             (vec![modkey], keysym::XK_y, cmd::lazy::focus_next()),
             (vec![modkey], keysym::XK_u, cmd::lazy::focus_previous()),
             (vec![modkey], keysym::XK_i, cmd::lazy::shuffle_next()),
             (vec![modkey], keysym::XK_o, cmd::lazy::shuffle_previous()),
             (vec![modkey], keysym::XK_p, cmd::lazy::spawn_command(Command::new("xterm"))),
             (vec![modkey], keysym::XK_b, cmd::lazy::layout_next())];

    let layouts = vec![StackLayout::new("stack".to_owned()),
                       TiledLayout::new("tiled".to_owned())];

    let group_metadata = vec![(keysym::XK_n, "g1", "stack"), (keysym::XK_m, "g2", "tiled")];
    let groups: Vec<GroupBuilder> = group_metadata
        .into_iter()
        .map(|(key, name, default_layout_name)| {
                 keys.push((vec![modkey], key, cmd::lazy::switch_group(name)));

                 GroupBuilder::new(name.to_owned(), default_layout_name.to_owned())
             })
        .collect();

    let mut wm = RustWindowManager::new(keys, groups, layouts).unwrap();
    wm.run_event_loop();
}
