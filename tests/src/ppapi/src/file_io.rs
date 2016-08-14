
use libc;

use super::{SyncInterfacePtr, ModuleInterface};
use super::sys::*;
use super::resource::{Resource, get_resource_instance};
use super::result::{ResultCode};

pub use super::filesystem_manager::FileIoState;

pub static FILEIO_INTERFACE: PPB_FileIO_1_1 = PPB_FileIO_1_1 {
    create: create,
    is: is,
    open: open,
    query: query,
    touch: touch,
    read: read,
    write: write,
    set_length: set_length,
    flush: flush,
    close: close,
    read_to_array: read_to_array,
};

pub static INTERFACES: &'static [(&'static str, SyncInterfacePtr)] = &[
    ("PPB_FileIO;1.1", SyncInterfacePtr::new(&FILEIO_INTERFACE)),
];

extern "C" fn create(instance: PP_Instance) -> PP_Resource {
    let ret = ModuleInterface::get_instance_interface(instance);
    let ret = ret.map(|i| i.create_file_io() );

    ret.map(|code| code.into_code() )
        .unwrap_or(0)
}
extern "C" fn is(_resource: PP_Resource) -> PP_Bool {
    unimplemented!()
}

extern "C" fn open(file_io: PP_Resource,
                   file_ref: PP_Resource,
                   open_flags: libc::int32_t,
                   callback: PP_CompletionCallback) -> libc::int32_t {
    ppb_f!(R(file_io), callback, file_ref, open_flags as u32 => open_file_io)
}
extern "C" fn query(file_io: PP_Resource,
                    info: *mut PP_FileInfo,
                    callback: PP_CompletionCallback) -> libc::int32_t {
    ppb_f!(R(file_io), callback, info => query_file_io)
}
extern "C" fn touch(file_io: PP_Resource,
                    last_access_time: PP_Time,
                    last_modified_time: PP_Time,
                    callback: PP_CompletionCallback) -> libc::int32_t {
    ppb_f!(R(file_io), callback, last_access_time, last_modified_time => touch_file_io)
}
extern "C" fn read(file_io: PP_Resource, offset: libc::int64_t,
                   buffer: *mut ::libc::c_char,
                   bytes_to_read: libc::int32_t,
                   callback: PP_CompletionCallback) -> libc::int32_t {
    use std::slice::from_raw_parts_mut;
    let buffer = unsafe {
        from_raw_parts_mut(buffer as *mut u8, bytes_to_read as usize)
    };
    ppb_f!(R(file_io), callback, offset as usize, buffer => read_file_io)
}
extern "C" fn write(file_io: PP_Resource, offset: libc::int64_t,
                    buffer: *const ::libc::c_char,
                    bytes_to_write: libc::int32_t,
                    callback: PP_CompletionCallback) -> libc::int32_t {
    use std::slice::from_raw_parts;
    let buffer = unsafe {
        from_raw_parts(buffer as *const u8, bytes_to_write as usize)
    };
    ppb_f!(R(file_io), callback, offset as usize, buffer => write_file_io)
}
extern "C" fn set_length(file_io: PP_Resource,
                         length: libc::int64_t,
                         callback: PP_CompletionCallback) -> libc::int32_t {
    ppb_f!(R(file_io), callback, length as usize => set_length_file_io)
}
extern "C" fn flush(file_io: PP_Resource,
                    callback: PP_CompletionCallback) -> libc::int32_t {
    ppb_f!(R(file_io), callback => flush_file_io )
}
extern "C" fn close(file_io: PP_Resource) {
    if let Some(instance) = get_resource_instance(file_io) {
        instance.close_file_io(file_io)
    }
}
extern "C" fn read_to_array(_file_io: PP_Resource, _offset: libc::int64_t,
                            _max_read_length: libc::int32_t, _output: *mut PP_ArrayOutput,
                            _callback: PP_CompletionCallback) -> libc::int32_t {
    PP_ERROR_NOTSUPPORTED
}

pub type FileIo = Resource<FileIoState>;
