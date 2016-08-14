
//! A simple implementation of the ppapi. This does not perform any actual IO;
//! instead it redirects it to memory.

#![feature(associated_type_defaults)]
#![feature(const_fn)]
#![feature(integer_atomics)]
#![feature(linkage)]

extern crate libc;
extern crate url;
#[macro_use]
extern crate log;
extern crate env_logger;

use self::sys::*;

use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{RwLock};
use std::sync::atomic::{AtomicPtr};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::{Duration, Instant, SystemTime};
use std::thread::JoinHandle;

pub use self::result::{Code, Error};
pub use self::instance::Instance;
pub use self::resource::{Resource, ResourceState};
pub use self::filesystem_manager::{FileIo, FileRef, FileSystem};

pub mod ppapi { pub use super::*; }

macro_rules! ppb_f {
    (R($res:expr), $callback:expr $(,$arg:expr)* => $fn_name:ident) => ({
        use ::result::ResultCode;
        let callback = match ::ppapi::callback::Callback::from_ffi($callback) {
            Ok(cb) => cb,
            Err(code) => {
                let err: ::ppapi::result::Code<()> = Err(code);
                return err.into_code();
            },
        };
        let ret = if let Some(instance) = ::ppapi::resource::get_resource_instance($res) {
            instance.$fn_name($res, $($arg,)* callback)
        } else {
            Err(::ppapi::result::Error::BadResource)
        };

        ret.into_code()
    })
}

// Must be public.
#[cfg(test)]
pub mod test;

pub mod sys;

pub mod audio;
pub mod console;
pub mod file_io;
pub mod file_ref;
pub mod resource;
pub mod instance;
pub mod result;
pub mod callback;
pub mod var;
pub mod filesystem_manager;
pub mod url_loader;
pub mod graphics;
pub mod mouse;
pub mod messaging;
pub mod view;

mod interface;
pub mod support;

pub mod prelude {
    pub use super::sys::{PP_Instance, PP_Resource, PP_VarId};
    pub use super::result::{Code, Error, ResultCode};

    pub use super::var::{Var, ArrayVar, StringVar, DictVar, ResVar};
    pub use super::resource::{Resource, WeakResource, ResourceState, RefCounted};
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct ModuleHandle(PP_Module);

#[derive(Clone)]
pub struct ModuleInterface {
    id: ModuleHandle,
    start_ts: Instant,
    tx: Sender<Message>,
}

impl ModuleInterface {
    pub fn create_instance(&self, args: Vec<(String, String)>) -> Code<Instance> {
        let (tx, rx) = channel();
        let msg = Message::CreateInstance {
            ret: tx,
            args: From::from(args),
        };
        self.tx.send(msg).unwrap();
        rx.recv()
            .unwrap()
            .map(|instance| {
                self::var::set_var_instance(instance.clone());
                instance
            })
    }

    /// Called only from the instance threads.
    fn destroy_instance(&self, id: PP_Instance, ret: Sender<Code<()>>) {
        self.tx.send(Message::DestroyInstance {
            id: id, ret: ret,
        }).unwrap();
    }

    pub fn get_instance_interface(id: PP_Instance) -> Code<Instance> {
        ModuleInstances::get(id).ok_or(Error::BadInstance)
    }

    pub fn id(&self) -> ModuleHandle { self.id }
    pub fn seconds_elapsed(&self) -> PP_TimeTicks {
        let elapsed = Instant::now().duration_since(self.start_ts);
        duration_to_seconds(elapsed)
    }
    pub fn wall_time(&self) -> PP_Time {
        use std::time::UNIX_EPOCH;
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();

        duration_to_seconds(elapsed)
    }
}

fn duration_to_seconds(elapsed: Duration) -> f64 {
    elapsed.as_secs() as f64 +
        elapsed.subsec_nanos() as f64 / 1_000_000_000f64
}

/// This must be independent of the module thread.
#[derive(Default)]
struct ModuleInstances(RwLock<HashMap<PP_Instance, (JoinHandle<()>, Instance)>>);
fn instances() -> &'static ModuleInstances { support::global_singleton_default() }
impl ModuleInstances {
    fn insert(id: PP_Instance, join: JoinHandle<()>, inst: Instance) {
        let res = {
            let mut lock = instances().0.write().unwrap();
            lock.insert(id, (join, inst))
        };

        // don't panic with the lock.
        assert!(res.is_none());
    }
    fn remove(id: PP_Instance) -> Option<(JoinHandle<()>, Instance)> {
        let mut lock = instances().0.write().unwrap();
        lock.remove(&id)
    }
    fn get(id: PP_Instance) -> Option<Instance> {
        let lock = instances().0.read().unwrap();
        lock.get(&id)
            .map(|&(_, ref i)| i.clone() )
    }
}

static INSTANCE_CALLBACKS: AtomicPtr<InstanceInterfaces> =
    AtomicPtr::new(0 as _);

#[derive(Clone, Copy)]
struct InstanceInterfaces {
    instance: Option<&'static PPP_Instance_1_1>,
}
impl InstanceInterfaces {
    pub fn instance() -> &'static PPP_Instance_1_1 {
        get_ppp().instance
            .expect("missing instance interface!")
    }
}
impl Default for InstanceInterfaces {
    fn default() -> InstanceInterfaces {
        use libc;
        use std::mem::transmute;

        extern {
            fn PPP_GetInterface(interface_name: *const libc::c_char) -> *const libc::c_void;
        }

        let iptr = unsafe { PPP_GetInterface("PPP_Instance;1.1\0".as_ptr() as *const i8) };
        let instance = unsafe { iptr.as_ref() };
        let instance = instance.map(|iptr| {
            let instance: &'static PPP_Instance_1_1 = unsafe { transmute(iptr) };
            instance
        });

        InstanceInterfaces {
            instance: instance,
        }
    }
}

fn get_ppp() -> &'static InstanceInterfaces {
    use std;
    use std::sync::atomic::{Ordering};
    use std::sync::{Once, ONCE_INIT};

    static START: Once = ONCE_INIT;

    START.call_once(|| {
        let inner: InstanceInterfaces = Default::default();
        let outer = Box::new(inner);
        let outer_ptr = Box::into_raw(outer);
        INSTANCE_CALLBACKS.store(outer_ptr, Ordering::SeqCst);
    });

    unsafe { std::mem::transmute(INSTANCE_CALLBACKS.load(Ordering::SeqCst)) }
}

fn take_instance_id() -> PP_Instance {
    use std::sync::atomic::{AtomicI32, Ordering};
    static NEXT_ID: AtomicI32 = AtomicI32::new(1);

    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

/// Call only from the module thread (ie don't call).
fn drop_instance(instance: PP_Instance) {
    let ppp = InstanceInterfaces::instance();
    (ppp.destroy)(instance);
}

struct ModuleState {
    this: ModuleInterface,

    rx: Receiver<Message>,
}
impl ModuleState {
    fn new(id: ModuleHandle) -> ModuleInterface {
        use std::thread::spawn;

        let (tx, rx) = channel();

        let this = ModuleInterface {
            id: id,
            start_ts: Instant::now(),
            tx: tx,
        };

        let state = ModuleState {
            rx: rx,
            this: this.clone(),
        };

        spawn(move || {
            state.run();
        });

        this
    }

    fn insert_instance(id: PP_Instance, join: JoinHandle<()>, i: Instance) {
        ModuleInstances::insert(id, join, i)
    }
    fn remove_instance(id: PP_Instance) -> Option<(JoinHandle<()>, Instance)> {
        ModuleInstances::remove(id)
    }

    fn run(self) {
        while let Ok(msg) = self.rx.recv() {
            match msg {
                Message::CreateInstance {
                    ret, args,
                } => {
                    let ppp = InstanceInterfaces::instance();

                    let id = take_instance_id();
                    let (join, instance) = self::instance::InstanceState::new(id, self.this.clone());
                    Self::insert_instance(id, join, instance.clone());

                    let success = (ppp.create)(id, args.len() as libc::uint32_t,
                                               args.argks_ptr() as *mut _,
                                               args.argvs_ptr() as *mut _);
                    let ret_v = if success == sys::PP_FALSE {
                        if let Some((join, _)) = Self::remove_instance(id) {
                            let _ = join.join();
                        }
                        Err(Error::Aborted)
                    } else {
                        Ok(instance)
                    };

                    let _ = ret.send(ret_v);
                },
                Message::DestroyInstance {
                    ret, id,
                } => {
                    // Call DidDestroy, stop the instance thread, *then* remove the instance from the global store.
                    let ret_v = ModuleInstances::get(id)
                        .ok_or(Error::BadArgument)
                        .and_then(|i| {
                            drop_instance(i.id());

                            i.stop();

                            let (join, _) = Self::remove_instance(i.id())
                                .unwrap();

                            join.join()
                                .map_err(|_| Error::Aborted )
                        });

                    let _ = ret.send(ret_v);
                },
            }
        }
    }
}

struct PreprocessedCArgs {
    args: Vec<(CString, CString)>,

    argk_ptrs: Vec<*const libc::c_char>,
    argv_ptrs: Vec<*const libc::c_char>,
}
impl PreprocessedCArgs {
    fn len(&self) -> usize { self.args.len() }
    fn argks_ptr(&self) -> *const *const libc::c_char {
        self.argk_ptrs.as_ptr()
    }
    fn argvs_ptr(&self) -> *const *const libc::c_char {
        self.argv_ptrs.as_ptr()
    }
}
impl From<Vec<(String, String)>> for PreprocessedCArgs {
    fn from(args: Vec<(String, String)>) -> PreprocessedCArgs {
        let args: Vec<_> = args.into_iter()
            .map(|(k, v)| (CString::new(k).unwrap(),
                           CString::new(v).unwrap()) )
            .collect();

        PreprocessedCArgs {
            argk_ptrs: args
                .iter()
                .map(|&(ref k, _)| k.as_ptr() )
                .collect(),
            argv_ptrs: args
                .iter()
                .map(|&(_, ref v)| v.as_ptr() )
                .collect(),

            args: args,
        }
    }
}

unsafe impl Send for PreprocessedCArgs { }

enum Message {
    CreateInstance {
        ret: Sender<Code<Instance>>,
        args: PreprocessedCArgs,
    },
    DestroyInstance {
        ret: Sender<Code<()>>,
        id: sys::PP_Instance,
    },
}

/// Creates a new module by calling PPP_InitializeModule. Should really only be called once.
/// Nb: the PPAPI says that PPP_ShutdownModule is basically never called, so we
/// don't call it at all.
fn new_module() -> Option<ModuleInterface> {
    use std::sync::atomic::{AtomicI32, Ordering};
    static NEXT_ID: AtomicI32 = AtomicI32::new(1);

    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

    extern "C" {
        fn PPP_InitializeModule(module: sys::PP_Module, get_i: sys::GetInterface) -> libc::int32_t;
    }

    let code = unsafe {
        PPP_InitializeModule(id, get_interface)
    };

    if code == sys::PP_OK {
        Some(ModuleState::new(ModuleHandle(id)))
    } else {
        None
    }
}

pub fn global_module() -> ModuleInterface {
    use std::sync::atomic::{AtomicPtr, Ordering};
    use std::sync::{Mutex, Once, ONCE_INIT};

    // Doesn't need to be atomic, but whatever. Still needs a mutex b/c `Sender`
    // isn't `Sync`.
    static MODULE: AtomicPtr<Mutex<ModuleInterface>> = AtomicPtr::new(0 as _);

    static INIT: Once = ONCE_INIT;
    INIT.call_once(|| {
        env_logger::init().unwrap();

        // fail loudly.
        let module = new_module().unwrap();
        let module = Mutex::new(module);
        let module = Box::new(module);
        let module = Box::into_raw(module);

        MODULE.store(module, Ordering::SeqCst);
    });

    let module = unsafe { MODULE.load(Ordering::SeqCst).as_ref().unwrap() };
    let lock = module.lock().unwrap();
    (*lock).clone()
}

#[derive(Clone, Copy)]
#[doc(hidden)]
pub struct SyncInterfacePtr(*const libc::c_void);
impl SyncInterfacePtr {
    pub const fn new<T>(interface: &'static T) -> SyncInterfacePtr {
        SyncInterfacePtr((interface as *const T) as *const libc::c_void)
    }
}
unsafe impl Sync for SyncInterfacePtr { }

#[doc(hidden)]
pub extern "C" fn get_interface(name: *const libc::c_char) -> *const libc::c_void {
    use std::slice::from_raw_parts;
    use std::str::from_utf8_unchecked;
    use libc::strlen;

    let name = unsafe {
        let len = strlen(name) as usize;
        from_utf8_unchecked(from_raw_parts(name as *const u8, len))
    };

    fn find_interface_inner(&(interface_name, interface_ptr): &(&str, SyncInterfacePtr),
                      name: &str) -> Option<*const libc::c_void> {
        if interface_name == name {
            Some(interface_ptr.0)
        } else {
            None
        }
    }
    fn find_interface(prev: Option<*const libc::c_void>,
                      interfaces: interface::Interfaces,
                      name: &str) -> Option<*const libc::c_void> {
        prev
            .or_else(|| {
                interfaces
                    .iter()
                    .filter_map(|entry| find_interface_inner(entry, name) )
                    .next()
            })
    }

    let r = None;
    let r = find_interface(r, file_io::INTERFACES,
                           name);
    let r = find_interface(r, file_ref::INTERFACES,
                           name);
    let r = find_interface(r, resource::INTERFACES,
                           name);
    let r = find_interface(r, audio::INTERFACES,
                           name);
    let r = find_interface(r, console::INTERFACES,
                           name);
    let r = find_interface(r, instance::INTERFACES,
                           name);
    let r = find_interface(r, filesystem_manager::INTERFACES,
                           name);
    let r = find_interface(r, var::INTERFACES,
                           name);
    let r = find_interface(r, callback::INTERFACES,
                           name);
    let r = find_interface(r, graphics::INTERFACES,
                           name);
    let r = find_interface(r, mouse::INTERFACES,
                           name);
    let r = find_interface(r, messaging::INTERFACES,
                           name);
    let r = find_interface(r, url_loader::INTERFACES,
                           name);
    let r = find_interface(r, view::INTERFACES,
                           name);

    r.unwrap_or(0 as *const libc::c_void)
}
