
use libc::{self};

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{PathBuf};
use std::sync::{Arc};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread::JoinHandle;

use super::callback::{Callback, MessageLoop};
use super::sys::{self, PP_FileInfo, PP_Time, PP_TimeTicks};
use super::resource::ResourceRc;
use super::filesystem_manager::{FileIo, FileRef, FileSystem,
                                FileRefResource, FileIoResource};
use super::prelude::*;
use super::interface::*;
use super::var::VarRc;

/// Do not use from the instance state thread. Nb: Instance can't be shared
/// between threads because Sender<> can't. We forcably get around this issue so
/// references to this object can still be used in resource states. The Instance
/// inside resources are still owned by the instance state thread!
#[derive(Clone, Debug)]
pub struct Instance {
    instance_id: PP_Instance,
    module_id: super::ModuleHandle,
    tx: Sender<Message>,
}
impl Instance {
    pub fn id(&self) -> PP_Instance { self.instance_id }

    pub fn ping(&self) -> Code<()> {
        let (tx, rx) = channel();

        let msg = Message::Ping(tx);
        if let Err(_) = self.tx.send(msg) {
            return Err(Error::BadInstance);
        }
        if let Err(_) = rx.recv() {
            return Err(Error::BadInstance);
        }

        Ok(())
    }

    #[doc(hidden)]
    pub fn stop(&self) {
        let _ = self.tx.send(Message::Stop);
    }

    /// Do not call any other function after calling this.
    pub unsafe fn destroy(&self) -> Code<()> {
        let (tx, rx) = channel();
        let msg = Message::Destroy {
            ret: tx,
        };
        if let Err(_) = self.tx.send(msg) {
            return Err(Error::BadInstance);
        }

        rx.recv().unwrap()
    }

    pub fn resource_ctor(&self, res: Arc<ResourceRc>) {
        let msg = Message::ResourceCtor(res);
        let _ = self.tx.send(msg);
    }
    pub fn resource_dtor(&self, res: Arc<ResourceRc>) {
        let msg = Message::ResourceDtor(res);
        let _ = self.tx.send(msg);
    }
    pub fn track_var(&self, id: PP_VarId) {
        let msg = Message::TrackVar(id);
        let _ = self.tx.send(msg);
    }

    pub fn get_live_vars(&self) -> Code<Vec<VarRc>> {
        let (tx, rx) = channel();
        let msg = Message::GetLiveVars {
            ret: tx,
        };

        if let Some(vars) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            let vars = try!(vars);
            let vars = super::var::read_vars(move |vs| {
                let vars: Vec<_> = vars.into_iter()
                    .filter_map(|v| vs.get(v) )
                    .collect();
                Ok(vars)
            });
            Ok(try!(vars))
        } else {
            Err(Error::BadInstance)
        }
    }

    pub fn create_message_loop(&self) -> Code<MessageLoop> {
        super::callback::MessageLoopState::create(self.clone(), false)
    }

    pub fn create_file_system(&self) -> Code<FileSystem> {
        let (tx, rx) = channel();
        let msg = Message::CreateFileSystem(tx);

        if let Some(fs) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fs))
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn open_file_system(&self, _fs: PP_Resource, callback: Callback) -> Code<()> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::OpenFileSystem(tx);

        if let Some(result) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            result
        } else {
            Err(Error::BadInstance)
        }
    }

    pub fn create_file_ref(&self, fs: PP_Resource,
                           path: PathBuf) -> Code<FileRef> {
        let (tx, rx) = channel();
        let msg = Message::CreateFileRef(tx, fs, path);

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr))
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn get_name_file_ref(&self, fr: PP_Resource) -> Code<Var> {
        let (tx, rx) = channel();
        let msg = Message::GetNameFileRef {
            ret: tx,
            file_ref: fr,
        };

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr).into())
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn get_path_file_ref(&self, fr: PP_Resource) -> Code<Var> {
        let (tx, rx) = channel();
        let msg = Message::GetPathFileRef {
            ret: tx,
            file_ref: fr,
        };

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr).into())
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn get_parent_file_ref(&self, fr: PP_Resource) -> Code<FileRef> {
        let (tx, rx) = channel();
        let msg = Message::GetParentFileRef {
            ret: tx,
            file_ref: fr,
        };

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr))
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn mkdir_file_ref(&self, fr: PP_Resource, flags: u32,
                          callback: Callback) -> Code<()> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::MkDirFileRef {
            ret: tx,
            file_ref: fr,
            flags: flags,
        };

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr))
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn touch_file_ref(&self, fr: PP_Resource, last_access_time: PP_Time,
                          last_modified_time: PP_Time,
                          callback: Callback) -> Code<()> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::TouchFileRef {
            ret: tx,
            file_ref: fr,
            last_access_time: last_access_time,
            last_modified_time: last_modified_time,
        };

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr))
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn delete_file_ref(&self, fr: PP_Resource,
                           callback: Callback) -> Code<()> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::DeleteFileRef {
            ret: tx,
            file_ref: fr,
        };

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr))
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn rename_file_ref(&self, fr: PP_Resource, new_fr: PP_Resource,
                           callback: Callback) -> Code<()> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::RenameFileRef {
            ret: tx,
            file_ref: fr,
            new_file_ref: new_fr,
        };

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr))
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn query_file_ref(&self, fr: PP_Resource, callback: Callback) -> Code<PP_FileInfo> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::QueryFileRef {
            ret: tx,
            file_ref: fr,
        };

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr))
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn read_dir_entries_file_ref(&self, fr: PP_Resource,
                                     callback: Callback) -> Code<Vec<FileRef>> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::ReadDirEntriesFileRef {
            ret: tx,
            file_ref: fr,
        };

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr))
        } else {
            Err(Error::BadInstance)
        }
    }

    pub fn create_file_io(&self) -> Code<FileIo> {
        let (tx, rx) = channel();
        let msg = Message::CreateFileIo {
            ret: tx,
        };
        self.tx.send(msg).unwrap();
        rx.recv().unwrap()
    }
    pub fn open_file_io(&self, res: PP_Resource, file_ref: PP_Resource,
                        flags: u32, callback: Callback) -> Code<()> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::OpenFileIo {
            ret: tx,
            io: res,
            file_ref: file_ref,
            flags: flags,
        };
        if let Err(_) = self.tx.send(msg) {
            return Err(Error::BadInstance);
        }
        rx.recv().unwrap()
    }
    pub fn is_file_io(&self, _res: PP_Resource) -> bool {
        unimplemented!();
    }
    pub fn query_file_io(&self, res: PP_Resource, dest: *mut sys::PP_FileInfo,
                         callback: Callback) -> Code<()> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::QueryFileIo {
            ret: tx,
            io: res,
        };
        if let Err(_) = self.tx.send(msg) {
            return Err(Error::BadInstance);
        }
        let ret = rx.recv();
        if let Err(_) = ret {
            return Err(Error::BadInstance);
        }
        let info = try!(ret.unwrap());

        if let Some(dest) = unsafe { dest.as_mut() } {
            *dest = info;
        }

        Ok(())
    }
    pub fn touch_file_io(&self, _res: PP_Resource, _last_access_time: PP_Time,
                         _last_modified_time: PP_Time,
                         _callback: Callback) -> Code<()> {
        Err(Error::NotSupported)
    }
    pub fn read_file_io(&self, res: PP_Resource, offset: usize,
                        buffer: &'static mut [u8],
                        callback: Callback) -> Code<usize> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::ReadFileIo {
            ret: tx,
            io: res,
            offset: offset,
            buffer: buffer,
        };
        if let Err(_) = self.tx.send(msg) {
            return Err(Error::BadInstance);
        }
        rx.recv().unwrap()
    }
    pub fn write_file_io(&self, res: PP_Resource, offset: usize,
                         buffer: &'static [u8],
                         callback: Callback) -> Code<usize> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::WriteFileIo {
            ret: tx,
            io: res,
            offset: offset,
            buffer: buffer,
        };
        if let Err(_) = self.tx.send(msg) {
            return Err(Error::BadInstance);
        }
        rx.recv().unwrap()
    }
    pub fn set_length_file_io(&self, res: PP_Resource, len: usize,
                              callback: Callback) -> Code<()> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::SetLengthFileIo {
            ret: tx,
            io: res,
            new_length: len,
        };
        if let Err(_) = self.tx.send(msg) {
            return Err(Error::BadInstance);
        }
        try!(rx.recv().unwrap());
        Ok(())
    }
    pub fn flush_file_io(&self, res: PP_Resource, callback: Callback) -> Code<()> {
        if !callback.blocking() { return Err(Error::NotSupported); }

        let (tx, rx) = channel();
        let msg = Message::FlushFileIo {
            ret: tx,
            io: res,
        };
        if let Err(_) = self.tx.send(msg) {
            return Err(Error::BadInstance);
        }
        rx.recv().unwrap()
    }
    pub fn close_file_io(&self, res: PP_Resource) {
        let (tx, rx) = channel();
        let msg = Message::CloseFileIo {
            ret: tx,
            io: res,
        };
        if let Err(_) = self.tx.send(msg) {
            return;
        }
        rx.recv().unwrap();
    }

    pub fn post_message(&self, msg: Var) {
        let msg = Message::PostMessage(msg);
        let _ = self.tx.send(msg);
    }

    pub fn register_message_handler(&self, user: *mut libc::c_void,
                                    handler: &'static sys::PPP_MessageHandler_0_2,
                                    ml: MessageLoop) -> Code<()> {
        let (tx, rx) = channel();
        let msg = Message::RegisterMessageHandler {
            ret: tx,
            user: user,
            handler: handler,
            ml: ml,
        };

        if let Some(fr) = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            })
        {
            Ok(try!(fr))
        } else {
            Err(Error::BadInstance)
        }
    }
    pub fn unregister_message_handler(&self) {
        let (tx, rx) = channel();
        let msg = Message::UnregisterMessageHandler(tx);

        let _ = self.tx.send(msg)
            .ok()
            .and_then(|_| {
                rx.recv().ok()
            });
    }
}
unsafe impl Sync for Instance { }
impl Hash for Instance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}
impl PartialEq for Instance {
    fn eq(&self, rhs: &Instance) -> bool {
        self.id() == rhs.id()
    }
}
impl Eq for Instance { }

enum Message {
    Ping(Sender<()>),
    Destroy {
        /// This isn't used by the instance thread; it's passed on to the module thread.
        ret: Sender<Code<()>>,
    },
    Stop,
    ResourceCtor(Arc<ResourceRc>),
    ResourceDtor(Arc<ResourceRc>),

    TrackVar(PP_VarId),

    GetLiveVars {
        ret: Sender<Code<Vec<PP_VarId>>>,
    },

    PostMessage(Var),
    RegisterMessageHandler {
        ret: Sender<Code<()>>,
        user: *mut libc::c_void,
        handler: &'static sys::PPP_MessageHandler_0_2,
        ml: MessageLoop,
    },
    UnregisterMessageHandler(Sender<()>),

    CreateFileSystem(Sender<Code<FileSystem>>),
    OpenFileSystem(Sender<Code<()>>),

    CreateFileRef(Sender<Code<FileRef>>, PP_Resource, PathBuf),
    GetNameFileRef {
        ret: Sender<Code<StringVar>>,
        file_ref: PP_Resource,
    },
    GetPathFileRef {
        ret: Sender<Code<StringVar>>,
        file_ref: PP_Resource,
    },
    GetParentFileRef {
        ret: Sender<Code<FileRef>>,
        file_ref: PP_Resource,
    },
    MkDirFileRef {
        ret: Sender<Code<()>>,
        file_ref: PP_Resource,
        flags: u32,
    },
    TouchFileRef {
        ret: Sender<Code<()>>,
        file_ref: PP_Resource,
        last_access_time: PP_Time,
        last_modified_time: PP_Time,
    },
    DeleteFileRef {
        ret: Sender<Code<()>>,
        file_ref: PP_Resource,
    },
    RenameFileRef {
        ret: Sender<Code<()>>,
        file_ref: PP_Resource,
        new_file_ref: PP_Resource,
    },
    QueryFileRef {
        ret: Sender<Code<PP_FileInfo>>,
        file_ref: PP_Resource,
    },
    ReadDirEntriesFileRef {
        ret: Sender<Code<Vec<FileRef>>>,
        file_ref: PP_Resource,
    },

    CreateFileIo {
        ret: Sender<Code<FileIo>>,
    },
    OpenFileIo {
        ret: Sender<Code<()>>,
        io: PP_Resource,
        file_ref: PP_Resource,
        flags: u32,
    },
    QueryFileIo {
        ret: Sender<Code<PP_FileInfo>>,
        io: PP_Resource,
    },
    ReadFileIo {
        ret: Sender<Code<usize>>,
        io: PP_Resource,
        offset: usize,
        buffer: &'static mut [u8],
    },
    WriteFileIo {
        ret: Sender<Code<usize>>,
        io: PP_Resource,
        offset: usize,
        buffer: &'static [u8],
    },
    SetLengthFileIo {
        ret: Sender<Code<usize>>,
        io: PP_Resource,
        new_length: usize,
    },
    FlushFileIo {
        ret: Sender<Code<()>>,
        io: PP_Resource,
    },
    CloseFileIo {
        ret: Sender<()>,
        io: PP_Resource,
    },
}
/// Short-circuit a recursion limit error in rustc.
unsafe impl Send for Message { }

pub struct InstanceState {
    parent: super::ModuleInterface,
    rx: Receiver<Message>,
    this: Instance,

    resources: HashMap<PP_Resource, Arc<ResourceRc>>,
    vars: HashSet<PP_VarId>,

    post_msg_dest: Option<Sender<Var>>,

    temp_fs_man: FileSystem,

    message_handler: Option<MessageLoop>,
}

impl InstanceState {
    #[doc(hidden)]
    pub fn new(id: PP_Instance, parent: super::ModuleInterface) -> (JoinHandle<()>, Instance) {
        use super::filesystem_manager::FileSystemState;
        use std::thread::*;
        let (tx, rx) = channel();
        let this = Instance {
            instance_id: id,
            module_id:   parent.id(),
            tx:          tx,
        };

        let mut state = InstanceState {
            parent: parent,
            rx: rx,
            this: this.clone(),
            resources: Default::default(),
            vars: Default::default(),
            temp_fs_man: FileSystemState::new(&this),
            message_handler: None,
            post_msg_dest: None,
        };

        state.resources.insert(state.temp_fs_man.id(), state.temp_fs_man.get_rc().clone());

        let join = spawn(move || {
            let mut state = state;
            state.run();
        });

        (join, this)
    }

    pub fn seconds_elapsed(&self) -> PP_TimeTicks {
        self.parent.seconds_elapsed()
    }
    pub fn wall_time(&self) -> PP_Time {
        self.parent.wall_time()
    }

    fn with_typed_resource<F, T, U, V>(&self, previous: Code<V>, id: PP_Resource,
                                       f: F) -> Code<U>
        where F: FnOnce(Resource<T>, V) -> Code<U>,
              T: ResourceState,
    {
        if let Some(res) = self.resources.get(&id) {
            let res = res.clone();
            let res = try!(<T as ResourceState>::from_resstate(res));
            f(res, try!(previous))
        } else {
            Err(Error::BadArgument)
        }
    }

    fn run(&mut self) {
        use self::Message::*;
        super::var::set_var_instance(self.this.clone());

        loop {
            let msg = match self.rx.recv() {
                Ok(msg) => msg,
                Err(_) => {
                    return;
                },
            };

            match msg {
                Ping(ret) => {
                    let _ = ret.send(());
                },
                Destroy { ret, } => {
                    // TODO XXX XXX

                    self.parent.destroy_instance(self.this.id(), ret);
                },
                Stop => {
                    return;
                },
                Message::ResourceCtor(res) => {
                    self.resources.insert(res.id(), res);
                },
                Message::ResourceDtor(res) => {
                    let id = res.id();
                    if id == self.temp_fs_man.id() {
                        self.temp_fs_man.close();
                        continue;
                    }

                    if self.temp_fs_man.opened() {
                        self.temp_fs_man.resource_dtor(res);
                    }
                },
                Message::TrackVar(id) => {
                    self.vars.insert(id);
                },
                Message::GetLiveVars { ret, } => {
                    let vars: Vec<_> = self.vars.iter().map(|&v| v ).collect();

                    let _ = ret.send(Ok(vars));
                },

                Message::PostMessage(msg) => {
                    if let Some(tx) = self.post_msg_dest.take() {
                        if tx.send(msg).is_ok() {
                            self.post_msg_dest = Some(tx);
                        }
                    }
                },
                Message::RegisterMessageHandler {
                    ret, user, handler, ml,
                } => {
                    let ret_v = ml.register_mh(user, handler);
                    self.message_handler = ret_v
                        .map(move |_| ml )
                        .ok();

                    let _ = ret.send(ret_v);
                },
                Message::UnregisterMessageHandler(ret) => {
                    if let Some(ml) = self.message_handler.take() {
                        ml.unregister_mh();
                    }

                    let _ = ret.send(());
                },

                CreateFileSystem(ret) => {
                    let ret_v = self.temp_fs_man
                        .create()
                        .map(|_| self.temp_fs_man.clone() );
                    let _ = ret.send(ret_v);
                },
                OpenFileSystem(ret) => {
                    let ret_v = self.temp_fs_man
                        .open();
                    let _ = ret.send(ret_v);
                },

                Message::CreateFileRef(ret, fs, path) => {
                    if fs == self.temp_fs_man.id() {
                        match self.temp_fs_man.create_file_ref(path.as_ref()) {
                            Ok(res) => {
                                self.resources.insert(res.id(), res.get_rc().clone());
                                let _ = ret.send(Ok(res));
                            },
                            Err(code) => {
                                let _ = ret.send(Err(code));
                            },
                        }
                    } else {
                        let _ = ret.send(Err(Error::NotSupported));
                    }
                },
                Message::GetNameFileRef {
                    ret, file_ref,
                } => {
                    let send = self.with_typed_resource(Ok(()), file_ref,
                                                        |file_ref: FileRef, _| {
                                                            Ok(file_ref.get_name())
                                                        });

                    let _ = ret.send(send);
                },
                GetPathFileRef {
                    ret, file_ref,
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), file_ref,
                                             |file_ref: FileRef, _| {
                                                 Ok(file_ref.get_path())
                                             });
                    let _ = ret.send(ret_v);
                },
                GetParentFileRef {
                    ret, file_ref,
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), file_ref,
                                             |file_ref: FileRef, _| {
                                                 file_ref.parent()
                                             });
                    let _ = ret.send(ret_v);
                },
                MkDirFileRef {
                    ret, file_ref, flags
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), file_ref,
                                             |file_ref: FileRef, _| {
                                                 let owner = try!(file_ref.owner());
                                                 owner.mkdir_file_ref(file_ref, flags)
                                             });
                    let _ = ret.send(ret_v);
                },
                TouchFileRef {
                    ret, file_ref, last_access_time, last_modified_time,
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), file_ref,
                                             |file_ref: FileRef, _| {
                                                 let owner = try!(file_ref.owner());
                                                 owner.touch_file_ref(file_ref.id(),
                                                                      last_access_time,
                                                                      last_modified_time)
                                             });
                    let _ = ret.send(ret_v);
                },
                DeleteFileRef {
                    ret, file_ref,
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), file_ref,
                                             |file_ref: FileRef, _| {
                                                 let owner = try!(file_ref.owner());
                                                 owner.delete_file_ref(file_ref.id())
                                             });
                    let _ = ret.send(ret_v);
                },
                RenameFileRef {
                    ret, file_ref, new_file_ref
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), file_ref,
                                             |file_ref: FileRef, _| {
                                                 let owner = try!(file_ref.owner());
                                                 owner.rename_file_ref(file_ref.id(),
                                                                       new_file_ref)
                                             });
                    let _ = ret.send(ret_v);
                },

                QueryFileRef {
                    ret, file_ref,
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), file_ref,
                                             |file_ref: FileRef, _| {
                                                 let owner = try!(file_ref.owner());
                                                 owner.query_file_ref(file_ref.id())
                                             });
                    let _ = ret.send(ret_v);
                },
                ReadDirEntriesFileRef {
                    ret, file_ref,
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), file_ref,
                                             |file_ref: FileRef, _| {
                                                 let owner = try!(file_ref.owner());
                                                 owner.read_dir_entries_file_ref(file_ref.id())
                                             });
                    let _ = ret.send(ret_v);
                },

                CreateFileIo {
                    ret,
                } => {
                    let ret_v = self.temp_fs_man.create_file_io();
                    let _ = ret.send(ret_v);
                },
                OpenFileIo {
                    ret, io, file_ref, flags,
                } => {
                    let ret_v = self.temp_fs_man.open_file_io(self, io, file_ref, flags);
                    let _ = ret.send(ret_v);
                },
                QueryFileIo {
                    ret, io,
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), io,
                                             |io: FileIo, _| {
                                                 let owner = try!(io.owner());
                                                 owner.query_file_io(io.id())
                                             });
                    let _ = ret.send(ret_v);
                },
                ReadFileIo {
                    ret, io, offset, buffer,
                } => {
                    let istate = &*self;
                    let ret_v = self
                        .with_typed_resource(Ok(()), io,
                                             move |io: FileIo, _| {
                                                 let owner = try!(io.owner());
                                                 owner.read_file_io(istate, io.id(), offset, buffer)
                                             });
                    let _ = ret.send(ret_v);
                },
                WriteFileIo {
                    ret, io, offset, buffer,
                } => {
                    let istate = &*self;
                    let ret_v = self
                        .with_typed_resource(Ok(()), io,
                                             move |io: FileIo, _| {
                                                 let owner = try!(io.owner());
                                                 owner.write_file_io(istate, io.id(), offset, buffer)
                                             });
                    let _ = ret.send(ret_v);
                },
                SetLengthFileIo {
                    ret, io, new_length,
                } => {
                    let istate = &*self;
                    let ret_v = self
                        .with_typed_resource(Ok(()), io,
                                             |io: FileIo, _| {
                                                 let owner = try!(io.owner());
                                                 owner.set_length_file_io(istate, io.id(), new_length)
                                             });
                    let _ = ret.send(ret_v);
                },
                FlushFileIo {
                    ret, io,
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), io,
                                             |io: FileIo, _| {
                                                 let owner = try!(io.owner());
                                                 owner.flush_file_io(io.id())
                                             });
                    let _ = ret.send(ret_v);
                },
                CloseFileIo {
                    ret, io,
                } => {
                    let ret_v = self
                        .with_typed_resource(Ok(()), io,
                                             |io: FileIo, _| {
                                                 let owner = try!(io.owner());
                                                 owner.close_file_io(io)
                                             });
                    let _ = ret.send(ret_v.ok().unwrap_or_default());
                },
            };
        }
    }
}

static INSTANCE_INTERFACE: sys::PPB_Instance_1_0 = sys::PPB_Instance_1_0 {
    BindGraphics: Some(ret_false_stub),
    IsFullFrame: Some(ret_false_stub),
};

pub static INTERFACES: Interfaces = &[
    ("PPB_Instance;1.0", interface_ptr(&INSTANCE_INTERFACE)),
];
