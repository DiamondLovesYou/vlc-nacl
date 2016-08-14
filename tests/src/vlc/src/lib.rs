
extern crate libc;

pub mod sys {
    #![allow(dead_code,
             non_camel_case_types,
             non_upper_case_globals,
             non_snake_case)]

    //unused_imports

    use libc::*;

    // A few types we don't care about, but which are needed for bindgen.
    pub type libvlc_int_t = c_void;
    pub type block_t = c_void;

    include!(concat!(env!("OUT_DIR"), "/sys.rs"));
}

pub mod object;
pub mod variable;
pub mod access;

pub struct Value(pub sys::vlc_value_t);
impl Value {

}
