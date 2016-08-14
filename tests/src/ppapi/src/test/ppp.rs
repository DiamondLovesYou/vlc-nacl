
#![allow(non_snake_case)]
#![allow(dead_code)]

use libc;
use std::collections::{HashSet, HashMap};
use std::sync::{Mutex, RwLock};

use std::slice::from_raw_parts;
use std::str::from_utf8_unchecked;

use ppapi::sys::{self, PP_Instance, PP_Resource};
use super::super::support::*;

pub static mut MODULE: Option<sys::PP_Module> = None;
pub static mut GET_INTERFACE: Option<sys::GetInterface> = None;

#[no_mangle]
pub extern "C" fn PPP_InitializeModule(module: sys::PP_Module, get_i: sys::GetInterface) -> libc::int32_t {
    unsafe {
        MODULE = Some(module);
        GET_INTERFACE = Some(get_i);
    }

    sys::PP_OK
}

/// A guarenteed unique type.
pub struct QueriedInterfaces(RwLock<HashSet<String>>);
impl QueriedInterfaces {
    pub fn add(&self, name: &str) {
        let mut lock = self.0.write().unwrap();
        lock.insert(name.to_string());
    }
    pub fn interface_queried(&self, name: &str) -> bool {
        let lock = self.0.read().unwrap();
        lock.contains(name)
    }
}
pub fn queried_interfaces() -> &'static QueriedInterfaces {
    fn init() -> QueriedInterfaces {
        QueriedInterfaces(RwLock::default())
    }
    global_singleton(init)
}

#[no_mangle]
pub extern "C" fn PPP_GetInterface(name: *const libc::c_char) -> *const libc::c_void {
    let name = unsafe {
        let len = libc::strlen(name);
        let slice = from_raw_parts(name as *const u8, len);
        from_utf8_unchecked(slice)
    };

    queried_interfaces().add(name);

    match name {
        "PPP_Instance;1.1" => {
            (&PPP_INSTANCE as *const sys::PPP_Instance_1_1) as *const libc::c_void
        },
        _ => 0 as _,
    }
}

static PPP_INSTANCE: sys::PPP_Instance_1_1 = sys::PPP_Instance_1_1 {
    create: create_instance,
    destroy: destroy_instance,
    change_view: change_view,
    change_focus: change_focus,
    handle_document_load: handle_document_load,
};
#[derive(Default)]
pub struct PPPInstanceCalls(Mutex<HashMap<PP_Instance, Vec<PPPInstanceCall>>>);
impl PPPInstanceCalls {
    pub fn add_call(&self, instance: PP_Instance, call: PPPInstanceCall) {
        use std::collections::hash_map::Entry;
        let mut lock = self.0.lock().unwrap();
        match lock.entry(instance) {
            Entry::Occupied(mut o) => {
                let om = o.get_mut();
                om.push(call);
            },
            Entry::Vacant(v) => {
                v.insert(vec!(call));
            }
        }
    }

    pub fn take_instance_calls(&self,
                               instance: PP_Instance) -> Vec<PPPInstanceCall>
    {
        let id = instance;
        self.0.lock().unwrap()
            .get(&id)
            .map(|calls| calls.clone() )
            .unwrap_or_default()
    }
}
pub fn ppp_instance_calls() -> &'static PPPInstanceCalls {
    global_singleton_default()
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PPPInstanceCall {
    CreateInstance {
        args: Vec<(String, String)>,
    },
    DestroyInstance,
}

extern "C" fn create_instance(instance: PP_Instance,
                              argc: libc::uint32_t,
                              argk: *mut *const ::libc::c_char,
                              argv: *mut *const ::libc::c_char)
                              -> sys::PP_Bool
{
    let argk = argk as *const *const u8;
    let argv = argv as *const *const u8;

    let argk = unsafe {
        from_raw_parts(argk, argc as usize)
    };
    let argv = unsafe {
        from_raw_parts(argv, argc as usize)
    };

    fn to_rstr(&s: &*const u8) -> String {
        unsafe {
            let len = libc::strlen(s as *const _);
            let slice = from_raw_parts(s, len);
            from_utf8_unchecked(slice)
                .to_string()
        }
    }

    let args = argk
        .iter()
        .map(to_rstr)
        .zip(argv.iter().map(to_rstr))
        .collect();

    let call = PPPInstanceCall::CreateInstance {
        args: args,
    };

    ppp_instance_calls().add_call(instance, call);

    sys::PP_TRUE
}
extern "C" fn destroy_instance(instance: PP_Instance) {
    let call = PPPInstanceCall::DestroyInstance;
    ppp_instance_calls().add_call(instance, call);
}
extern "C" fn change_view(_instance: PP_Instance, _view: PP_Resource) {
    unimplemented!()
}
extern "C" fn change_focus(_instance: PP_Instance, _has_focus: sys::PP_Bool) {
    unimplemented!()
}
extern "C" fn handle_document_load(_instance: PP_Instance,
                                   _url_loader: PP_Resource) -> sys::PP_Bool {
    unimplemented!()
}
