use std;

use super::sys::*;
use super::resource::{Resource, ResourceState};

pub trait ResultCode {
    fn into_code(&self) -> PP_Code;
}

impl ResultCode for Result<(), Error> {
    fn into_code(&self) -> PP_Code {
        match self {
            &Ok(()) => PP_OK,
            &Err(e) => {
                e.into()
            },
        }
    }
}
impl ResultCode for Result<usize, Error> {
    fn into_code(&self) -> PP_Code {
        match self {
            &Ok(bytes) => bytes as PP_Code,
            &Err(e) => {
                e.into()
            },
        }
    }
}
impl<T> ResultCode for Result<Resource<T>, Error>
    where T: ResourceState,
{
    fn into_code(&self) -> PP_Code {
        match self {
            &Ok(ref r) => r.id(),
            &Err(e) => {
                e.into()
            },
        }
    }
}

/// Put your gdb breakpoints here.
#[inline(never)] #[no_mangle] #[allow(unused_variables)]
pub extern "C" fn gdb_breakpoint_ppapi_error(code: i32) { }

pub type Code<T> = Result<T, Error>;

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum Error {
    CompletionPending, // = ffi::PP_OK_COMPLETIONPENDING,

    BadResource,       // = ffi::PP_ERROR_BADRESOURCE,
    BadArgument,       // = ffi::PP_ERROR_BADARGUMENT,
    WrongThread,       // = ffi::PP_ERROR_WRONG_THREAD,
    InProgress,        // = ffi::PP_ERROR_INPROGRESS,
    Failed,            // = ffi::PP_ERROR_FAILED,
    NotSupported,      // = ffi::PP_ERROR_NOTSUPPORTED,
    NoMemory,          // = ffi::PP_ERROR_NOMEMORY,
    NoSpace,           // = ffi::PP_ERROR_NOSPACE,
    NoQuota,           // = ffi::PP_ERROR_NOQUOTA,
    ContextLost,       // = ffi::PP_ERROR_CONTEXT_LOST,
    FileNotFound,      // = ffi::PP_ERROR_FILENOTFOUND,
    FileExists,        // = ffi::PP_ERROR_FILEEXISTS,
    NotAFile,
    NoAccess,          // = ffi::PP_ERROR_NOACCESS,
    ConnectionRefused, // = ffi::PP_ERROR_CONNECTION_REFUSED,
    ConnectionReset,   // = ffi::PP_ERROR_CONNECTION_RESET,
    ConnectionAborted, // = ffi::PP_ERROR_CONNECTION_ABORTED,
    ConnectionClosed,  // = ffi::PP_ERROR_CONNECTION_CLOSED,
    TimedOut,          // = ffi::PP_ERROR_TIMEDOUT,
    NoMessageLoop,     // = ffi::PP_ERROR_NO_MESSAGE_LOOP,
    ResourceFailed,    // = sys::PP_ERROR_RESOURCE_FAILED,

    /// See PP_ERROR_ABORTED.
    Aborted,

    /// See PP_ERROR_NOINTERFACE.
    NoInterface,
    /// The instance handle is no longer valid. This will happen after the
    /// instance is destroyed.
    BadInstance,
}

impl Into<i32> for Error {
    fn into(self) -> i32 {
        let code = match self {
            Error::CompletionPending => PP_OK_COMPLETIONPENDING,
            Error::BadResource       => PP_ERROR_BADRESOURCE,
            Error::BadArgument       => PP_ERROR_BADARGUMENT,
            Error::WrongThread       => PP_ERROR_WRONG_THREAD,
            Error::InProgress        => PP_ERROR_INPROGRESS,
            Error::Failed            => PP_ERROR_FAILED,
            Error::NotSupported      => PP_ERROR_NOTSUPPORTED,
            Error::NoMemory          => PP_ERROR_NOMEMORY,
            Error::ContextLost       => PP_ERROR_CONTEXT_LOST,
            Error::NoSpace           => PP_ERROR_NOSPACE,
            Error::NoQuota           => PP_ERROR_NOQUOTA,
            Error::FileNotFound      => PP_ERROR_FILENOTFOUND,
            Error::FileExists        => PP_ERROR_FILEEXISTS,
            Error::NotAFile          => PP_ERROR_NOTAFILE,
            Error::NoAccess          => PP_ERROR_NOACCESS,
            Error::ConnectionRefused => PP_ERROR_CONNECTION_REFUSED,
            Error::ConnectionReset   => PP_ERROR_CONNECTION_RESET,
            Error::ConnectionAborted => PP_ERROR_CONNECTION_ABORTED,
            Error::ConnectionClosed  => PP_ERROR_CONNECTION_CLOSED,
            Error::TimedOut          => PP_ERROR_TIMEDOUT,
            Error::NoMessageLoop     => PP_ERROR_NO_MESSAGE_LOOP,
            Error::NoInterface       => PP_ERROR_NOINTERFACE,
            Error::Aborted           => PP_ERROR_ABORTED,

            Error::BadInstance |
            Error::ResourceFailed    => PP_ERROR_RESOURCE_FAILED,
        };

        gdb_breakpoint_ppapi_error(code);

        code
    }
}
impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_: std::sync::PoisonError<T>) -> Error {
        Error::ResourceFailed
    }
}
