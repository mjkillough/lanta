extern crate env_logger;
extern crate x11;

extern crate lanta;

use std::process::Command;
use std::rc::Rc;

use lanta::{Config, RustWindowManager};
use lanta::keys::{KeyCombo, KeyHandlers, ModKey};
use lanta::layout::TiledLayout;


fn main() {
    env_logger::init().unwrap();

    let keys = KeyHandlers::new(vec![(KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_t),
                                      Rc::new(lanta::close_window)),
                                     (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_y),
                                      Rc::new(lanta::focus_next)),
                                     (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_u),
                                      Rc::new(lanta::focus_previous)),
                                     (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_i),
                                      Rc::new(lanta::shuffle_next)),
                                     (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_o),
                                      Rc::new(lanta::shuffle_previous)),
                                     (KeyCombo::new(vec![ModKey::Mod4], x11::keysym::XK_p),
                                      lanta::spawn_command(Command::new("xterm")))]);

    let layout = Box::new(TiledLayout {});
    let config = Config {
        keys: keys,
        layout: layout,
    };

    let mut wm = RustWindowManager::new(config).unwrap();
    wm.run_event_loop();
}
