extern crate env_logger;
extern crate x11;

extern crate lanta;


use lanta::RustWindowManager;
use lanta::groups::GroupBuilder;
use lanta::keys::{KeyCombo, KeyHandler, KeyHandlers, ModKey};
use lanta::layout::{StackLayout, TiledLayout};
use std::process::Command;
use std::rc::Rc;


fn main() {
    env_logger::init().unwrap();

    let mut keys: Vec<(KeyCombo, KeyHandler)> =
        vec![(KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_t), Rc::new(lanta::close_window)),
             (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_y), Rc::new(lanta::focus_next)),
             (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_u),
              Rc::new(lanta::focus_previous)),
             (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_i), Rc::new(lanta::shuffle_next)),
             (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_o),
              Rc::new(lanta::shuffle_previous)),
             (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_p),
              lanta::spawn_command(Command::new("xterm"))),
             (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_b), Rc::new(lanta::layout_next)),
             (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_n), lanta::switch_group("g1")),
             (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_m), lanta::switch_group("g2"))];

    let layouts = vec![StackLayout::new("stack".to_owned()),
                       TiledLayout::new("tiled".to_owned())];


    let groups: Vec<GroupBuilder> = vec![(x11::keysym::XK_n, "g1", "stack"),
                                         (x11::keysym::XK_m, "g2", "tiled")]
            .into_iter()
            .map(|(key, name, default_layout_name)| {
                     keys.push((KeyCombo::new(vec![ModKey::Mod4], key), lanta::switch_group(name)));

                     GroupBuilder::new(name.to_owned(), default_layout_name.to_owned())
                 })
            .collect();



    let mut wm = RustWindowManager::new(KeyHandlers::new(keys), groups, layouts).unwrap();
    wm.run_event_loop();
}
