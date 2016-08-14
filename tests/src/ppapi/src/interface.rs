
/// Some interface helpers.

use super::sys::*;
pub use super::SyncInterfacePtr;

pub type Interfaces = &'static [(&'static str, SyncInterfacePtr)];

pub const fn interface_ptr<T>(iptr: &'static T) -> SyncInterfacePtr {
    SyncInterfacePtr::new(iptr)
}

pub extern "C" fn ret_default_stub<T>() -> T
    where T: Default,
{
    Default::default()
}
pub extern "C" fn ret_false_stub() -> PP_Bool {
    PP_FALSE
}
pub extern "C" fn ret_not_supported_stub() -> ::libc::int32_t { PP_ERROR_NOTSUPPORTED }
