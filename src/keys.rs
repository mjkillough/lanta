use std::rc::Rc;
use std::collections::HashMap;
use std::os::raw::c_uint;

use x11::xlib;

use super::RustWindowManager;


/// Represents a modifier key.
#[allow(dead_code)]
pub enum ModKey {
    Mod1,
    Mod2,
    Mod3,
    Mod4,
    Mod5,
}

pub type ModMask = c_uint;

impl ModKey {
    pub fn mask_all() -> ModMask {
        xlib::Mod1Mask | xlib::Mod2Mask | xlib::Mod3Mask | xlib::Mod4Mask | xlib::Mod5Mask
    }

    pub fn mask(&self) -> ModMask {
        match self {
            &ModKey::Mod1 => xlib::Mod1Mask,
            &ModKey::Mod2 => xlib::Mod2Mask,
            &ModKey::Mod3 => xlib::Mod3Mask,
            &ModKey::Mod4 => xlib::Mod4Mask,
            &ModKey::Mod5 => xlib::Mod5Mask,
        }
    }
}


/// A single key, of the same type as the `x11::keysym` constants.
pub type Key = c_uint;


/// A combination of zero or more mods and a key.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct KeyCombo {
    pub mod_mask: ModMask,
    pub keysym: Key,
}

impl KeyCombo {
    pub fn new(mods: Vec<ModKey>, keysym: Key) -> KeyCombo {
        let mask = mods.iter()
            .fold(0, |mask, mod_key| mask | mod_key.mask());
        debug!("{}", mask);
        KeyCombo {
            mod_mask: mask,
            keysym: keysym,
        }
    }
}


// XXX We need to use Rc in order to allow us to borrow RustWindowManager mutably to pass it to
// handlers. However, using a Rc for an event handler sounds like a terrible idea - could it cause
// UAF?
pub type KeyHandler = Rc<Fn(&mut RustWindowManager)>;


/// A collection of `KeyHandler`.
pub struct KeyHandlers {
    handlers: HashMap<KeyCombo, KeyHandler>,
}

impl KeyHandlers {
    pub fn new(mut handlers: Vec<(KeyCombo, KeyHandler)>) -> Self {
        let mut hashmap = HashMap::new();
        loop {
            let (combo, handler) = match handlers.pop() {
                Some((c, h)) => (c, h),
                None => break,
            };
            hashmap.insert(combo, handler);
        }

        KeyHandlers { handlers: hashmap }
    }

    pub fn key_combos(&self) -> Vec<&KeyCombo> {
        self.handlers.keys().collect()
    }

    pub fn get(&self, key_combo: &KeyCombo) -> Option<KeyHandler> {
        self.handlers.get(key_combo).map(|rc| rc.clone())
    }
}
