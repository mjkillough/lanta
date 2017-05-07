use std::collections::HashMap;
use std::os::raw::c_uint;

use x11::xlib;

use cmd::Command;


/// Represents a modifier key.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModKey {
    Shift,
    Lock,
    Control,
    Mod1,
    Mod2,
    Mod3,
    Mod4,
    Mod5,
}

type ModMask = c_uint;

impl ModKey {
    pub fn mask_all() -> ModMask {
        xlib::ShiftMask | xlib::LockMask | xlib::ControlMask | xlib::Mod1Mask |
        xlib::Mod2Mask | xlib::Mod3Mask | xlib::Mod4Mask | xlib::Mod5Mask
    }

    fn mask(&self) -> ModMask {
        match self {
            &ModKey::Shift => xlib::ShiftMask,
            &ModKey::Lock => xlib::LockMask,
            &ModKey::Control => xlib::ControlMask,
            &ModKey::Mod1 => xlib::Mod1Mask,
            &ModKey::Mod2 => xlib::Mod2Mask,
            &ModKey::Mod3 => xlib::Mod3Mask,
            &ModKey::Mod4 => xlib::Mod4Mask,
            &ModKey::Mod5 => xlib::Mod5Mask,
        }
    }
}


/// A single key, of the same type as the `x11::keysym` constants.
type Key = c_uint;


/// A combination of zero or more mods and a key.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct KeyCombo {
    pub mod_mask: ModMask,
    pub keysym: Key,
}

impl KeyCombo {
    fn new(mods: Vec<ModKey>, keysym: Key) -> KeyCombo {
        let mask = mods.iter()
            .fold(0, |mask, mod_key| mask | mod_key.mask());
        KeyCombo {
            mod_mask: mask,
            keysym: keysym,
        }
    }
}


pub struct KeyHandlers {
    hashmap: HashMap<KeyCombo, Command>,
}

impl KeyHandlers {
    pub fn key_combos(&self) -> Vec<&KeyCombo> {
        self.hashmap.keys().collect()
    }

    pub fn get(&self, key_combo: &KeyCombo) -> Option<Command> {
        self.hashmap.get(key_combo).map(|rc| rc.clone())
    }
}

impl From<Vec<(Vec<ModKey>, Key, Command)>> for KeyHandlers {
    fn from(handlers: Vec<(Vec<ModKey>, Key, Command)>) -> KeyHandlers {
        let mut hashmap = HashMap::new();
        for (modkeys, keysym, handler) in handlers {
            hashmap.insert(KeyCombo::new(modkeys, keysym), handler);
        }
        KeyHandlers { hashmap }
    }
}
