
use libc;

use std::cell::{RefCell};
use std::collections::HashMap;
use std::hash::{Hasher, Hash};
use std::ops::{Deref};
use std::sync::atomic::{AtomicPtr, AtomicUsize, AtomicI64, Ordering};
use std::sync::{Arc, RwLock};

use super::instance::Instance;
use super::sys::*;
use super::resource::RefCounted;
use super::result::{Code, Error};
use super::interface::*;

pub fn take_var_id() -> PP_VarId {
    static NEXT_ID: AtomicI64 = AtomicI64::new(1);

    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug)]
pub struct VarRc_ {
    id: PP_VarId,
    refs: AtomicUsize,
    var:  VarRef,
}
impl VarRc_ {
    pub fn id(&self) -> PP_VarId { self.id }
    pub fn ref_count(&self) -> usize { self.refs.load(Ordering::Relaxed) }
}
impl Deref for VarRc_ {
    type Target = VarRef;
    fn deref(&self) -> &VarRef { &self.var }
}

#[derive(Debug)]
pub struct VarRc(Arc<VarRc_>);
impl VarRc {
    fn new(id: PP_VarId, var: VarRef) -> VarRc {
        INSTANCE.with(|instance| {
            if let Some(instance) = instance.borrow().as_ref() {
                instance.track_var(id);
            }

            let v = VarRc_ {
                id: id,
                refs: AtomicUsize::new(1),
                var: var,
            };

            VarRc(Arc::new(v))
        })
    }
}
impl Deref for VarRc {
    type Target = VarRc_;
    fn deref(&self) -> &VarRc_ { &*self.0 }
}
impl AsRef<Arc<VarRc_>> for VarRc {
    fn as_ref(&self) -> &Arc<VarRc_> { &self.0 }
}
impl Clone for VarRc {
    fn clone(&self) -> VarRc {
        self.up_ref();

        VarRc(self.0.clone())
    }
}
impl Drop for VarRc {
    fn drop(&mut self) {
        down_ref_var(&self.0);
    }
}
impl PartialEq for VarRc {
    fn eq(&self, rhs: &VarRc) -> bool {
        self.id() == rhs.id()
    }
}
impl Eq for VarRc { }

impl RefCounted for VarRc_ {
    fn refs(&self) -> &AtomicUsize { &self.refs }
}

#[derive(Debug)]
pub enum VarRef {
    Dictionary(RwLock<HashMap<StringVar, Var>>),
    Array(RwLock<Vec<Var>>),
    String(String),
    Resource(PP_Resource),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Var {
    Undefined,
    Null,
    Bool(bool),
    Int(i32),
    Double(f64),
    String(StringVar),
    Object(VarRc),
    Array(ArrayVar),
    Dictionary(DictVar),
    Resource(ResVar),
}
impl Var {
    #[doc(hidden)]
    pub fn up_ref(&self) {
        use self::Var::*;
        let inner = match self {
            &String(ref s) => s.0.as_ref(),
            &Object(ref v) => v.as_ref(),
            &Array(ref v)  => v.0.as_ref(),
            &Dictionary(ref d) => d.0.as_ref(),
            &Resource(ref v) => v.0.as_ref(),
            _ => { return; },
        };

        inner.up_ref();
    }
    #[doc(hidden)]
    #[allow(dead_code)]
    fn down_ref(&self) {
        use self::Var::*;
        let inner = match self {
            &String(ref s) => s.0.as_ref(),
            &Object(ref v) => v.as_ref(),
            &Array(ref v)  => v.0.as_ref(),
            &Dictionary(ref d) => d.0.as_ref(),
            &Resource(ref v) => v.0.as_ref(),
            _ => { return; },
        };

        inner.down_ref();
    }

    pub fn from(mut v: PP_Var) -> Code<Var> {
        let var_rc = get_var_id(v)
            .ok_or(Error::Failed)
            .and_then(get_var);
        match v._type {
            PP_VARTYPE_ARRAY => Ok(Var::Array(try!(ArrayVar::from(var_rc.unwrap())))),
            PP_VARTYPE_DICTIONARY => Ok(Var::Dictionary(try!(DictVar::from(var_rc.unwrap())))),
            PP_VARTYPE_RESOURCE => Ok(Var::Resource(try!(ResVar::from(var_rc.unwrap())))),
            PP_VARTYPE_OBJECT => Ok(Var::Object(var_rc.unwrap())),
            PP_VARTYPE_STRING => Ok(Var::String(try!(StringVar::from(var_rc.unwrap())))),

            PP_VARTYPE_BOOL => {
                Ok(Var::Bool(unsafe { *v.value.as_bool() } != PP_FALSE))
            },
            PP_VARTYPE_DOUBLE => {
                Ok(Var::Double(unsafe { *v.value.as_double() }))
            },
            PP_VARTYPE_INT32 => {
                Ok(Var::Int(unsafe { *v.value.as_int() }))
            },
            PP_VARTYPE_NULL => Ok(Var::Null),
            PP_VARTYPE_UNDEFINED => Ok(Var::Undefined),

            _ => Err(Error::BadArgument),
        }
    }
}

impl Into<PP_Var> for Var {
    fn into(self) -> PP_Var {
        // this is a move, so up the ref count for the other side (b/c the drop
        // on self will down ref it).

        let v = match self {
            Var::Undefined => PP_Var {
                _type: PP_VARTYPE_UNDEFINED,
                padding: 0,
                value: Default::default(),
            },
            Var::Null => PP_Var {
                _type: PP_VARTYPE_NULL,
                padding: 0,
                value: Default::default(),
            },
            Var::Bool(v) => {
                let mut value: Union_PP_VarValue = Default::default();
                unsafe { *value.as_bool() = if v { PP_TRUE } else { PP_FALSE } };
                PP_Var {
                    _type: PP_VARTYPE_BOOL,
                    padding: 0,
                    value: value,
                }
            },
            Var::Int(v) => {
                let mut value: Union_PP_VarValue = Default::default();
                unsafe { *value.as_int() = v };
                PP_Var {
                    _type: PP_VARTYPE_INT32,
                    padding: 0,
                    value: value,
                }
            },
            Var::Double(v) => {
                let mut value: Union_PP_VarValue = Default::default();
                unsafe { *value.as_double() = v };
                PP_Var {
                    _type: PP_VARTYPE_DOUBLE,
                    padding: 0,
                    value: value,
                }
            },
            Var::String(ref v) => {
                v.0.up_ref();
                let mut value: Union_PP_VarValue = Default::default();
                unsafe { *value.as_id() = v.id() };
                PP_Var {
                    _type: PP_VARTYPE_STRING,
                    padding: 0,
                    value: value,
                }
            },
            Var::Object(_) => unimplemented!(),
            Var::Array(ref v) => {
                v.0.up_ref();
                let mut value: Union_PP_VarValue = Default::default();
                unsafe { *value.as_id() = v.id() };
                PP_Var {
                    _type: PP_VARTYPE_ARRAY,
                    padding: 0,
                    value: value,
                }
            },
            Var::Dictionary(ref v) => {
                v.0.up_ref();
                let mut value: Union_PP_VarValue = Default::default();
                unsafe { *value.as_id() = v.id() };
                PP_Var {
                    _type: PP_VARTYPE_DICTIONARY,
                    padding: 0,
                    value: value,
                }
            },
            Var::Resource(_) => unimplemented!(),
        };

        v
    }
}
impl Default for Var {
    fn default() -> Var { Var::Null }
}

#[derive(Debug, Clone)]
pub struct StringVar(VarRc);
impl StringVar {
    pub fn new(v: String) -> StringVar {
        let v = VarRef::String(v);
        let v = register_var(v);
        StringVar(v)
    }
    pub fn id(&self) -> PP_VarId { self.0.id() }
    pub fn from(v: VarRc) -> Code<StringVar> {
        match v.0.var {
            VarRef::String(_) => Ok(StringVar(v)),
            _ => Err(Error::BadArgument),
        }
    }

    pub fn inner(&self) -> &VarRc { &self.0 }
}
impl From<String> for StringVar {
    fn from(s: String) -> StringVar {
        StringVar::new(s)
    }
}
impl AsRef<str> for StringVar {
    fn as_ref(&self) -> &str {
        match self.0.var {
            VarRef::String(ref s) => &s[..],
            _ => unreachable!(),
        }
    }
}
impl Deref for StringVar {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_ref()
    }
}
impl PartialEq for StringVar {
    fn eq(&self, rhs: &StringVar) -> bool {
        self.0.id == rhs.0.id || {
            let l: &str = self.as_ref();
            let r: &str = rhs.as_ref();
            l == r
        }
    }
}
impl Eq for StringVar { }
impl Hash for StringVar {
    fn hash<H>(&self, state: &mut H)
        where H: Hasher,
    {
        let s: &str = self.as_ref();
        s.hash(state)
    }
}
impl Into<Var> for StringVar {
    fn into(self) -> Var { Var::String(self) }
}
impl Into<PP_Var> for StringVar {
    fn into(self) -> PP_Var {
        let v: Var = self.into();
        v.into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ArrayVar(VarRc);
impl ArrayVar {
    pub fn new() -> ArrayVar {
        let v = VarRef::Array(RwLock::default());
        let v = register_var(v);
        ArrayVar(v)
    }
    pub fn id(&self) -> PP_VarId { self.0.id() }

    pub fn from(v: VarRc) -> Code<ArrayVar> {
        match v.0.var {
            VarRef::Array(_) => Ok(ArrayVar(v)),
            _ => Err(Error::BadArgument),
        }
    }

    fn read_inner<F, U>(&self, f: F) -> U
        where F: FnOnce(&[Var]) -> U,
    {
        match self.0.var {
            VarRef::Array(ref lock) => {
                let read = lock.read().unwrap();
                f(read.as_ref())
            },
            _ => unreachable!(),
        }
    }
    fn write_inner<F, U>(&self, f: F) -> U
        where F: FnOnce(&mut Vec<Var>) -> U,
    {
        match self.0.var {
            VarRef::Array(ref lock) => {
                let mut write = lock.write().unwrap();
                f(write.as_mut())
            },
            _ => unreachable!(),
        }
    }

    pub fn get(&self, idx: usize) -> Var {
        self.read_inner(|inner| {
            if idx >= inner.len() { return Default::default(); }
            inner[idx].clone()
        })
    }
    pub fn set(&self, idx: usize, v: Var) -> bool {
        self.write_inner(|inner| {
            if inner.len() <= idx {
                false
            } else {
                inner[idx] = v;
                true
            }
        })
    }
    pub fn set_len(&self, new_len: usize) {
        self.write_inner(|inner| {
            inner.resize(new_len, Default::default())
        })
    }
    pub fn len(&self) -> usize {
        self.read_inner(|inner| {
            inner.len()
        })
    }
}
impl Into<Var> for ArrayVar {
    fn into(self) -> Var {
        Var::Array(self)
    }
}
impl Into<PP_Var> for ArrayVar {
    fn into(self) -> PP_Var {
        let v: Var = self.into();

        v.into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DictVar(VarRc);
impl DictVar {
    pub fn new() -> DictVar {
        let v = VarRef::Dictionary(RwLock::default());
        let v = register_var(v);
        DictVar(v)
    }
    pub fn id(&self) -> PP_VarId { self.0.id() }

    pub fn from(v: VarRc) -> Code<DictVar> {
        match v.0.var {
            VarRef::Dictionary(_) => Ok(DictVar(v)),
            _ => Err(Error::BadArgument),
        }
    }

    fn read_inner<F, U>(&self, f: F) -> U
        where F: FnOnce(&HashMap<StringVar, Var>) -> U,
    {
        match self.0.var {
            VarRef::Dictionary(ref lock) => {
                let read = lock.read().unwrap();
                f(&*read)
            },
            _ => unreachable!(),
        }
    }
    fn write_inner<F, U>(&self, f: F) -> U
        where F: FnOnce(&mut HashMap<StringVar, Var>) -> U,
    {
        match self.0.var {
            VarRef::Dictionary(ref lock) => {
                let mut write = lock.write().unwrap();
                f(&mut *write)
            },
            _ => unreachable!(),
        }
    }

    pub fn get(&self, k: &StringVar) -> Var {
        self.read_inner(|inner| {
            inner.get(k)
                .cloned()
                .unwrap_or(Default::default())
        })
    }
    pub fn set(&self, k: StringVar, v: Var) {
        self.write_inner(move |inner| {
            inner.insert(k, v);
        })
    }
    pub fn delete(&self, k: &StringVar) {
        self.write_inner(|inner| {
            inner.remove(k);
        })
    }

    pub fn has_key(&self, k: &StringVar) -> bool {
        self.read_inner(|inner| {
            inner.contains_key(k)
        })
    }

    pub fn keys(&self) -> ArrayVar {
        self.read_inner(|inner| {
            let a = ArrayVar::new();
            a.set_len(inner.len());

            let mut idx = 0;
            for key in inner.keys() {
                a.set(idx, key.clone().into());
                idx += 1;
            }

            a
        })
    }

    pub fn len(&self) -> usize {
        self.read_inner(|inner| inner.len() )
    }
}

impl Into<Var> for DictVar {
    fn into(self) -> Var {
        Var::Dictionary(self)
    }
}
impl Into<PP_Var> for DictVar {
    fn into(self) -> PP_Var {
        let v: Var = self.into();

        v.into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResVar(VarRc);
impl ResVar {
    pub fn from(v: VarRc) -> Code<ResVar> {
        match v.0.var {
            VarRef::Resource(_) => Ok(ResVar(v)),
            _ => Err(Error::BadArgument),
        }
    }
}
impl Deref for ResVar {
    type Target = PP_Resource;
    fn deref(&self) -> &PP_Resource {
        match self.0.var {
            VarRef::Resource(ref res) => res,
            _ => unreachable!(),
        }
    }
}

/// Used to info test instances about created variables for tracking.
thread_local!(static INSTANCE: RefCell<Option<Instance>> = Default::default());
pub fn set_var_instance(instance: Instance) {
    assert_eq!(INSTANCE.with(|i| i.borrow().clone() ), None);

    INSTANCE.with(|i| *i.borrow_mut() = Some(instance) );
}
pub fn clear_var_instance() {
    INSTANCE.with(|i| {
        let _ = i.borrow_mut().take();
    });
}

type Vars = HashMap<PP_VarId, Arc<VarRc_>>;
type VarsLock = RwLock<Vars>;
static VARS: AtomicPtr<VarsLock> = AtomicPtr::new(0 as *mut _);
fn get_vars() -> &'static VarsLock {
    let ptr = VARS.load(Ordering::Relaxed);
    if let Some(map) = unsafe { ptr.as_ref() } {
        map
    } else {
        let new: Box<VarsLock> = Box::new(Default::default());
        let raw = Box::into_raw(new);

        if VARS.compare_and_swap(ptr, raw, Ordering::SeqCst) != ptr {
            let _ = unsafe { Box::from_raw(raw) };
        }

        get_vars()
    }
}

pub fn register_var(value: VarRef) -> VarRc {
    use std::collections::hash_map::Entry;
    let map = get_vars();
    let (duplicate_id, ret) = {
        let mut mwg = map.write().unwrap();
        let id = take_var_id();
        match mwg.entry(id) {
            Entry::Vacant(entry) => {
                let inner = VarRc::new(id, value);
                entry.insert(inner.as_ref().clone());
                (false, Some(inner))
            },
            Entry::Occupied(_) => (true, None),
        }
    };

    // Don't panic with the lock.
    assert!(!duplicate_id);

    ret.unwrap()
}

pub struct VarsAccess<'a>(&'a Vars);
impl<'a> VarsAccess<'a> {
    pub fn get(&self, id: PP_VarId) -> Option<VarRc> {
        self.0.get(&id)
            .map(|v| {
                v.up_ref();

                VarRc(v.clone())
            })
    }
}
pub fn read_vars<F, U>(f: F) -> Code<U>
    where F: FnOnce(VarsAccess) -> Code<U>,
{
    let map = get_vars();
    let lock = try!(map.read());
    f(VarsAccess(&*lock))
}

/// DO NOT CHANGE THE TYPE
pub fn with_var_ref<F, U>(id: PP_VarId, f: F) -> Code<U>
    where F: FnOnce(&VarRef) -> Code<U>,
{
    let map = get_vars();
    let mg = map.read().unwrap();
    if let Some(var_rc) = mg.get(&id) {
        f(&var_rc.var)
    } else {
        Err(Error::BadArgument)
    }
}
pub fn get_var(id: PP_VarId) -> Code<VarRc> {
    let map = get_vars();
    let mg = map.read().unwrap();
    if let Some(v) = mg.get(&id) {
        v.up_ref();
        Ok(VarRc(v.clone()))
    } else {
        Err(Error::BadArgument)
    }
}

pub fn up_ref_var_id(id: PP_VarId) {
    let map = get_vars();
    let mg = map.read().unwrap();
    if let Some(v) = mg.get(&id) {
        v.up_ref();
    }
}
pub fn down_ref_var_id(id: PP_VarId) {
    let map = get_vars();
    let remove = {
        let mg = map.read().unwrap();
        if let Some(v) = mg.get(&id) {
            v.down_ref()
        } else {
            return;
        }
    };

    if remove {
        let mut mg = map.write().unwrap();
        mg.remove(&id);
    }
}
pub fn down_ref_var(var: &Arc<VarRc_>) {
    let map = get_vars();
    if var.down_ref() {
        let mut mg = map.write().unwrap();
        mg.remove(&var.id());
    }
}

static VAR_INTERFACE: PPB_Var_1_2 = PPB_Var_1_2 {
    AddRef: Some(ppb_var_up_ref),
    Release: Some(ppb_var_down_ref),
    VarFromUtf8: Some(ppb_var_from_utf8),
    VarToUtf8: Some(ppb_var_to_utf8),
    VarFromResource: Some(ppb_var_from_resource),
    VarToResource: Some(ppb_var_to_resource),
    GlobalVarFromUtf8: Some(ppb_var_global_var_from_utf8),
};
static VAR_ARRAY_INTERFACE: PPB_VarArray_1_0 = PPB_VarArray_1_0 {
    Create: Some(ppb_var_array_create),
    Get: Some(ppb_var_array_get),
    Set: Some(ppb_var_array_set),
    GetLength: Some(ppb_var_array_get_length),
    SetLength: Some(ppb_var_array_set_length),
};
static VAR_DICT_INTERFACE: PPB_VarDictionary_1_0 = PPB_VarDictionary_1_0 {
    Create: Some(ppb_var_dict_create),
    Get: Some(ppb_var_dict_get),
    Set: Some(ppb_var_dict_set),
    Delete: Some(ppb_var_dict_delete),
    HasKey: Some(ppb_var_dict_contains),
    GetKeys: Some(ppb_var_dict_keys),
};

pub static INTERFACES: Interfaces = &[
    ("PPB_Var;1.2", interface_ptr(&VAR_INTERFACE)),
    ("PPB_VarArray;1.0", interface_ptr(&VAR_ARRAY_INTERFACE)),
    ("PPB_VarDictionary;1.0", interface_ptr(&VAR_DICT_INTERFACE)),
];

fn get_var_id(mut var: PP_Var) -> Option<PP_VarId> {
    match var._type {
        PP_VARTYPE_ARRAY |
        PP_VARTYPE_ARRAY_BUFFER |
        PP_VARTYPE_DICTIONARY |
        PP_VARTYPE_OBJECT |
        PP_VARTYPE_RESOURCE |
        PP_VARTYPE_STRING => {
            Some(unsafe { *var.value.as_id() })
        },

        _ => None,
    }
}
macro_rules! get_var_type {
    ($ty:ident => Id($expr:expr) => R($ret:expr)) => ({

        let id = get_var_id($expr);
        if id.is_none() { return $ret; }
        let id = id.unwrap();

        let var = get_var(id);
        if var.is_err() { return $ret; }
        let var = var.unwrap();

        let s = $ty::from(var);
        if s.is_err() { return $ret; }
        let s = s.unwrap();

        s
    });
    (Any($expr:expr) => R($ret:expr)) => ({
        let s = Var::from($expr);
        if s.is_err() { return $ret; }
        let s = s.unwrap();

        s
    })
}

extern "C" fn ppb_var_up_ref(var: PP_Var) {
    if let Some(id) = get_var_id(var) {
        up_ref_var_id(id);
    }
}
extern "C" fn ppb_var_down_ref(var: PP_Var) {
    if let Some(id) = get_var_id(var) {
        down_ref_var_id(id);
    }
}

extern "C" fn ppb_var_from_utf8(cstr: *const libc::c_char,
                                len: libc::uint32_t) -> PP_Var {
    use std::slice::from_raw_parts;
    use std::str::from_utf8;

    let cstr = cstr as *const u8;
    let cbuf = unsafe {
        from_raw_parts(cstr, len as usize)
    };
    match from_utf8(cbuf) {
        Ok(s) => {
            let s = StringVar::new(s.to_string());
            return s.into();
        },
        Err(_) => {
            return Default::default();
        },
    }
}

extern "C" fn ppb_var_to_utf8(var: PP_Var,
                              len: *mut libc::uint32_t) -> *const libc::c_char
{
    let len = unsafe { len.as_mut() };
    if len.is_none() { return 0 as _; }
    let len = len.unwrap();

    let s = get_var_type!(StringVar => Id(var) => R(0 as _));

    let str: &str = s.as_ref();

    let ptr = str.as_ptr() as *const libc::c_char;
    *len = str.len() as libc::uint32_t;

    return ptr;
}

extern "C" fn ppb_var_to_resource(_var: PP_Var) -> PP_Resource {
    ret_default_stub()
}
extern "C" fn ppb_var_from_resource(_r: PP_Resource) -> PP_Var {
    ret_default_stub()
}

extern "C" fn ppb_var_global_var_from_utf8(cstr: *const libc::c_char,
                                           len: libc::uint32_t) -> PP_Var {
    let instance = INSTANCE.with(|i| i.borrow_mut().take() );
    let r = ppb_var_from_utf8(cstr, len);
    INSTANCE.with(move |i| { *i.borrow_mut() = instance; });

    r
}

// ------- PPB_VarArray_1_0 -------

extern "C" fn ppb_var_array_create() -> PP_Var {
    let a = ArrayVar::new();

    a.into()
}
extern "C" fn ppb_var_array_get(array: PP_Var, idx: libc::uint32_t) -> PP_Var {
    let a = get_var_type!(ArrayVar => Id(array) => R(Default::default()));

    let v = a.get(idx as usize);
    v.into()
}
extern "C" fn ppb_var_array_set(array: PP_Var, idx: libc::uint32_t,
                                v: PP_Var) -> PP_Bool {
    let v = get_var_type!(Any(v) => R(PP_FALSE));
    let array = get_var_type!(ArrayVar => Id(array) => R(PP_FALSE));

    if array.set(idx as usize, v) {
        PP_TRUE
    } else {
        PP_FALSE
    }
}
extern "C" fn ppb_var_array_set_length(array: PP_Var,
                                       new_len: libc::uint32_t) -> PP_Bool {
    let a = get_var_type!(ArrayVar => Id(array) => R(PP_FALSE));
    a.set_len(new_len as usize);

    PP_TRUE
}
extern "C" fn ppb_var_array_get_length(array: PP_Var) -> libc::uint32_t {
    let a = get_var_type!(ArrayVar => Id(array) => R(0));
    a.len() as _
}


// ------- PPB_VarDictionary_1_0 -------

extern "C" fn ppb_var_dict_create() -> PP_Var {
    let d = DictVar::new();

    d.into()
}
extern "C" fn ppb_var_dict_get(dict: PP_Var, k: PP_Var) -> PP_Var {
    let d = get_var_type!(DictVar => Id(dict) => R(Default::default()));
    let k = get_var_type!(StringVar => Id(k) => R(Default::default()));

    d.get(&k).into()
}
extern "C" fn ppb_var_dict_set(dict: PP_Var, k: PP_Var, v: PP_Var) -> PP_Bool {
    let d = get_var_type!(DictVar => Id(dict) => R(PP_FALSE));
    let k = get_var_type!(StringVar => Id(k) => R(PP_FALSE));
    let v = get_var_type!(Any(v) => R(PP_FALSE));

    d.set(k, v);

    PP_TRUE
}
extern "C" fn ppb_var_dict_delete(dict: PP_Var, k: PP_Var) {
    let d = get_var_type!(DictVar => Id(dict) => R(Default::default()));
    let k = get_var_type!(StringVar => Id(k) => R(Default::default()));

    d.delete(&k);
}
extern "C" fn ppb_var_dict_contains(dict: PP_Var, k: PP_Var) -> PP_Bool {
    let d = get_var_type!(DictVar => Id(dict) => R(PP_FALSE));
    let k = get_var_type!(StringVar => Id(k) => R(PP_FALSE));

    if d.has_key(&k) {
        PP_TRUE
    } else {
        PP_FALSE
    }
}
extern "C" fn ppb_var_dict_keys(dict: PP_Var) -> PP_Var {
    let d = get_var_type!(DictVar => Id(dict) => R(Default::default()));
    let keys = d.keys();

    keys.into()
}
