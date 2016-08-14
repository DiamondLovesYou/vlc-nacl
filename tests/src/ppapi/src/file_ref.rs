
use libc;

use super::callback::Callback;
use super::interface::*;
use super::sys::*;
use super::resource::{get_resource_instance, get_resource_arc, ResState};

static FILE_REF_INTERFACE: PPB_FileRef_1_2 = PPB_FileRef_1_2 {
    create: create,
    is: is,
    fs_type: fs_type,
    get_name: get_name,
    get_path: get_path,
    get_parent: get_parent,
    mkdir: mkdir,
    touch: touch,
    delete: delete,
    rename: rename,
    query: query,
    read_dir_entries: read_dir_entries,
};

pub static INTERFACES: Interfaces = &[
    ("PPB_FileRef;1.2", interface_ptr(&FILE_REF_INTERFACE)),
];

extern "C" fn create(file_system: PP_Resource,
                     path: *const libc::c_char) -> PP_Resource {
    use std::ffi::CStr;
    use std::path::Path;
    let path = unsafe { CStr::from_ptr(path) };
    let path = path.to_str();
    let path = match path {
        Ok(s) => Path::new(s).to_path_buf(),
        Err(_) => { return 0; },
    };

    let instance = if let Some(i) = get_resource_instance(file_system) {
        i
    } else {
        return 0;
    };

    instance.create_file_ref(file_system, path)
        .map(|r| r.id() )
        .unwrap_or(0)
}

extern "C" fn is(r: PP_Resource) -> PP_Bool {
    if let Some(r) = unsafe { get_resource_arc(r) } {
        match r.state() {
            &ResState::FileRef(_) => PP_TRUE,
            _ => PP_FALSE,
        }
    } else {
        PP_FALSE
    }
}

extern "C" fn fs_type(_: PP_Resource) -> PP_FileSystemType { PP_FILESYSTEMTYPE_LOCALTEMPORARY }

extern "C" fn get_name(r: PP_Resource) -> PP_Var {
    if let Some(i) = get_resource_instance(r) {
        i.get_name_file_ref(r)
            .map(|v| v.into() )
            .ok()
            .unwrap_or_default()
    } else {
        Default::default()
    }
}
extern "C" fn get_path(r: PP_Resource) -> PP_Var {
    if let Some(i) = get_resource_instance(r) {
        i.get_path_file_ref(r)
            .map(|v| v.into() )
            .ok()
            .unwrap_or_default()
    } else {
        Default::default()
    }
}
extern "C" fn get_parent(r: PP_Resource) -> PP_Resource {
    if let Some(i) = get_resource_instance(r) {
        i.get_parent_file_ref(r)
            .map(|r| r.id() )
            .unwrap_or(0)
    } else {
        0
    }
}

extern "C" fn mkdir(directory_ref: PP_Resource,
                    make_directory_flags: libc::int32_t,
                    callback: PP_CompletionCallback) -> libc::int32_t {
    ppb_f!(R(directory_ref), callback, make_directory_flags as u32 => mkdir_file_ref)
}
extern "C" fn touch(file_ref: PP_Resource,
                    last_access_time: PP_Time,
                    last_modified_time: PP_Time,
                    callback: PP_CompletionCallback) -> libc::int32_t {
    ppb_f!(R(file_ref), callback, last_access_time, last_modified_time => touch_file_ref)
}
extern "C" fn delete(file_ref: PP_Resource, callback: PP_CompletionCallback) -> libc::int32_t {
    ppb_f!(R(file_ref), callback => delete_file_ref)
}
extern "C" fn rename(file_ref: PP_Resource,
                     new_file_ref: PP_Resource,
                     callback: PP_CompletionCallback) -> libc::int32_t {
    ppb_f!(R(file_ref), callback, new_file_ref => rename_file_ref)
}
extern "C" fn query(file_ref: PP_Resource,
                    info: *mut PP_FileInfo,
                    callback: PP_CompletionCallback) -> libc::int32_t {
    let dest_info = unsafe { info.as_mut() };
    if dest_info.is_none() {
        return PP_ERROR_BADARGUMENT;
    }
    let dest_info = dest_info.unwrap();

    if let Some(i) = get_resource_instance(file_ref) {

        let callback = match Callback::from_ffi(callback) {
            Ok(cb) => cb,
            Err(c) => { return c.into(); },
        };

        match i.query_file_ref(file_ref, callback) {
            Ok(info) => {
                *dest_info = info;
                return PP_OK;
            },
            Err(c) => { return c.into(); },
        }
    } else {
        return PP_ERROR_BADARGUMENT;
    }
}

extern "C" fn read_dir_entries(file_ref: PP_Resource,
                               output: PP_ArrayOutput,
                               callback: PP_CompletionCallback) -> libc::int32_t {
    use std::mem::size_of;
    use std::slice::from_raw_parts_mut;

    use ppapi::resource::RefCounted;

    if output.alloc.is_none() { return PP_ERROR_BADARGUMENT; }

    if let Some(i) = get_resource_instance(file_ref) {

        let callback = match Callback::from_ffi(callback) {
            Ok(cb) => cb,
            Err(c) => { return c.into(); },
        };

        match i.read_dir_entries_file_ref(file_ref, callback) {
            Ok(entries) => {
                let entry_size = size_of::<PP_DirectoryEntry>();
                let entry_count = entries.len();

                let alloc_f = output.alloc.unwrap();
                let dest = alloc_f(output.user_data,
                                   entry_count as u32,
                                   entry_size as u32);

                // Errors must not happen from here on.
                let dest = unsafe {
                    from_raw_parts_mut(dest as *mut PP_DirectoryEntry, entry_count)
                };

                for idx in 0..entry_count {
                    let fr = &entries[idx];
                    let info = &mut dest[idx];

                    fr.get_rc().up_ref(); // they are expected to down ref the file refs when done.

                    info.file_ref = fr.id();
                    info.file_type = {
                        let r = fr.read_inner(|inner| Ok(inner.file_type()) );

                        // We can't dealloc, so we must gracefully handle all errors.
                        r.unwrap_or(PP_FILETYPE_OTHER)
                    };
                }

                PP_OK
            },
            Err(c) => { return c.into(); },
        }
    } else {
        return PP_ERROR_BADARGUMENT;
    }
}
