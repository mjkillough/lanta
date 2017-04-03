use std::fmt;
use std::os::raw::c_uint;

use super::RustWindowManager;


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key {
    pub mod_mask: c_uint,
    pub keysym: c_uint,
}


pub struct KeyHandler {
    pub key: Key,
    pub handler: Box<Fn(&RustWindowManager)>,
}
