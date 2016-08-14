use libc;

use std;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::ffi::{OsStr, OsString};
use std::sync::{Arc, RwLock, Weak, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::{Path, PathBuf};
use std::ops::Deref;

use super::resource::{take_resource_id, get_resource_arc};
use super::resource::{ResState, ResourceRc};
use super::instance::{Instance, InstanceState};
use super::prelude::*;
use super::sys::{self, PP_FileInfo, PP_FileSystemType, PP_Time, PP_Bool, PP_TRUE, PP_FALSE,
                 PP_CompletionCallback};
use super::interface::*;

pub enum IoLimitType {
    Repeated {
        /// None means forever.
        count: Option<usize>,
    }
}
pub enum IoLimit {
    /// Only read or write up to N bytes.
    Bytes(usize),
}
pub struct IoLimitation {
    /// Some(0) means forever.
    pub io: IoLimit,
    pub limit: IoLimitType,
}

#[derive(Debug)]
pub enum FileRefType {
    File {
        data: Vec<u8>,

        // io_limits: RwLock<VecDeque<IoLimitation>>,
    },
    Dir {
        entries: HashSet<OsString>,
    },
    DoesNotExist,
}
impl FileRefType {
    pub fn len(&self) -> usize {
        match self {
            &FileRefType::File { ref data } => data.len(),
            _ => 0,
        }
    }
}

#[derive(Debug)]
pub struct FileRef_ {
    id: PP_Resource,
    owner: FileSystem,
    /// Only `None` for the root (obviously).
    parent: Option<FileRef>,

    name_var: Option<StringVar>,
    path_var: Option<StringVar>,

    path: PathBuf,
    creation_time: f64,
    last_access_time: f64,
    last_modified_time: f64,
    data: FileRefType,

    current_uses: AtomicUsize,
}

impl FileRef_ {
    pub fn file_type(&self) -> u32 {
        match self.data {
            FileRefType::File { .. } => sys::PP_FILETYPE_REGULAR,
            FileRefType::Dir { .. }  => sys::PP_FILETYPE_DIRECTORY,
            FileRefType::DoesNotExist => sys::PP_FILETYPE_OTHER,
        }
    }

    pub fn is_file(&self) -> bool {
        match self.data {
            FileRefType::File { .. } => true,
            _ => false,
        }
    }
    pub fn is_dir(&self) -> bool {
        match self.data {
            FileRefType::Dir { .. } => true,
            _ => false,
        }
    }
    pub fn exists(&self) -> bool {
        match self.data {
            FileRefType::DoesNotExist => false,
            _ => true,
        }
    }
}

pub type FileRef = Resource<FileRefState>;
pub type WeakFileRef = WeakResource<FileRefState>;
#[derive(Debug)]
pub struct FileRefState(RwLock<FileRef_>, Instance, PP_Resource, AtomicUsize);
impl FileRefState {
    pub fn id(&self) -> PP_Resource {
        self.2
    }
    pub fn path(&self) -> Code<PathBuf> {
        self.read_inner(|inner| {
            Ok(inner.path.clone())
        })
    }

    pub fn read_inner<F, U>(&self, f: F) -> Code<U>
        where F: FnOnce(&FileRef_) -> Code<U>,
    {
        let l = try!(self.0.read());
        f(&*l)
    }
    pub fn write_inner<F, U>(&self, f: F) -> Code<U>
        where F: FnOnce(&mut FileRef_) -> Code<U>,
    {
        let mut lock = try!(self.0.write());
        f(&mut *lock)
    }

    pub fn get_name(&self) -> StringVar {
        let s = {
            let lock = self.0.read().unwrap();
            if let Some(v) = lock.name_var.as_ref() {
                return v.clone()
            } else {
                format!("{}", lock.path.display())
            }
        };

        let s = StringVar::new(s);

        {
            let mut lock = self.0.write().unwrap();
            lock.name_var = Some(s.clone());
        }

        s
    }
    pub fn get_path(&self) -> StringVar {
        let s = {
            let lock = self.0.read().unwrap();
            if let Some(v) = lock.name_var.as_ref() {
                return v.clone()
            } else {
                let path = lock.path
                    .file_name()
                    .unwrap();
                let path = Path::new(path);
                format!("{}", path.display())
            }
        };

        let s = StringVar::new(s);

        {
            let mut lock = self.0.write().unwrap();
            lock.path_var = Some(s.clone());
        }

        s
    }

    pub fn uses(&self) -> Code<usize> {
        Ok(self.3.load(Ordering::SeqCst))
    }
    fn _use(&self) {
        self.3.fetch_add(1, Ordering::SeqCst);
    }
    fn _unuse(&self) {
        self.3.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn file_type(&self) -> Code<u32> {
        self.read_inner(|i| Ok(i.file_type()) )
    }

    pub fn is_file(&self) -> bool {
        self.is_file_error().unwrap_or(false)
    }
    pub fn is_file_error(&self) -> Code<bool> {
        self.read_inner(|inner| {
            match inner.data {
                FileRefType::File { .. } => Ok(true),
                _ => Ok(false),
            }
        })
    }
    pub fn is_dir(&self) -> bool {
        self.is_dir_error().unwrap_or(false)
    }
    pub fn is_dir_error(&self) -> Code<bool> {
        self.read_inner(|inner| {
            Ok(inner.is_dir())
        })
    }
    pub fn exists(&self) -> bool {
        self.read_inner(|inner| {
            match inner.data {
                FileRefType::DoesNotExist => Ok(false),
                _ => Ok(true),
            }
        })
            .unwrap_or(false)
    }

    pub fn query(&self, fs_type: PP_FileSystemType) -> Code<PP_FileInfo> {
        self.read_inner(|inner| {
            match inner.data {
                FileRefType::DoesNotExist => { return Err(Error::FileNotFound); },
                _ => {},
            }

            let mut dest: PP_FileInfo = Default::default();

            dest._type = inner.file_type();
            dest.size = inner.data.len() as i64;
            dest.system_type = fs_type;
            dest.creation_time = inner.creation_time;
            dest.last_access_time = inner.last_access_time;
            dest.last_modified_time = inner.last_modified_time;

            Ok(dest)
        })
    }
}

pub trait FileRefResource {
    fn owner(&self) -> Code<FileSystem>;
    fn parent(&self) -> Code<FileRef>;
    fn with_parents<F>(&self, f: F) -> Code<()>
        where F: FnMut(&FileRef) -> Code<()>;
}
impl FileRefResource for FileRef {
    fn owner(&self) -> Code<FileSystem> {
        self.deref()
            .read_inner(|inner| {
                Ok(inner.owner.clone())
            })
    }
    fn parent(&self) -> Code<FileRef> {
        self.deref()
            .read_inner(|inner| {
                let parent = inner.parent
                    .as_ref()
                    .unwrap_or(self);
                Ok(parent.clone())
            })
    }
    fn with_parents<F>(&self, mut f: F) -> Code<()>
        where F: FnMut(&FileRef) -> Code<()>,
    {
        let parent = {
            let lock = try!(self.deref().0.read());
            lock.parent.clone()
        };
        if let Some(parent) = parent {
            // Nb: `f` must be passed as a trait ref (ie virtually).
            try!(parent.with_parents(&mut f as &mut FnMut(&FileRef) -> Code<()>));
        }

        f(self)
    }
}

pub type FileIo = Resource<FileIoState>;
#[derive(Debug)]
pub struct FileIoState {
    id: PP_Resource,
    instance: Instance,
    data: Mutex<Option<(FileRef, bool, bool)>>,
}
impl FileIoState {
    pub fn id(&self) -> PP_Resource { self.id }
    /// f(file_ref, readable, writable)
    pub fn with_file_ref<F, U>(&self, f: F) -> Code<U>
        where F: FnOnce(&FileRef, bool, bool) -> Code<U>,
    {
        let lock = try!(self.data.lock());
        lock.as_ref()
            .ok_or(Error::Failed)
            .and_then(|&(ref fr, readable, writable)| {
                f(fr, readable, writable)
            })
    }


    pub fn close(&self) -> Code<()> {
        let mut lock = try!(self.data.lock());
        if let Some((fr, _, _)) = lock.take() {
            fr.with_parents(|fr| {
                fr._unuse();
                Ok(())
            })
        } else {
            Ok(())
        }
    }
}
pub trait FileIoResource {
    fn owner(&self) -> Code<FileSystem>;

    fn _open_file_io(&self, istate: &InstanceState,
                    fr: FileRef, create: bool,
                    readable: bool, writable: bool,
                    trunc: bool, exclusive: bool,
                    append: bool) -> Code<()>;
    fn _close_file_io(&self) -> Code<()>;
}
impl FileIoResource for FileIo {
    fn owner(&self) -> Code<FileSystem> {
        self.with_file_ref(|fr, _, _| fr.owner() )
    }

    fn _open_file_io(&self, istate: &InstanceState,
                     fr: FileRef, create: bool,
                     readable: bool, writable: bool,
                     trunc: bool, exclusive: bool,
                     append: bool) -> Code<()> {
        if try!(self.deref().data.lock()).is_some() {
            return Err(Error::Failed);
        }
        if append {
            return Err(Error::NotSupported);
        }

        if trunc && !writable {
            return Err(Error::NoAccess);
        }

        try!(fr.read_inner(|inner| {
            match inner.data {
                FileRefType::Dir { .. } => Err(Error::NotAFile),
                _ => Ok(()),
            }
        }));

        // These locks could be more fine grained. TODO

        let parent = try!(fr.parent());
        try!(parent.write_inner(|inner| {
            match inner.data {
                FileRefType::File { .. } |
                FileRefType::DoesNotExist => { return Err(Error::Failed); },
                _ => {},
            }

            let created = if create || trunc {
                try!(fr.write_inner(|inner| {
                    let created = if create {
                        if exclusive && match inner.data {
                            FileRefType::DoesNotExist => false,
                            _ => true,
                        } {
                            return Err(Error::FileExists);
                        }

                        match inner.data {
                            FileRefType::File { .. } => false,
                            FileRefType::Dir { .. } => unreachable!(),
                            FileRefType::DoesNotExist => {
                                inner.data = FileRefType::File {
                                    data: Default::default(),
                                };
                                inner.creation_time = istate.seconds_elapsed();
                                false
                            },
                        }
                    } else {
                        false
                    };

                    if trunc {
                        match inner.data {
                            FileRefType::File {
                                ref mut data,
                            } => {
                                data.clear();
                            },
                            _ => unreachable!(),
                        }

                        inner.last_modified_time = istate.seconds_elapsed();
                    }

                    Ok(created)
                }))
            } else {
                false
            };

            if created {
                match inner.data {
                    FileRefType::Dir { ref mut entries } => {
                        let last = try!(fr.path());
                        let last = last.iter().next_back().unwrap();
                        entries.insert(last.into());
                    },
                    _ => unreachable!(),
                }
            }

            Ok(())
        }));

        try!(fr.with_parents(|fr| {
            fr._use();
            Ok(())
        }));
        let mut lock = try!(self.deref().data.lock());
        *lock = Some((fr, readable, writable));
        Ok(())
    }

    fn _close_file_io(&self) -> Code<()> { self.close() }
}

fn rebuild_path(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut current: Vec<&OsStr> = Vec::new();

    if !path.has_root() {
        let s: &OsStr = "/".as_ref();
        current.push(s);
    }

    for component in path.components() {
        match component {
            Component::CurDir => continue,
            Component::Normal(p) => {
                current.push(p);
            },
            Component::ParentDir => {
                if current.len() > 1 {
                    current.pop();
                }
            },
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                current.clear();

                let s: &OsStr = "/".as_ref();
                current.push(s);
            },
        }
    }

    current.into_iter()
        .fold(PathBuf::new(), |prev, comp| {
            prev.join(comp)
                .to_path_buf()
        })
        .to_path_buf()
}

#[derive(Debug)]
pub struct FileSystem_ {
    id: PP_Resource,

    this: Option<(Weak<ResourceRc>, Weak<FileSystemState>)>,

    file_data: HashMap<PP_Resource, WeakResource<FileRefState>>,
    path_data: HashMap<PathBuf,     FileRef>,
    files:     HashMap<PP_Resource, WeakResource<FileIoState>>,

    closed: bool,
    created: bool,
}
impl FileSystem_ {
    pub fn resource(&self) -> Code<FileSystem> {
        if let Some((ref rc, ref state)) = self.this {
            Ok(Resource::from_weak(rc, state).unwrap())
        } else {
            Err(Error::Failed)
        }
    }

    fn get_file_data_ref(&self, res: PP_Resource) -> Code<FileRef> {
        if let Some(state) = self.file_data.get(&res) {
            if let Some(resource) = state.upgrade() {
                Ok(resource)
            } else {
                Err(Error::BadResource)
            }
        } else {
            Err(Error::BadArgument)
        }
    }
    fn get_file_io(&self, io: PP_Resource) -> Code<FileIo> {
        if let Some(io) = self.files
            .get(&io)
            .and_then(|io| io.upgrade() )
        {
            Ok(io)
        } else {
            Err(Error::BadResource)
        }
    }
}

#[derive(Debug)]
pub struct FileSystemState(RwLock<FileSystem_>, Instance, PP_Resource);
impl FileSystemState {
    pub fn new(instance: &Instance) -> FileSystem {
        let id = take_resource_id();
        let inner = FileSystem_ {
            id: id,
            closed: false,
            created: false,
            this: None,

            file_data: Default::default(),
            path_data: Default::default(),
            files: Default::default(),
        };
        let inner = RwLock::new(inner);

        let state = FileSystemState(inner, instance.clone(), id);
        let state = Arc::new(state);

        Resource::create(instance, state)
    }
    pub fn id(&self) -> PP_Resource { self.0.read().unwrap().id }
    pub fn instance(&self) -> &Instance { &self.1 }

    pub fn created(&self) -> bool { self.0.read().unwrap().created }
    pub fn create(&self) -> Code<()> {
        if !self.created() {
            let mut lock = try!(self.0.write());
            lock.created = true;
            Ok(())
        } else {
            Err(Error::Failed)
        }
    }

    pub fn open(&self) -> Code<()> {
        let mut l = self.0.write().unwrap();
        let this = unsafe { get_resource_arc(l.id).unwrap() };
        let this_state = match this.state() {
            &ResState::FileSystem(ref this_state) => this_state.clone(),
            _ => { return Err(Error::BadResource); },
        };
        l.this = Some((Arc::downgrade(&this), Arc::downgrade(&this_state)));

        if l.path_data.get(Path::new("/")).is_none() {
            let root = self.create_file_ref_(try!(l.resource()),
                                             None,
                                             &Path::new("/"));
            try!(root.write_inner(|inner| {
                inner.data = FileRefType::Dir {
                    entries: Default::default(),
                };

                Ok(())
            }));

            l.file_data.insert(root.id(), root.downgrade());
            l.path_data.insert(Path::new("/").to_path_buf(), root);
        }

        Ok(())
    }
    pub fn opened(&self) -> bool {
        let r = self.0.read().unwrap();
        r.this.is_some() && !r.closed
    }
    pub fn close(&self) {
        let mut w = self.0.write().unwrap();
        w.closed = true;
    }

    pub fn mkdir<T>(&self, path: T) -> Code<()>
        where T: Into<PathBuf>,
    {
        let path: PathBuf = path.into();
        let fr = try!(self.create_file_ref(path.as_ref()));

        self.mkdir_file_ref(fr, sys::PP_MAKEDIRECTORYFLAG_WITH_ANCESTORS)
    }

    pub fn resource_dtor(&self, res: Arc<ResourceRc>) {
        let state = res.state();
        match state {
            &ResState::FileIo(ref io) => {
                let _ = io.close();
            },
            &ResState::FileRef(ref io) => {
                unimplemented!()
            },
            &ResState::FileSystem(ref io) => {
                unreachable!()
            },
            _ => {},
        }
    }

    fn create_file_ref_(&self, owner: FileSystem,
                        parent: Option<FileRef>,
                        path: &Path) -> FileRef {
        let id = take_resource_id();
        let path = rebuild_path(path);
        let ref_ = FileRef_ {
            id: id,
            owner: owner,
            parent: parent,
            path: path,
            creation_time: 0.0,
            last_access_time: 0.0,
            last_modified_time: 0.0,
            data: FileRefType::DoesNotExist,

            name_var: None,
            path_var: None,

            current_uses: AtomicUsize::new(0),
        };
        let state = RwLock::new(ref_);
        let state = FileRefState(state, self.instance().clone(), id,
                                 Default::default());
        let state = Arc::new(state);
        Resource::create(self.instance(), state)
    }

    pub fn create_file_ref(&self, path: &Path) -> Code<FileRef> {
        let path = rebuild_path(path);

        let mut this = try!(self.0.write());
        if let Some(r) = this.path_data.get(&path).map(|p| p.clone() ) {
            Ok(r)
        } else {
            let mut prev_parts = Path::new("/").to_path_buf();
            let mut parent_fr = None;
            for parent in path.components()
                .map(|comp| {
                    prev_parts = prev_parts.join(comp.as_ref()).to_path_buf();

                    prev_parts.clone()
                })
            {
                let pp: &Path = parent.as_ref();
                let new_parent_fr = {
                    this.path_data
                        .get(pp)
                        .map(|p| p.clone() )
                };


                if parent_fr.is_none() && new_parent_fr.is_some() {
                    parent_fr = new_parent_fr;
                    continue;
                }

                let new_parent_fr: FileRef = new_parent_fr
                    .ok_or(Error::Failed)
                    .or_else(|_| -> Code<FileRef> {
                        let owner = try!(this.resource());
                        let fr = self.create_file_ref_(owner, parent_fr.clone(),
                                                       parent.as_ref());

                        this.file_data.insert(fr.id(), fr.downgrade());
                        this.path_data.insert(parent.clone(), fr.clone());

                        Ok(fr)
                    })
                    .unwrap();

                parent_fr = Some(new_parent_fr);
            }

            assert!(parent_fr.is_some());

            parent_fr.ok_or(Error::Failed)
        }
    }

    pub fn mkdir_file_ref(&self, file_state: FileRef, flags: u32) -> Code<()> {
        use std::path::Component;

        use super::sys::{PP_MAKEDIRECTORYFLAG_EXCLUSIVE, PP_MAKEDIRECTORYFLAG_WITH_ANCESTORS};

        if file_state.is_file() {
            return Err(Error::FileExists);
        }

        let fs_inner = try!(self.0.write());

        let mut inner_state = try!(file_state.0.write());

        {
            let mut comps = inner_state.path
                .components();

            let mut prev_path = Path::new("/").to_path_buf();

            while let Some(comp) = comps.next() {
                let path = match comp {
                    Component::Prefix(..) |
                    Component::ParentDir |
                    Component::CurDir => panic!("unexpected path component: `{:?}`",
                                                comp),
                    Component::RootDir => {
                        prev_path = Path::new("/").to_path_buf();
                        continue;
                    },
                    Component::Normal(p) => prev_path.join(p),
                };

                let path_data = fs_inner.path_data
                    .get(&path)
                    .map(|p| p.clone() )
                    .ok_or(Error::Failed);
                let path_data = try!(path_data);

                // Don't lock this file lock.
                if path_data.id() != file_state.id() {
                    if path_data.exists() && flags & PP_MAKEDIRECTORYFLAG_EXCLUSIVE != 0 {
                        return Err(Error::FileExists);
                    }
                    if !path_data.exists() && flags & PP_MAKEDIRECTORYFLAG_WITH_ANCESTORS != 0 {
                        try!(path_data.write_inner(|inner| {
                            match inner.data {
                                FileRefType::DoesNotExist => { },
                                FileRefType::Dir { ref mut entries } => {
                                    let rest = comps.as_path();
                                    let name = rest.iter().next().unwrap().into();
                                    if !entries.insert(name) &&
                                        flags & PP_MAKEDIRECTORYFLAG_EXCLUSIVE != 0
                                    {
                                        return Err(Error::FileExists);
                                    }

                                    return Ok(());
                                },
                                _ => { return Err(Error::FileExists); },
                            }

                            inner.data = FileRefType::Dir {
                                entries: Default::default(),
                            };

                            Ok(())
                        }));
                    }

                    prev_path = path;
                }
            }
        }

        inner_state.data = FileRefType::Dir { entries: Default::default(), };
        inner_state.creation_time = super::global_module().wall_time();

        Ok(())
    }

    pub fn touch_file_ref(&self, res: PP_Resource, last_access: f64,
                          last_mod: f64) -> Code<()> {
        let state = try!({
            let read = try!(self.0.read());
            read.get_file_data_ref(res)
        });
        let mut inner = try!(state.0.write());
        inner.last_access_time = last_access;
        inner.last_modified_time = last_mod;
        match inner.data {
            FileRefType::DoesNotExist => {
                inner.data = FileRefType::File {
                    data: Default::default(),
                };
                inner.creation_time = super::global_module().wall_time();
            },
            _ => {},
        }

        let mut fs_inner = try!(self.0.write());
        match fs_inner.path_data.entry(inner.path.clone()) {
            Entry::Occupied(_) => {},
            Entry::Vacant(v) => {
                v.insert(state.clone());
            },
        }
        Ok(())
    }

    pub fn delete_file_ref(&self, res: PP_Resource) -> Code<()> {
        let mut lock = try!(self.0.write());
        match lock.file_data.entry(res) {
            Entry::Occupied(o) => {
                if let Some(file_ref) = o.get().upgrade() {
                    if try!(file_ref.uses()) != 0 {
                        o.remove();
                        Ok(())
                    } else {
                        Err(Error::Failed)
                    }
                } else {
                    Err(Error::Failed)
                }
            },
            Entry::Vacant(_) => {
                Err(Error::FileNotFound)
            },
        }
    }

    pub fn rename_file_ref(&self, _file_ref: PP_Resource,
                           _new_file_ref: PP_Resource) -> Code<()> {
        /*let new_entry = self.file_data.get(&new_file_ref).cloned();
        if let Some(new_state) = new_entry {
            if new_state.in_use() {
                return Err(PP_ERROR_FAILED);
            }
        }

        let old_entry = self.file_data.get(&file_ref).cloned();

        match old_entry {
            None => { return Err(PP_ERROR_FILENOTFOUND); },
            Some(old_State) => {

            },
        }
        match*/

        Err(Error::NotSupported)
    }
    pub fn query_file_ref(&self, file_ref: PP_Resource) -> Code<PP_FileInfo> {
        let read = try!(self.0.read());
        let file_ref = try!(read.get_file_data_ref(file_ref));
        file_ref.query(sys::PP_FILESYSTEMTYPE_LOCALTEMPORARY)
    }

    pub fn read_dir_entries_file_ref(&self, file_ref: PP_Resource) -> Code<Vec<FileRef>> {
        let read = try!(self.0.read());
        let file_ref = try!(read.get_file_data_ref(file_ref));
        file_ref.read_inner(|inner| {
            match inner.data {
                FileRefType::Dir {
                    ref entries,
                } => {
                    let entries = entries.iter()
                        .map(|entry| {
                            let path_ref = read.path_data
                                .get(&inner.path.join(entry));
                            debug_assert!(path_ref.is_some());
                            path_ref.unwrap().clone()
                        })
                        .collect();
                    Ok(entries)
                },
                _ => { return Err(Error::BadArgument); },
            }
        })
    }

    pub fn create_file_io(&self) -> Code<FileIo> {
        let mut l = try!(self.0.write());
        let id = take_resource_id();
        let io = FileIoState {
            id: id,
            instance: self.1.clone(),
            data: Mutex::new(None),
        };
        let io = Arc::new(io);
        let io = Resource::create(&self.1, io);

        l.files.insert(id, io.downgrade());

        Ok(io)
    }

    pub fn get_file_io(&self, io: PP_Resource) -> Code<FileIo> {
        let lock = try!(self.0.read());
        lock.get_file_io(io)
    }

    pub fn open_file_io(&self, istate: &InstanceState,
                        io: PP_Resource, file_ref: PP_Resource, flags: u32) -> Code<()> {
        use super::sys::{PP_FILEOPENFLAG_READ, PP_FILEOPENFLAG_WRITE, PP_FILEOPENFLAG_CREATE,
                         PP_FILEOPENFLAG_TRUNCATE, PP_FILEOPENFLAG_EXCLUSIVE,
                         PP_FILEOPENFLAG_APPEND};

        let lock = try!(self.0.read());
        let io = try!(lock.get_file_io(io));
        let file_ref = try!(lock.get_file_data_ref(file_ref));

        let readable = flags & PP_FILEOPENFLAG_READ != 0;
        let writable = flags & PP_FILEOPENFLAG_WRITE != 0;
        let create   = flags & PP_FILEOPENFLAG_CREATE != 0;
        let trunc    = flags & PP_FILEOPENFLAG_TRUNCATE != 0;
        let exclusive = flags & PP_FILEOPENFLAG_EXCLUSIVE != 0;
        let append = flags & PP_FILEOPENFLAG_APPEND != 0;

        io._open_file_io(istate, file_ref, create, readable, writable,
                        trunc, exclusive, append)
    }

    pub fn query_file_io(&self, io: PP_Resource) -> Code<PP_FileInfo> {
        let io = try!(self.get_file_io(io));

        io.with_file_ref(|fr, readable, _writable| {
            if readable {
                fr.query(sys::PP_FILESYSTEMTYPE_LOCALTEMPORARY)
            } else {
                Err(Error::NoAccess)
            }
        })
    }

    pub fn touch_file_io(&self, _istate: &InstanceState,
                         io: PP_Resource, last_accessed_time: PP_Time,
                         last_modified_time: PP_Time) -> Code<()> {
        let io = try!(self.get_file_io(io));
        io.with_file_ref(|fr, _readable, writable| {
            if writable {
                let mut write = try!(fr.0.write());
                write.last_access_time = last_accessed_time;
                write.last_modified_time = last_modified_time;
                Ok(())
            } else {
                Err(Error::NoAccess)
            }
        })
    }
    pub fn read_file_io(&self, istate: &InstanceState,
                        io: PP_Resource, offset: usize,
                        buffer: &'static mut [u8]) -> Code<usize> {
        let io = try!(self.get_file_io(io));
        io.with_file_ref(|fr, readable, _writable| {
            if readable {
                let mut inner = try!(fr.0.write());
                let src_len = {
                    let src = match inner.data {
                        FileRefType::File {
                            ref data,
                        } => &data[offset..],
                        _ => { return Err(Error::NotAFile); },
                    };
                    buffer.copy_from_slice(src);
                    src.len()
                };
                inner.last_access_time = istate.seconds_elapsed();
                Ok(src_len)
            } else {
                Err(Error::NoAccess)
            }
        })
    }
    pub fn write_file_io(&self, istate: &InstanceState,
                         io: PP_Resource, offset: usize,
                         buffer: &'static [u8]) -> Code<usize> {
        let io = try!(self.get_file_io(io));
        io.with_file_ref(|fr, _readable, writable| {
            if writable {
                let mut lock = try!(fr.0.write());
                let dest_len = {
                    let dest = match lock.data {
                        FileRefType::File {
                            ref mut data,
                        } => &mut data[offset..],
                        _ => { return Err(Error::NotAFile); },
                    };
                    dest.copy_from_slice(buffer);
                    std::cmp::min(dest.len(), buffer.len())
                };
                lock.last_modified_time = istate.seconds_elapsed();
                Ok(dest_len)
            } else {
                Err(Error::NoAccess)
            }
        })
    }
    pub fn set_length_file_io(&self, istate: &InstanceState,
                              io: PP_Resource, new_length: usize) -> Code<usize> {
        let io = try!(self.get_file_io(io));
        io.with_file_ref(|fr, _readable, writable| {
            if writable {
                let mut lock = try!(fr.0.write());
                let len = match lock.data {
                    FileRefType::File {
                        ref mut data,
                    } => {
                        data.resize(new_length, 0);
                        data.len()
                    },
                    _ => { return Err(Error::NotAFile); },
                };
                lock.last_modified_time = istate.seconds_elapsed();
                Ok(len)
            } else {
                Err(Error::NoAccess)
            }
        })
    }
    pub fn flush_file_io(&self, _io: PP_Resource) -> Code<()> {
        // In memory only so this is a no-op.
        Ok(())
    }
    pub fn close_file_io(&self, io: FileIo) -> Code<()> {
        io._close_file_io()
    }
}

impl ResourceState for FileSystemState {
    fn into_resstate(this: Arc<Self>) -> ResState {
        ResState::FileSystem(this)
    }
    fn from_resstate(this: Arc<ResourceRc>) -> Code<Resource<FileSystemState>> {
        let state = try!(Self::state_from_resstate(&this))
            .clone();

        Ok(Resource::new(this, state))
    }
    fn state_from_resstate(rs: &Arc<ResourceRc>) -> Code<&Arc<Self>> {
        match rs.state() {
            &ResState::FileSystem(ref io) => Ok(io),
            _ => Err(Error::BadArgument),
        }
    }
    fn resource_id(this: &Arc<FileSystemState>) -> PP_Resource {
        this.2
    }
    fn resource_instance(this: &Arc<FileSystemState>) -> Instance {
        this.1.clone()
    }
}
unsafe impl Send for FileSystemState { }
pub type FileSystem = Resource<FileSystemState>;

impl ResourceState for FileIoState {
    fn into_resstate(this: Arc<Self>) -> ResState {
        ResState::FileIo(this)
    }
    fn from_resstate(this: Arc<ResourceRc>) -> Code<Resource<FileIoState>> {
        let state = try!(Self::state_from_resstate(&this))
            .clone();

        Ok(Resource::new(this, state))
    }
    fn state_from_resstate(rs: &Arc<ResourceRc>) -> Code<&Arc<Self>> {
        match rs.state() {
            &ResState::FileIo(ref io) => Ok(io),
            _ => Err(Error::BadArgument),
        }
    }
    fn resource_id(this: &Arc<FileIoState>) -> PP_Resource {
        this.id()
    }
    fn resource_instance(this: &Arc<Self>) -> Instance {
        this.instance.clone()
    }
}
impl ResourceState for FileRefState {
    fn into_resstate(this: Arc<Self>) -> ResState {
        ResState::FileRef(this)
    }
    fn from_resstate(this: Arc<ResourceRc>) -> Code<Resource<FileRefState>> {
        let state = try!(Self::state_from_resstate(&this))
            .clone();

        Ok(Resource::new(this, state))
    }
    fn state_from_resstate(rs: &Arc<ResourceRc>) -> Code<&Arc<Self>> {
        match rs.state() {
            &ResState::FileRef(ref r) => Ok(r),
            _ => Err(Error::BadArgument),
        }
    }
    fn resource_id(this: &Arc<Self>) -> PP_Resource {
        this.id()
    }
    fn resource_instance(this: &Arc<Self>) -> Instance {
        this.1.clone()
    }
}

static FILESYSTEM_INTERFACE: sys::PPB_FileSystem_1_0 = sys::PPB_FileSystem_1_0 {
    Create: Some(create_filesystem),
    IsFileSystem: Some(is_filesystem),
    Open: Some(open_filesystem),
    GetType: Some(get_fs_type),
};

pub static INTERFACES: Interfaces = &[
    ("PPB_FileSystem;1.0", interface_ptr(&FILESYSTEM_INTERFACE)),
];

extern "C" fn create_filesystem(instance: PP_Instance,
                                _type: PP_FileSystemType) -> PP_Resource {
    match _type {
        sys::PP_FILESYSTEMTYPE_LOCALTEMPORARY => {},
        _ => { return 0; },
    }

    let i = super::ModuleInterface::get_instance_interface(instance);
    let i = match i {
        Ok(i) => i,
        Err(_) => { return 0; },
    };

    match i.create_file_system() {
        Ok(fs) => { return fs.move_into_id(); },
        Err(_) => { return 0; },
    }
}
extern "C" fn is_filesystem(fs: PP_Resource) -> PP_Bool {
    match unsafe { get_resource_arc(fs) } {
        Some(rc) => {
            match rc.state() {
                &ResState::FileSystem(_) => PP_TRUE,
                _ => PP_FALSE,
            }
        },
        None => { return PP_FALSE; },
    }
}
extern "C" fn open_filesystem(fs: PP_Resource,
                              _size: libc::int64_t,
                              callback: PP_CompletionCallback) -> libc::int32_t {
    ppb_f!(R(fs), callback => open_file_system)
}
extern "C" fn get_fs_type(_fs: PP_Resource) -> sys::PP_FileSystemType {
    sys::PP_FILESYSTEMTYPE_LOCALTEMPORARY
}
