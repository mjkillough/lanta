use std::collections::HashMap;
use std::os::raw::c_uint;

use crate::cmd::Command;

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
        xcb::MOD_MASK_SHIFT
            | xcb::MOD_MASK_LOCK
            | xcb::MOD_MASK_CONTROL
            | xcb::MOD_MASK_1
            | xcb::MOD_MASK_2
            | xcb::MOD_MASK_3
            | xcb::MOD_MASK_4
            | xcb::MOD_MASK_5
    }

    fn mask(self) -> ModMask {
        match self {
            ModKey::Shift => xcb::MOD_MASK_SHIFT,
            ModKey::Lock => xcb::MOD_MASK_LOCK,
            ModKey::Control => xcb::MOD_MASK_CONTROL,
            ModKey::Mod1 => xcb::MOD_MASK_1,
            ModKey::Mod2 => xcb::MOD_MASK_2,
            ModKey::Mod3 => xcb::MOD_MASK_3,
            ModKey::Mod4 => xcb::MOD_MASK_4,
            ModKey::Mod5 => xcb::MOD_MASK_5,
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
    fn new(mods: &[ModKey], keysym: Key) -> KeyCombo {
        let mod_mask = mods.iter().fold(0, |mask, mod_key| mask | mod_key.mask());
        KeyCombo { mod_mask, keysym }
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
        self.hashmap.get(key_combo).cloned()
    }
}

impl From<Vec<(Vec<ModKey>, Key, Command)>> for KeyHandlers {
    fn from(handlers: Vec<(Vec<ModKey>, Key, Command)>) -> KeyHandlers {
        let mut hashmap = HashMap::new();
        for (modkeys, keysym, handler) in handlers {
            hashmap.insert(KeyCombo::new(&modkeys, keysym), handler);
        }
        KeyHandlers { hashmap }
    }
}
