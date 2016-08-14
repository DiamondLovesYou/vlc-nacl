
use libc;

use std::cell::{RefCell};
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Mutex, Arc};

use super::sys;
use super::prelude::*;
use super::resource::{ResourceRc, ResState, take_resource_id,
                      get_resource};
use super::instance::Instance;
use super::interface::*;

pub type MessageLoop = Resource<MessageLoopState>;

/// NOTE: ONLY SEND FROM A CLONED `self.tx`.
#[derive(Debug)]
pub struct MessageLoopState {
    id:       PP_Resource,
    instance: Instance,
    rx:       Mutex<Option<Receiver<MlMsg>>>,
    tx:       Sender<MlMsg>,
    attached: AtomicBool,
    shutdown: AtomicBool,
    main:     bool,

    /// Only the message thread accesses this, so doesn't *need* a mutex, but Rust.
    message_handler_data: Mutex<Option<(&'static sys::PPP_MessageHandler_0_2,
                                        *mut ::libc::c_void)>>,
}

impl MessageLoopState {
    pub fn create(i: Instance, main: bool) -> Code<MessageLoop> {
        let id = take_resource_id();
        let (tx, rx) = channel();
        let inner = MessageLoopState {
            id: id,
            instance: i.clone(),
            rx: Mutex::new(Some(rx)),
            tx: tx,
            attached: AtomicBool::new(false),
            shutdown: AtomicBool::new(false),
            main: main,
            message_handler_data: Mutex::new(None),
        };
        Ok(Resource::create(&i, Arc::new(inner)))
    }

    /// Don't call.
    pub fn register_mh(&self, user: *mut libc::c_void,
                       handler: &'static sys::PPP_MessageHandler_0_2) -> Code<()> {
        let mgr_tx = self.tx.clone();
        let msg = MlMsg::RegisterMessageHandler {
            user: user,
            handler: handler,
        };

        if let Some(_) = mgr_tx.send(msg).ok() {
            Ok(())
        } else {
            Err(Error::ResourceFailed)
        }
    }
    pub fn unregister_mh(&self) {
        let (tx, rx) = channel();

        let mgr_tx = self.tx.clone();
        let msg = MlMsg::UnregisterMessageHandler { ret: tx, };

        let _ = mgr_tx.send(msg);
        let _ = rx.recv();
    }

    pub fn run(&self) -> Code<()> {
        ATTACHED.with(|attached| {
            let rx = {
                let b = attached.borrow();
                if let &Some(ref ml) = &*b {
                    if ml.id != self.id {
                        return Err(Error::WrongThread);
                    }
                    try!(ml.rx.lock()).take()
                } else {
                    return Err(Error::NoMessageLoop);
                }
            };
            if rx.is_none() {
                return Err(Error::InProgress);
            }
            let rx = rx.unwrap();

            while let Ok(msg) = rx.recv() {
                match msg {
                    MlMsg::Shutdown {
                        pause,
                    } => {
                        self.shutdown.store(!pause, Ordering::SeqCst);
                        if !pause {
                            attached.borrow_mut().take();
                        } else {
                            let b = attached.borrow();
                            let mut lock = try!(b.as_ref().unwrap().rx.lock());
                            *lock = Some(rx);
                        }
                        return Ok(());
                    },
                    MlMsg::Post {
                        f, user, result,
                    } => {
                        let result = result.into_code();
                        f(user, result);
                    },
                    MlMsg::Message {
                        ret, msg,
                    } => {
                        let handler = self.message_handler_data.lock()
                            .ok()
                            .and_then(|mut hnd_l| hnd_l.take() );
                        if handler.is_none() { continue; }
                        let handler = handler.unwrap();

                        let arg: sys::PP_Var = msg.into();
                        if let Some(ret) = ret {
                            let mut ret_v: sys::PP_Var = Default::default();

                            let hbm = handler.0.HandleBlockingMessage
                                .expect("PPP_MessageHandler missing HandleBlockingMessage");
                            hbm(self.instance.id(), handler.1,
                                &arg, &mut ret_v);

                            if let Ok(ret_v) = Var::from(ret_v) {
                                let _ = ret.send(ret_v);
                            } else {
                                // XXX
                            }
                        } else {
                            let hm = handler.0.HandleMessage
                                .expect("PPP_MessageHandler missing HandleMessage");

                            hm(self.instance.id(), handler.1, &arg);
                        }

                        // Deref:
                        let _arg = Var::from(arg);

                        let mut hl = self.message_handler_data.lock().unwrap();
                        if hl.is_some() {
                            if let Some(dtor) = handler.0.Destroy {
                                (dtor)(self.instance.id(), handler.1);
                            }
                        } else {
                            *hl = Some(handler);
                        }
                    },
                    MlMsg::RegisterMessageHandler {
                        user, handler,
                    } => {
                        {
                            let mut lock = self.message_handler_data.lock().unwrap();
                            if let Some(prev) = lock.take() {
                                if let Some(dtor) = prev.0.Destroy {
                                    dtor(self.instance.id(), prev.1);
                                }
                            }

                            *lock = Some((handler, user));
                        }
                    },
                    MlMsg::UnregisterMessageHandler {
                        ret,
                    } => {
                        {
                            let mut lock = self.message_handler_data.lock().unwrap();
                            if let Some(prev) = lock.take() {
                                if let Some(dtor) = prev.0.Destroy {
                                    dtor(self.instance.id(), prev.1);
                                }
                            }
                        }

                        let _ = ret.send(());
                    }
                }
            }

            Ok(())
        })
    }

    pub fn attach_to_current_thread(this: MessageLoop) -> Code<()> {
        if this.attached.load(Ordering::Relaxed) {
            return Err(Error::InProgress);
        }

        ATTACHED.with(|attached| {
            let mut b = attached.borrow_mut();
            if let &Some(_) = &*b {
                Err(Error::InProgress)
            } else {
                this.attached.store(true, Ordering::Relaxed);
                *b = Some(this);
                Ok(())
            }
        })
    }

    pub fn post(&self, f: sys::PP_CompletionCallback_Func,
                user: *mut libc::c_void) -> Code<()> {
        self.post_result(f, user, Ok(()))
    }
    pub fn post_result(&self, f: sys::PP_CompletionCallback_Func,
                       user: *mut libc::c_void,
                       result: Code<()>) -> Code<()> {
        let msg = MlMsg::Post {
            f: f,
            user: user,
            result: result,
        };
        let tx = self.tx.clone();
        if let Err(_) = tx.send(msg) {
            Err(Error::Failed)
        } else {
            Ok(())
        }
    }
    pub fn post_quit(&self, shutdown: bool) -> Code<()> {
        if self.main {
            return Err(Error::WrongThread);
        }
        let tx = self.tx.clone();
        tx
            .send(MlMsg::Shutdown {
                pause: !shutdown,
            })
            .map_err(|_| Error::Failed )
    }
}
unsafe impl Send for MessageLoopState { }
unsafe impl Sync for MessageLoopState { }
impl ResourceState for MessageLoopState {
    fn from_resstate(this: Arc<ResourceRc>) -> Code<Resource<Self>> {
        let state = try!(Self::state_from_resstate(&this))
            .clone();

        Ok(Resource::new(this, state))
    }
    fn into_resstate(this: Arc<Self>) -> ResState {
        ResState::MessageLoop(this)
    }
    fn state_from_resstate(rs: &Arc<ResourceRc>) -> Code<&Arc<Self>> {
        match rs.state() {
            &ResState::MessageLoop(ref r) => Ok(r),
            _ => Err(Error::BadArgument),
        }
    }
    fn resource_id(this: &Arc<Self>) -> PP_Resource {
        this.id
    }
    fn resource_instance(this: &Arc<Self>) -> Instance {
        this.instance.clone()
    }
}

thread_local!(static ATTACHED: RefCell<Option<MessageLoop>> = Default::default());

pub fn current_message_loop() -> Code<MessageLoop> {
    ATTACHED.with(|attached| {
        if let Some(msg_loop) = attached.borrow().clone() {
            Ok(msg_loop)
        } else {
            Err(Error::NoMessageLoop)
        }
    })
}


pub enum Callback {
    Async {
        /// Will not be null.
        f: sys::PP_CompletionCallback_Func,
        user: *mut libc::c_void,
        message_loop: MessageLoop,
    },
    Sync,
}
impl Callback {
    pub fn from_ffi(ffi: sys::PP_CompletionCallback) -> Code<Callback> {
        use std::intrinsics::transmute;
        if ffi.flags as u32 & sys::PP_COMPLETIONCALLBACK_FLAG_OPTIONAL != 0 {
            // TODO?
            return Err(Error::NotSupported);
        }

        let fp: *const () = unsafe { transmute(ffi.func) };

        if fp.is_null() {
            Ok(Callback::Sync)
        } else {
            let msg_loop = try!(current_message_loop());
            Ok(Callback::Async {
                f: ffi.func,
                user: ffi.user_data,
                message_loop: msg_loop,
            })
        }
    }

    pub fn blocking(&self) -> bool {
        match self {
            &Callback::Sync => true,
            _ => false,
        }
    }

    pub fn trigger<U>(self, result: Code<U>) -> Code<U> {
        match self {
            Callback::Sync => result,
            Callback::Async {
                f, user, message_loop: ml,
            } => {
                match result {
                    Ok(_) => {
                        let _ = ml.post_result(f, user, Ok(()));
                        return result;
                    },
                    Err(err) => {
                        let _ = ml.post_result(f, user, Err(err));
                        return result;
                    },
                }
            },
        }
    }
}

impl Default for Callback {
    fn default() -> Callback { Callback::Sync }
}

enum MlMsg {
    Shutdown {
        pause: bool,
    },
    Post {
        f: sys::PP_CompletionCallback_Func,
        user: *mut libc::c_void,
        result: Code<()>,
    },
    RegisterMessageHandler {
        user: *mut libc::c_void,
        handler: &'static sys::PPP_MessageHandler_0_2,
    },
    UnregisterMessageHandler {
        ret: Sender<()>,
    },
    Message {
        ret: Option<Sender<Var>>,
        msg: Var,
    },
}

unsafe impl Send for MlMsg { }

pub fn main_thread_init() {
    unimplemented!();
}

static ML_INTERFACE: sys::PPB_MessageLoop_1_0 = sys::PPB_MessageLoop_1_0 {
    Create: Some(ppb_ml_create),
    GetForMainThread: Some(ret_default_stub),
    GetCurrent: Some(ppb_ml_get_current),
    AttachToCurrentThread: Some(ppb_ml_attach_to_current_thread),
    Run: Some(ppb_ml_run),
    PostWork: Some(ppb_ml_post_work),
    PostQuit: Some(ppb_ml_post_quit),
};
pub static INTERFACES: Interfaces = &[
    ("PPB_MessageLoop;1.0", interface_ptr(&ML_INTERFACE)),
];

fn get(id: PP_Resource) -> Code<MessageLoop> { get_resource(id) }

extern "C" fn ppb_ml_create(instance: PP_Instance) -> PP_Resource {
    let i = super::ModuleInterface::get_instance_interface(instance);
    if i.is_err() { return 0; }
    let i = i.unwrap();

    let ml = i.create_message_loop();
    if ml.is_err() { return 0; }
    ml.unwrap().move_into_id()
}
extern "C" fn ppb_ml_get_current() -> PP_Resource {
    let ml = current_message_loop();
    if ml.is_err() { return 0; }
    ml.unwrap().move_into_id()
}
extern "C" fn ppb_ml_attach_to_current_thread(ml: PP_Resource) -> libc::int32_t {
    get(ml)
        .and_then(|ml| {
            MessageLoopState::attach_to_current_thread(ml)
        })
        .into_code()
}
extern "C" fn ppb_ml_run(ml: PP_Resource) -> libc::int32_t {
    get(ml)
        .and_then(|ml| {
            ml.run()
        })
        .into_code()
}
extern "C" fn ppb_ml_post_work(ml: PP_Resource,
                               callback: sys::PP_CompletionCallback,
                               delay_ms: libc::int64_t) -> libc::int32_t {
    let cb = Callback::from_ffi(callback);
    if let Err(e) = cb { return e.into(); }
    let cb = cb.unwrap();

    if cb.blocking() { return Error::BadArgument.into(); }

    if delay_ms != 0 { return Error::NotSupported.into(); }

    get(ml)
        .and_then(|ml| {
            ml.post(callback.func, callback.user_data)
        })
        .into_code()
}

extern "C" fn ppb_ml_post_quit(ml: PP_Resource,
                               shutdown: sys::PP_Bool) -> libc::int32_t {
    get(ml)
        .and_then(|ml| {
            ml.post_quit(shutdown != sys::PP_FALSE)
        })
        .into_code()
}
