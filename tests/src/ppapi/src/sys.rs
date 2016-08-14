
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_snake_case)]

/// This file is constructed from `rust-ppapi`.

use libc;
use libc::*;

#[cfg(not(test))]
#[link(name = "ppapi_modules", kind = "static")]
extern { }

pub type PP_Module = i32;
pub type PP_Instance = libc::int32_t;
pub type PP_Resource = libc::int32_t;
pub type PP_VarId    = libc::int64_t;
pub type PP_Time = libc::c_double;
pub type PP_TimeTicks = libc::c_double;
pub type PP_Code = ::libc::c_int;

pub type GetInterface = extern "C" fn(c_str: *const libc::c_char) -> *const libc::c_void;

pub type StubInterfaceFunc<T = PP_Code> = extern "C" fn() -> T;

pub const PP_COMPLETIONCALLBACK_FLAG_NONE: ::libc::c_uint = 0;
pub const PP_COMPLETIONCALLBACK_FLAG_OPTIONAL: ::libc::c_uint = 1;
pub type PP_CompletionCallback_Func = extern "C" fn(user_data: *mut libc::c_void,
                                                    result: libc::int32_t);
#[repr(C)]
#[derive(Copy)]
pub struct PP_CompletionCallback {
    pub func: PP_CompletionCallback_Func,
    pub user_data: *mut ::libc::c_void,
    pub flags: int32_t,
}
impl ::std::clone::Clone for PP_CompletionCallback {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PP_CompletionCallback {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
unsafe impl Send for PP_CompletionCallback { }

pub const PP_OK: ::libc::c_int = 0;
pub const PP_OK_COMPLETIONPENDING: ::libc::c_int = -1;
pub const PP_ERROR_FAILED: ::libc::c_int = -2;
pub const PP_ERROR_ABORTED: ::libc::c_int = -3;
pub const PP_ERROR_BADARGUMENT: ::libc::c_int = -4;
pub const PP_ERROR_BADRESOURCE: ::libc::c_int = -5;
pub const PP_ERROR_NOINTERFACE: ::libc::c_int = -6;
pub const PP_ERROR_NOACCESS: ::libc::c_int = -7;
pub const PP_ERROR_NOMEMORY: ::libc::c_int = -8;
pub const PP_ERROR_NOSPACE: ::libc::c_int = -9;
pub const PP_ERROR_NOQUOTA: ::libc::c_int = -10;
pub const PP_ERROR_INPROGRESS: ::libc::c_int = -11;
pub const PP_ERROR_NOTSUPPORTED: ::libc::c_int = -12;
pub const PP_ERROR_BLOCKS_MAIN_THREAD: ::libc::c_int = -13;
pub const PP_ERROR_MALFORMED_INPUT: ::libc::c_int = -14;
pub const PP_ERROR_RESOURCE_FAILED: ::libc::c_int = -15;
pub const PP_ERROR_FILENOTFOUND: ::libc::c_int = -20;
pub const PP_ERROR_FILEEXISTS: ::libc::c_int = -21;
pub const PP_ERROR_FILETOOBIG: ::libc::c_int = -22;
pub const PP_ERROR_FILECHANGED: ::libc::c_int = -23;
pub const PP_ERROR_NOTAFILE: ::libc::c_int = -24;
pub const PP_ERROR_TIMEDOUT: ::libc::c_int = -30;
pub const PP_ERROR_USERCANCEL: ::libc::c_int = -40;
pub const PP_ERROR_NO_USER_GESTURE: ::libc::c_int = -41;
pub const PP_ERROR_CONTEXT_LOST: ::libc::c_int = -50;
pub const PP_ERROR_NO_MESSAGE_LOOP: ::libc::c_int = -51;
pub const PP_ERROR_WRONG_THREAD: ::libc::c_int = -52;
pub const PP_ERROR_WOULD_BLOCK_THREAD: ::libc::c_int = -53;
pub const PP_ERROR_CONNECTION_CLOSED: ::libc::c_int = -100;
pub const PP_ERROR_CONNECTION_RESET: ::libc::c_int = -101;
pub const PP_ERROR_CONNECTION_REFUSED: ::libc::c_int = -102;
pub const PP_ERROR_CONNECTION_ABORTED: ::libc::c_int = -103;
pub const PP_ERROR_CONNECTION_FAILED: ::libc::c_int = -104;
pub const PP_ERROR_CONNECTION_TIMEDOUT: ::libc::c_int = -105;
pub const PP_ERROR_ADDRESS_INVALID: ::libc::c_int = -106;
pub const PP_ERROR_ADDRESS_UNREACHABLE: ::libc::c_int = -107;
pub const PP_ERROR_ADDRESS_IN_USE: ::libc::c_int = -108;
pub const PP_ERROR_MESSAGE_TOO_BIG: ::libc::c_int = -109;
pub const PP_ERROR_NAME_NOT_RESOLVED: ::libc::c_int = -110;

pub const PP_FALSE: ::libc::c_uint = 0;
pub const PP_TRUE: ::libc::c_uint = 1;
pub type PP_Bool = libc::c_uint;

pub const PP_VARTYPE_UNDEFINED: ::libc::c_uint = 0;
pub const PP_VARTYPE_NULL: ::libc::c_uint = 1;
pub const PP_VARTYPE_BOOL: ::libc::c_uint = 2;
pub const PP_VARTYPE_INT32: ::libc::c_uint = 3;
pub const PP_VARTYPE_DOUBLE: ::libc::c_uint = 4;
pub const PP_VARTYPE_STRING: ::libc::c_uint = 5;
pub const PP_VARTYPE_OBJECT: ::libc::c_uint = 6;
pub const PP_VARTYPE_ARRAY: ::libc::c_uint = 7;
pub const PP_VARTYPE_DICTIONARY: ::libc::c_uint = 8;
pub const PP_VARTYPE_ARRAY_BUFFER: ::libc::c_uint = 9;
pub const PP_VARTYPE_RESOURCE: ::libc::c_uint = 10;
pub type PP_VarType = libc::c_uint;

#[repr(C)]
#[derive(Copy)]
pub struct Union_PP_VarValue {
    pub _bindgen_data_: [u64; 1usize],
}
impl Union_PP_VarValue {
    pub unsafe fn as_bool(&mut self) -> *mut PP_Bool {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn as_int(&mut self) -> *mut int32_t {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn as_double(&mut self) -> *mut ::libc::c_double {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn as_id(&mut self) -> *mut int64_t {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
}
impl ::std::clone::Clone for Union_PP_VarValue {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for Union_PP_VarValue {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy)]
pub struct PP_Var {
    pub _type: PP_VarType,
    pub padding: int32_t,
    pub value: Union_PP_VarValue,
}
impl ::std::default::Default for PP_Var {
    fn default() -> Self {
        PP_Var {
            _type: PP_VARTYPE_NULL,
            padding: 0,
            value: Default::default(),
        }
    }
}
impl ::std::clone::Clone for PP_Var {
    fn clone(&self) -> Self { *self }
}

#[repr(C)]
#[derive(Copy)]
pub struct PPP_Instance_1_1 {
    pub create: extern "C" fn(instance: PP_Instance,
                              argc: uint32_t,
                              argn: *mut *const ::libc::c_char,
                              argv: *mut *const ::libc::c_char)
                              -> PP_Bool,
    pub destroy: extern "C" fn(instance: PP_Instance),
    pub change_view: extern "C" fn(instance: PP_Instance, view: PP_Resource),
    pub change_focus: extern "C" fn(instance: PP_Instance, has_focus: PP_Bool),
    pub handle_document_load: extern "C" fn(instance: PP_Instance,
                                            url_loader: PP_Resource) -> PP_Bool,
}
impl ::std::clone::Clone for PPP_Instance_1_1 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPP_Instance_1_1 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}


#[repr(C)]
#[derive(Copy)]
pub struct PPB_Instance_1_0 {
    pub BindGraphics: Option<StubInterfaceFunc<PP_Bool>>,
    pub IsFullFrame: Option<StubInterfaceFunc<PP_Bool>>,
}
impl ::std::clone::Clone for PPB_Instance_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_Instance_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
pub struct PPB_Core_1_0 {
    pub up_ref_resource: extern "C" fn(resource: PP_Resource),
    pub down_ref_resource: extern "C" fn(resource: PP_Resource),
    pub get_time: extern "C" fn() -> PP_Time,
    pub get_time_ticks: extern "C" fn() -> PP_TimeTicks,
    pub call_on_main_thread: extern "C" fn(delay_in_milliseconds: int32_t,
                                           callback: PP_CompletionCallback,
                                           result: int32_t),
    pub on_main_thread: extern "C" fn() -> PP_Bool,
}

pub const PP_FILESYSTEMTYPE_LOCALTEMPORARY: ::libc::c_uint = 3;
pub type PP_FileSystemType = libc::c_uint;

pub const PP_FILETYPE_REGULAR: ::libc::c_uint = 0;
pub const PP_FILETYPE_DIRECTORY: ::libc::c_uint = 1;
pub const PP_FILETYPE_OTHER: ::libc::c_uint = 2;
pub type PP_FileType = ::libc::c_uint;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PP_FileInfo {
    pub size: int64_t,
    pub _type: PP_FileType,
    pub system_type: PP_FileSystemType,
    pub creation_time: PP_Time,
    pub last_access_time: PP_Time,
    pub last_modified_time: PP_Time,
}
impl ::std::default::Default for PP_FileInfo {
    fn default() -> PP_FileInfo { unsafe { ::std::mem::zeroed() } }
}

pub const PP_FILEOPENFLAG_READ: ::libc::c_uint = 1;
pub const PP_FILEOPENFLAG_WRITE: ::libc::c_uint = 2;
pub const PP_FILEOPENFLAG_CREATE: ::libc::c_uint = 4;
pub const PP_FILEOPENFLAG_TRUNCATE: ::libc::c_uint = 8;
pub const PP_FILEOPENFLAG_EXCLUSIVE: ::libc::c_uint = 16;
pub const PP_FILEOPENFLAG_APPEND: ::libc::c_uint = 32;
pub type PP_FileOpenFlags = ::libc::c_uint;

#[repr(C)]
pub struct PPB_FileIO_1_1 {
    pub create: extern "C" fn(instance: PP_Instance) -> PP_Resource,
    pub is: extern "C" fn(resource: PP_Resource) -> PP_Bool,
    pub open: extern "C" fn(file_io: PP_Resource,
                            file_ref: PP_Resource,
                            open_flags: int32_t,
                            callback: PP_CompletionCallback) -> int32_t,
    pub query: extern "C" fn(file_io: PP_Resource,
                             info: *mut PP_FileInfo,
                             callback: PP_CompletionCallback) -> int32_t,
    pub touch: extern "C" fn(file_io: PP_Resource,
                             last_access_time: PP_Time,
                             last_modified_time: PP_Time,
                             callback: PP_CompletionCallback) -> int32_t,
    pub read: extern "C" fn(file_io: PP_Resource, offset: int64_t,
                            buffer: *mut ::libc::c_char,
                            bytes_to_read: int32_t,
                            callback: PP_CompletionCallback) -> int32_t,
    pub write: extern "C" fn(file_io: PP_Resource, offset: int64_t,
                             buffer: *const ::libc::c_char,
                             bytes_to_write: int32_t,
                             callback: PP_CompletionCallback) -> int32_t,
    pub set_length: extern "C" fn(file_io: PP_Resource,
                                  length: int64_t,
                                  callback: PP_CompletionCallback) -> int32_t,
    pub flush: extern "C" fn(file_io: PP_Resource,
                             callback: PP_CompletionCallback) -> int32_t,
    pub close: extern "C" fn(file_io: PP_Resource),
    pub read_to_array: extern "C" fn(file_io: PP_Resource, offset: int64_t,
                                     max_read_length: int32_t, output: *mut PP_ArrayOutput,
                                     callback: PP_CompletionCallback) -> int32_t,
}

#[repr(C)]
#[derive(Copy)]
pub struct PP_DirectoryEntry {
    pub file_ref: PP_Resource,
    pub file_type: PP_FileType,
}
impl ::std::clone::Clone for PP_DirectoryEntry {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PP_DirectoryEntry {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

pub type PP_ArrayOutput_GetDataBuffer = extern "C" fn(user_data: *mut ::libc::c_void,
                                                      element_count: uint32_t,
                                                      element_size: uint32_t) -> *mut ::libc::c_void;
#[repr(C)]
#[derive(Copy)]
pub struct PP_ArrayOutput {
    pub alloc: Option<PP_ArrayOutput_GetDataBuffer>,
    pub user_data: *mut ::libc::c_void,
}
impl ::std::clone::Clone for PP_ArrayOutput {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PP_ArrayOutput {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

pub const PP_MAKEDIRECTORYFLAG_NONE: ::libc::c_uint = 0;
pub const PP_MAKEDIRECTORYFLAG_WITH_ANCESTORS: ::libc::c_uint = 1;
pub const PP_MAKEDIRECTORYFLAG_EXCLUSIVE: ::libc::c_uint = 2;
pub type PP_MakeDirectoryFlags = libc::c_uint;
#[repr(C)]
#[derive(Copy)]
pub struct PPB_FileRef_1_2 {
    pub create: extern "C" fn(file_system: PP_Resource,
                              path: *const ::libc::c_char) -> PP_Resource,
    pub is: extern "C" fn(resource: PP_Resource) -> PP_Bool,
    pub fs_type: extern "C" fn(file_ref: PP_Resource) -> PP_FileSystemType,
    pub get_name: extern "C" fn(file_ref: PP_Resource) -> PP_Var,
    pub get_path: extern "C" fn(file_ref: PP_Resource) -> PP_Var,
    pub get_parent: extern "C" fn(file_ref: PP_Resource) -> PP_Resource,
    pub mkdir: extern "C" fn(directory_ref: PP_Resource,
                             make_directory_flags: int32_t,
                             callback: PP_CompletionCallback) -> int32_t,
    pub touch: extern "C" fn(file_ref: PP_Resource,
                             last_access_time: PP_Time,
                             last_modified_time: PP_Time,
                             callback: PP_CompletionCallback) -> int32_t,
    pub delete: extern "C" fn(file_ref: PP_Resource, callback: PP_CompletionCallback) -> int32_t,
    pub rename: extern "C" fn(file_ref: PP_Resource,
                              new_file_ref: PP_Resource,
                              callback: PP_CompletionCallback) -> int32_t,
    pub query: extern "C" fn(file_ref: PP_Resource,
                             info: *mut PP_FileInfo,
                             callback: PP_CompletionCallback) -> int32_t,
    pub read_dir_entries: extern "C" fn(file_ref: PP_Resource,
                                        output: PP_ArrayOutput,
                                        callback: PP_CompletionCallback) -> int32_t,
}
impl ::std::clone::Clone for PPB_FileRef_1_2 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_FileRef_1_2 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy)]
pub struct PPB_Audio_1_1 {
    pub create: StubInterfaceFunc<PP_Resource>,
    pub is_audio: StubInterfaceFunc<PP_Bool>,
    pub get_config: StubInterfaceFunc<PP_Resource>,
    pub start_playback: StubInterfaceFunc<PP_Bool>,
    pub stop_playback: StubInterfaceFunc<PP_Bool>,
}
impl ::std::clone::Clone for PPB_Audio_1_1 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_Audio_1_1 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy)]
pub struct PPB_AudioConfig_1_1 {
    pub CreateStereo16Bit: StubInterfaceFunc<PP_Resource>,
    pub RecommendSampleFrameCount: StubInterfaceFunc<uint32_t>,
    pub IsAudioConfig: StubInterfaceFunc<PP_Bool>,
    pub GetSampleRate: StubInterfaceFunc<c_uint>,
    pub GetSampleFrameCount: StubInterfaceFunc<uint32_t>,
    pub RecommendSampleRate: StubInterfaceFunc<c_uint>,
}
impl ::std::clone::Clone for PPB_AudioConfig_1_1 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_AudioConfig_1_1 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

pub type Enum_Unnamed7 = ::libc::c_uint;
pub const PP_LOGLEVEL_TIP: ::libc::c_uint = 0;
pub const PP_LOGLEVEL_LOG: ::libc::c_uint = 1;
pub const PP_LOGLEVEL_WARNING: ::libc::c_uint = 2;
pub const PP_LOGLEVEL_ERROR: ::libc::c_uint = 3;
pub type PP_LogLevel = Enum_Unnamed7;

#[repr(C)]
#[derive(Copy)]
pub struct PPB_Console_1_0 {
    pub Log: ::std::option::Option<extern "C" fn(instance: PP_Instance,
                                                 level: PP_LogLevel,
                                                 value: PP_Var) -> ()>,
    pub LogWithSource: ::std::option::Option<extern "C" fn(instance: PP_Instance,
                                                           level: PP_LogLevel,
                                                           source: PP_Var,
                                                           value: PP_Var)
                                                 -> ()>,
}
impl ::std::clone::Clone for PPB_Console_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_Console_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy)]
pub struct PPB_FileSystem_1_0 {
    pub Create: Option<extern "C" fn(instance: PP_Instance,
                                     _type: PP_FileSystemType)
                                     -> PP_Resource>,
    pub IsFileSystem: Option<extern "C" fn(resource: PP_Resource) -> PP_Bool>,
    pub Open: Option<extern "C" fn(file_system: PP_Resource,
                                   expected_size: int64_t,
                                   callback: PP_CompletionCallback)
                                   -> int32_t>,
    pub GetType: Option<extern "C" fn(file_system: PP_Resource)
                                      -> PP_FileSystemType>,
}
impl ::std::clone::Clone for PPB_FileSystem_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_FileSystem_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}


#[repr(C)]
#[derive(Copy)]
pub struct PPB_Var_1_2 {
    pub AddRef: Option<extern "C" fn(var: PP_Var)>,
    pub Release: Option<extern "C" fn(var: PP_Var)>,
    pub VarFromUtf8: Option<extern "C" fn(data: *const ::libc::c_char,
                                          len: uint32_t) -> PP_Var>,
    pub VarToUtf8: Option<extern "C" fn(var: PP_Var,
                                        len: *mut uint32_t)
                                        -> *const ::libc::c_char>,
    pub VarToResource: Option<extern "C" fn(var: PP_Var) -> PP_Resource>,
    pub VarFromResource: Option<extern "C" fn(resource: PP_Resource) -> PP_Var>,
    pub GlobalVarFromUtf8: Option<extern "C" fn(data: *const ::libc::c_char,
                                                len: uint32_t) -> PP_Var>
}
impl ::std::clone::Clone for PPB_Var_1_2 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_Var_1_2 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy)]
pub struct PPB_VarArray_1_0 {
    pub Create: Option<extern "C" fn() -> PP_Var>,
    pub Get: Option<extern "C" fn(array: PP_Var,
                                  index: uint32_t) -> PP_Var>,
    pub Set: Option<extern "C" fn(array: PP_Var,
                                  index: uint32_t,
                                  value: PP_Var) -> PP_Bool>,
    pub GetLength: Option<extern "C" fn(array: PP_Var) -> uint32_t>,
    pub SetLength: Option<extern "C" fn(array: PP_Var,
                                        length: uint32_t) -> PP_Bool>,
}
impl ::std::clone::Clone for PPB_VarArray_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_VarArray_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy)]
pub struct PPB_VarDictionary_1_0 {
    pub Create: Option<extern "C" fn() -> PP_Var>,
    pub Get: Option<extern "C" fn(dict: PP_Var,
                                  key: PP_Var) -> PP_Var>,
    pub Set: Option<extern "C" fn(dict: PP_Var,
                                  key: PP_Var,
                                  value: PP_Var) -> PP_Bool>,
    pub Delete: Option<extern "C" fn(dict: PP_Var,
                                     key: PP_Var)>,
    pub HasKey: Option<extern "C" fn(dict: PP_Var,
                                     key: PP_Var) -> PP_Bool>,
    pub GetKeys: Option<extern "C" fn(dict: PP_Var) -> PP_Var>,
}
impl ::std::clone::Clone for PPB_VarDictionary_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_VarDictionary_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy)]
pub struct PPB_MessageLoop_1_0 {
    pub Create: Option<extern "C" fn(instance: PP_Instance) -> PP_Resource>,
    pub GetForMainThread: Option<extern "C" fn() -> PP_Resource>,
    pub GetCurrent: Option<extern "C" fn() -> PP_Resource>,
    pub AttachToCurrentThread: Option<extern "C" fn(message_loop: PP_Resource) -> int32_t>,
    pub Run: Option<extern "C" fn(message_loop: PP_Resource) -> int32_t>,
    pub PostWork: Option<extern "C" fn(message_loop: PP_Resource,
                                       callback: PP_CompletionCallback,
                                       delay_ms: int64_t) -> int32_t>,
    pub PostQuit: Option<extern "C" fn(message_loop: PP_Resource,
                                       should_destroy: PP_Bool) -> int32_t>,
}
impl ::std::clone::Clone for PPB_MessageLoop_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_MessageLoop_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy)]
pub struct PPB_Graphics3D_1_0 {
    pub GetAttribMaxValue: Option<StubInterfaceFunc<int32_t>>,
    pub Create: Option<StubInterfaceFunc<PP_Resource>>,
    pub IsGraphics3D: Option<StubInterfaceFunc<PP_Bool>>,
    pub GetAttribs: Option<StubInterfaceFunc<int32_t>>,
    pub SetAttribs: Option<StubInterfaceFunc<int32_t>>,
    pub GetError: Option<StubInterfaceFunc<int32_t>>,
    pub ResizeBuffers: Option<StubInterfaceFunc<int32_t>>,
    pub SwapBuffers: Option<StubInterfaceFunc<int32_t>>,
}
impl ::std::clone::Clone for PPB_Graphics3D_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_Graphics3D_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy)]
pub struct PPB_MouseCursor_1_0 {
    pub SetCursor: Option<StubInterfaceFunc<PP_Bool>>,
}
impl ::std::clone::Clone for PPB_MouseCursor_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_MouseCursor_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Debug)]
pub struct PPP_MessageHandler_0_2 {
    pub HandleMessage: Option<extern "C" fn(instance: PP_Instance,
                                            user_data:  *mut ::libc::c_void,
                                            message: *const PP_Var)>,
    pub HandleBlockingMessage: Option<extern "C" fn(instance: PP_Instance,
                                                    user_data: *mut ::libc::c_void,
                                                    message: *const PP_Var,
                                                    response: *mut PP_Var)>,
    pub Destroy: Option<extern "C" fn(instance: PP_Instance,
                                      user_data: *mut ::libc::c_void)>,
}
impl ::std::clone::Clone for PPP_MessageHandler_0_2 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPP_MessageHandler_0_2 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
pub type PPP_MessageHandler = PPP_MessageHandler_0_2;
#[repr(C)]
#[derive(Copy)]
pub struct PPB_Messaging_1_2 {
    pub PostMessage: Option<extern "C" fn(instance: PP_Instance,
                                          message: PP_Var)>,
    pub RegisterMessageHandler: Option<extern "C" fn(instance: PP_Instance,
                                                     user_data: *mut ::libc::c_void,
                                                     handler: *const PPP_MessageHandler,
                                                     message_loop: PP_Resource) -> int32_t>,
    pub UnregisterMessageHandler: Option<extern "C" fn(instance: PP_Instance)>,
}
impl ::std::clone::Clone for PPB_Messaging_1_2 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_Messaging_1_2 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy)]
pub struct PPB_URLLoader_1_0 {
    pub Create: Option<extern "C" fn(instance: PP_Instance) -> PP_Resource>,
    pub IsURLLoader: Option<extern "C" fn(resource: PP_Resource) -> PP_Bool>,
    pub Open: Option<extern "C" fn(loader: PP_Resource,
                                   request_info: PP_Resource,
                                   callback: PP_CompletionCallback) -> int32_t>,
    pub FollowRedirect: Option<extern "C" fn(loader: PP_Resource,
                                             callback: PP_CompletionCallback) -> int32_t>,
    pub GetUploadProgress: Option<extern "C" fn(loader: PP_Resource,
                                                bytes_sent: *mut int64_t,
                                                total_bytes_to_be_sent: *mut int64_t) -> PP_Bool>,
    pub GetDownloadProgress: Option<extern "C" fn(loader: PP_Resource,
                                                  bytes_received: *mut int64_t,
                                                  total_bytes_to_be_received: *mut int64_t) -> PP_Bool>,
    pub GetResponseInfo: Option<extern "C" fn(loader: PP_Resource) -> PP_Resource>,
    pub ReadResponseBody: Option<extern "C" fn(loader: PP_Resource,
                                               buffer: *mut ::libc::c_void,
                                               bytes_to_read: int32_t,
                                               callback: PP_CompletionCallback) -> int32_t>,
    pub FinishStreamingToFile: Option<extern "C" fn(loader: PP_Resource,
                                                    callback: PP_CompletionCallback) -> int32_t>,
    pub Close: Option<extern "C" fn(loader: PP_Resource)>,
}
impl ::std::clone::Clone for PPB_URLLoader_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_URLLoader_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
pub type PPB_URLLoader = PPB_URLLoader_1_0;
pub type Enum_Unnamed29 = ::libc::c_uint;
pub const PP_URLREQUESTPROPERTY_URL: ::libc::c_uint = 0;
pub const PP_URLREQUESTPROPERTY_METHOD: ::libc::c_uint = 1;
pub const PP_URLREQUESTPROPERTY_HEADERS: ::libc::c_uint = 2;
pub const PP_URLREQUESTPROPERTY_STREAMTOFILE: ::libc::c_uint = 3;
pub const PP_URLREQUESTPROPERTY_FOLLOWREDIRECTS: ::libc::c_uint = 4;
pub const PP_URLREQUESTPROPERTY_RECORDDOWNLOADPROGRESS: ::libc::c_uint = 5;
pub const PP_URLREQUESTPROPERTY_RECORDUPLOADPROGRESS: ::libc::c_uint = 6;
pub const PP_URLREQUESTPROPERTY_CUSTOMREFERRERURL: ::libc::c_uint = 7;
pub const PP_URLREQUESTPROPERTY_ALLOWCROSSORIGINREQUESTS: ::libc::c_uint = 8;
pub const PP_URLREQUESTPROPERTY_ALLOWCREDENTIALS: ::libc::c_uint = 9;
pub const PP_URLREQUESTPROPERTY_CUSTOMCONTENTTRANSFERENCODING: ::libc::c_uint = 10;
pub const PP_URLREQUESTPROPERTY_PREFETCHBUFFERUPPERTHRESHOLD: ::libc::c_uint = 11;
pub const PP_URLREQUESTPROPERTY_PREFETCHBUFFERLOWERTHRESHOLD: ::libc::c_uint = 12;
pub const PP_URLREQUESTPROPERTY_CUSTOMUSERAGENT: ::libc::c_uint = 13;
pub type PP_URLRequestProperty = Enum_Unnamed29;

#[repr(C)]
#[derive(Copy)]
pub struct PPB_URLRequestInfo_1_0 {
    pub Create: Option<extern "C" fn(instance: PP_Instance) -> PP_Resource>,
    pub IsURLRequestInfo: Option<extern "C" fn(resource: PP_Resource) -> PP_Bool>,
    pub SetProperty: Option<extern "C" fn(request: PP_Resource,
                                          property: PP_URLRequestProperty,
                                          value: PP_Var) -> PP_Bool>,
    pub AppendDataToBody: Option<extern "C" fn(request: PP_Resource,
                                               data: *const ::libc::c_void,
                                               len: uint32_t) -> PP_Bool>,
    pub AppendFileToBody: Option<extern "C" fn(request: PP_Resource,
                                               file_ref: PP_Resource,
                                               start_offset: int64_t,
                                               number_of_bytes: int64_t,
                                               expected_last_modified_time: PP_Time) -> PP_Bool>,
}
impl ::std::clone::Clone for PPB_URLRequestInfo_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_URLRequestInfo_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
pub type Enum_Unnamed30 = ::libc::c_uint;
pub const PP_URLRESPONSEPROPERTY_URL: ::libc::c_uint = 0;
pub const PP_URLRESPONSEPROPERTY_REDIRECTURL: ::libc::c_uint = 1;
pub const PP_URLRESPONSEPROPERTY_REDIRECTMETHOD: ::libc::c_uint = 2;
pub const PP_URLRESPONSEPROPERTY_STATUSCODE: ::libc::c_uint = 3;
pub const PP_URLRESPONSEPROPERTY_STATUSLINE: ::libc::c_uint = 4;
pub const PP_URLRESPONSEPROPERTY_HEADERS: ::libc::c_uint = 5;
pub type PP_URLResponseProperty = Enum_Unnamed30;

#[repr(C)]
#[derive(Copy)]
pub struct PPB_URLResponseInfo_1_0 {
    pub IsURLResponseInfo: Option<extern "C" fn(resource: PP_Resource) -> PP_Bool>,
    pub GetProperty: Option<extern "C" fn(response: PP_Resource,
                                          property: PP_URLResponseProperty) -> PP_Var>,
    pub GetBodyAsFileRef: Option<extern "C" fn(response: PP_Resource) -> PP_Resource>,
}
impl ::std::clone::Clone for PPB_URLResponseInfo_1_0 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_URLResponseInfo_1_0 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy)]
pub struct PPB_View_1_2 {
    pub IsView: Option<StubInterfaceFunc<PP_Bool>>,
    pub GetRect: Option<StubInterfaceFunc<PP_Bool>>,
    pub IsFullscreen: Option<StubInterfaceFunc<PP_Bool>>,
    pub IsVisible: Option<StubInterfaceFunc<PP_Bool>>,
    pub IsPageVisible: Option<StubInterfaceFunc<PP_Bool>>,
    pub GetClipRect: Option<StubInterfaceFunc<PP_Bool>>,
    pub GetDeviceScale: Option<StubInterfaceFunc<::libc::c_float>>,
    pub GetCSSScale: Option<StubInterfaceFunc<::libc::c_float>>,
    pub GetScrollOffset: Option<StubInterfaceFunc<::libc::c_float>>,
}
impl ::std::clone::Clone for PPB_View_1_2 {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for PPB_View_1_2 {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
