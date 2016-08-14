
use super::interface::*;
use super::sys::*;

static MOUSE_CURSOR_INTERFACE: PPB_MouseCursor_1_0 = PPB_MouseCursor_1_0 {
    SetCursor: Some(ret_false_stub),
};
pub static INTERFACES: Interfaces = &[
    ("PPB_MouseCursor;1.0", interface_ptr(&MOUSE_CURSOR_INTERFACE)),
];
