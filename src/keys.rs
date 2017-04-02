use std::fmt;
use std::os::raw::c_uint;

use super::RustWindowManager;


pub struct Key {
    pub mod_mask: c_uint,
    pub keysym: c_uint,
    pub handler: Box<Fn(&RustWindowManager)>,
}


impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Key {{ mod_mask: {}, keysym: {} }}",
               self.mod_mask,
               self.keysym)
    }
}
