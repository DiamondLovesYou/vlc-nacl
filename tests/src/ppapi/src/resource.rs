use libc;

use std;
use std::collections::{HashMap};
use std::hash::{Hasher, Hash};
use std::ops::{Deref};
use std::sync::{Arc, RwLock, Weak};
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};

use super::instance::Instance;
use super::sys::*;
use super::result::{Code, Error};
use super::callback::MessageLoopState;
use super::url_loader::{UrlLoaderState, UrlRequestInfoState, UrlResponseInfoState};
use super::filesystem_manager::{FileRefState, FileIoState,
                                FileSystemState};
use super::SyncInterfacePtr;

pub fn take_resource_id() -> PP_Resource {
    use std::sync::atomic::AtomicI32;
    static NEXT_ID: AtomicI32 = AtomicI32::new(1);
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}
#[derive(Debug)]
pub struct ResourceRc {
    refs: AtomicUsize,
    res:  ResState,
}
impl ResourceRc {
    pub fn id(&self) -> PP_Resource { self.res.id() }
    pub fn state(&self) -> &ResState { &self.res }

    pub fn instance(&self) -> Instance { self.res.instance() }
}
pub trait RefCounted {
    fn refs(&self) -> &AtomicUsize;
    fn up_ref(&self) {
        self.refs().fetch_add(1, Ordering::SeqCst);
    }
    fn down_ref(&self) -> bool {
        self.refs().fetch_sub(1, Ordering::SeqCst) == 1
    }

    fn ref_count(&self) -> usize { self.refs().load(Ordering::SeqCst) }
}
impl RefCounted for ResourceRc {
    fn refs(&self) -> &AtomicUsize { &self.refs }
}

#[derive(Debug)]
pub enum ResState {
    MessageLoop(Arc<MessageLoopState>),
    UrlLoader(Arc<UrlLoaderState>),
    UrlRequestInfo(Arc<UrlRequestInfoState>),
    UrlResponseInfo(Arc<UrlResponseInfoState>),
    FileIo(Arc<FileIoState>),
    FileRef(Arc<FileRefState>),
    FileSystem(Arc<FileSystemState>),
}
impl ResState {
    pub fn id(&self) -> PP_Resource {
        use self::ResState::*;
        match self {
            &MessageLoop(ref v) => <MessageLoopState as ResourceState>::resource_id(v),
            &UrlLoader(ref v) => <UrlLoaderState as ResourceState>::resource_id(v),
            &FileIo(ref v) => <FileIoState as ResourceState>::resource_id(v),
            &FileRef(ref v) => <FileRefState as ResourceState>::resource_id(v),
            &FileSystem(ref v) => <FileSystemState as ResourceState>::resource_id(v),
            &UrlRequestInfo(ref v) => <UrlRequestInfoState as ResourceState>::resource_id(v),
            &UrlResponseInfo(ref v) => <UrlResponseInfoState as ResourceState>::resource_id(v),
        }
    }

    pub fn needs_close(&self) -> bool {
        use self::ResState::*;
        match self {
            &FileIo(..) |
            &FileRef(..) |
            &FileSystem(..) |
            &UrlLoader(..) => true,
            _ => false,
        }
    }

    pub fn instance(&self) -> Instance {
        use self::ResState::*;
        match self {
            &MessageLoop(ref v) => <MessageLoopState as ResourceState>::resource_instance(v),
            &UrlLoader(ref v) => <UrlLoaderState as ResourceState>::resource_instance(v),
            &FileIo(ref v) => <FileIoState as ResourceState>::resource_instance(v),
            &FileRef(ref v) => <FileRefState as ResourceState>::resource_instance(v),
            &FileSystem(ref v) => <FileSystemState as ResourceState>::resource_instance(v),
            &UrlRequestInfo(ref v) => <UrlRequestInfoState as ResourceState>::resource_instance(v),
            &UrlResponseInfo(ref v) => <UrlResponseInfoState as ResourceState>::resource_instance(v),
        }
    }
}

pub trait ResourceState: Sized {
    fn into_resstate(this: Arc<Self>) -> ResState;
    fn from_resstate(rc: Arc<ResourceRc>) -> Code<Resource<Self>> {
        let state = try!(Self::state_from_resstate(&rc))
            .clone();

        Ok(Resource::new(rc, state))
    }
    fn state_from_resstate(rs: &Arc<ResourceRc>) -> Code<&Arc<Self>>;

    fn resource_id(this: &Arc<Self>) -> PP_Resource;
    /// Note: Instance can't be shared between threads because Sender<> can't.
    fn resource_instance(this: &Arc<Self>) -> Instance;
}

#[derive(Debug)]
pub struct Resource<T>(Arc<ResourceRc>, Arc<T>)
    where T: ResourceState;
impl<T> Resource<T>
    where T: ResourceState,
{
    pub fn create(instance: &Instance, state: Arc<T>) -> Resource<T> {
        let rc = ResourceRc {
            refs: AtomicUsize::new(1),
            res: <T as ResourceState>::into_resstate(state.clone()),
        };
        let arc = Arc::new(rc);

        instance.resource_ctor(arc.clone());
        register_resource(arc.clone());

        Resource(arc, state)
    }
    pub fn id(&self) -> PP_Resource { <T as ResourceState>::resource_id(&self.1) }
    pub fn instance(&self) -> Instance { <T as ResourceState>::resource_instance(&self.1) }

    #[doc(hidden)]
    pub fn new(arc: Arc<ResourceRc>, state: Arc<T>) -> Resource<T> {
        Resource(arc, state)
    }
    #[doc(hidden)]
    pub fn get_rc(&self) -> &Arc<ResourceRc> { &self.0 }

    pub fn from_weak(rc: &Weak<ResourceRc>, state: &Weak<T>) -> Option<Resource<T>> {
        rc.upgrade()
            .and_then(move |rc| {
                state.upgrade()
                    .map(move |state| {
                        rc.up_ref();
                        Resource(rc, state)
                    })
            })
    }

    pub fn downgrade(&self) -> WeakResource<T> {
        WeakResource(Arc::downgrade(&self.0), Arc::downgrade(&self.1), self.id())
    }

    pub fn move_into_id(self) -> PP_Resource {
        let id = self.id();
        self.0.up_ref();

        id
    }

    pub fn ref_count(&self) -> usize { self.0.ref_count() }
}
impl<T> Drop for Resource<T>
    where T: ResourceState,
{
    fn drop(&mut self) {
        if self.0.down_ref() {
            {
                let map = get_resources();
                let mut lock = map.write().unwrap();
                lock.remove(&self.id());
            }

            self.instance()
                .resource_dtor(self.0.clone());
        }
    }
}
impl<T> Clone for Resource<T>
    where T: ResourceState,
{
    fn clone(&self) -> Resource<T> {
        self.0.up_ref();

        Resource(self.0.clone(), self.1.clone())
    }
}
impl<T> Deref for Resource<T>
    where T: ResourceState,
{
    type Target = T;
    fn deref(&self) -> &T {
        self.1.deref()
    }
}
impl<T> Hash for Resource<T>
    where T: ResourceState,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}
impl<T, U> PartialEq<Resource<U>> for Resource<T>
    where T: ResourceState,
          U: ResourceState,
{
    fn eq(&self, rhs: &Resource<U>) -> bool {
        <T as ResourceState>::resource_id(&self.1) ==
            <U as ResourceState>::resource_id(&rhs.1)
    }
}

#[derive(Clone)]
pub struct WeakResource<T>(Weak<ResourceRc>, Weak<T>, PP_Resource);
impl<T> WeakResource<T>
    where T: ResourceState,
{
    pub fn id(&self) -> PP_Resource { self.2 }
    pub fn upgrade(&self) -> Option<Resource<T>> {
        Resource::from_weak(&self.0, &self.1)
    }
}
impl<T> std::fmt::Debug for WeakResource<T>
    where T: ResourceState + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, r#"WeakResource {{
\tid: {:?},
\tresource: {:?},
}}"#, self.2, self.upgrade())
    }
}

type Resources = HashMap<PP_Resource, Arc<ResourceRc>>;
type ResourcesLock = RwLock<Resources>;
static RESOURCES: AtomicPtr<ResourcesLock> = AtomicPtr::new(0 as *mut _);

fn get_resources() -> &'static ResourcesLock {
    let ptr = RESOURCES.load(Ordering::Relaxed);
    if let Some(map) = unsafe { ptr.as_ref() } {
        map
    } else {
        let new: Box<ResourcesLock> = Box::new(Default::default());
        let raw = Box::into_raw(new);

        if RESOURCES.compare_and_swap(ptr, raw, Ordering::SeqCst) != ptr {
            let _ = unsafe { Box::from_raw(raw) };
        }

        get_resources()
    }
}
/// TODO: support passing ownership to another instance.
pub fn register_resource(rc: Arc<ResourceRc>) {
    use std::collections::hash_map::Entry;
    info!("registering resource id `{}`", rc.id());
    let map = get_resources();
    let duplicate_id = {
        let mut mwg = map.write().unwrap();
        match mwg.entry(rc.id()) {
            Entry::Vacant(entry) => {
                entry.insert(rc);
                false
            },
            Entry::Occupied(_) => true,
        }
    };

    // Don't panic with the lock.
    assert!(!duplicate_id);
}
pub fn unregister_resource(res: PP_Resource) {
    if let Some(resource) = {
        let map = get_resources();
        let mut mwg = map.write().unwrap();
        mwg.remove(&res)
    } {
        resource.refs.store(0, Ordering::SeqCst);
        let instance = resource.instance();
        instance.resource_dtor(resource);
    }
}

/// YOU MUST UP REF THE RESOURCE
pub unsafe fn get_resource_arc(id: PP_Resource) -> Option<Arc<ResourceRc>> {
    let map = get_resources();
    let mg = map.read().unwrap();

    mg.get(&id).cloned()
}
pub fn get_resource_instance(res: PP_Resource) -> Option<Instance> {
    let map = get_resources();
    let mg = map.read().unwrap();
    mg.get(&res)
        .map(|i| i.instance() )
}
pub fn get_resource<T>(id: PP_Resource) -> Code<Resource<T>>
    where T: ResourceState,
{
    let arc = unsafe { get_resource_arc(id) }
        .ok_or(Error::BadResource);
    let arc = try!(arc);

    arc.up_ref();

    <T as ResourceState>::from_resstate(arc)
}

pub extern "C" fn up_ref_resource(res: PP_Resource) {
    let map = get_resources();
    let mg = map.read().unwrap();
    if let Some(rc) = mg.get(&res) {
        rc.up_ref();
    }
}
pub extern "C" fn down_ref_resource(res: PP_Resource) {
    let map = get_resources();
    let result = {
        let read = map.read().unwrap();
        if let Some(rc) = read.get(&res) {
            rc.down_ref()
        } else {
            return;
        }
    };
    if result {
        info!("deleting resource `{}`", res);
        let resource = {
            let mut lock = map.write().unwrap();
            lock.remove(&res)
        };
        if let Some(resource) = resource {
            let instance = resource.instance();
            instance.resource_dtor(resource);
        }
    }
}

static CORE_INTERFACE: PPB_Core_1_0 = PPB_Core_1_0 {
    up_ref_resource: up_ref_resource,
    down_ref_resource: down_ref_resource,
    get_time: get_wall_time,
    get_time_ticks: get_module_ticks,
    call_on_main_thread: call_on_main_thread,
    on_main_thread: on_main_thread,
};

extern "C" fn get_wall_time() -> PP_Time {
    let module = super::global_module();
    module.wall_time()
}
extern "C" fn get_module_ticks() -> PP_TimeTicks {
    // XXX the clone of the global module interface struct involves atomic ops,
    // but we don't need those parts of it.
    let module = super::global_module();
    module.seconds_elapsed()
}

extern "C" fn call_on_main_thread(_delay_in_milliseconds: libc::int32_t,
                                  _callback: PP_CompletionCallback,
                                  _result: libc::int32_t) {
    unimplemented!()
}
extern "C" fn on_main_thread() -> PP_Bool {
    unimplemented!()
}

pub static INTERFACES: &'static [(&'static str, SyncInterfacePtr)] = &[
    ("PPB_Core;1.0", SyncInterfacePtr::new(&CORE_INTERFACE)),
];
