
/// This module holds the code for testing the PPAPI testing backend.

use super::ppapi::*;

use self::ppp::{PPPInstanceCall, ppp_instance_calls};

pub mod ppp;
mod api;

pub struct TestInstance(ModuleInterface, Instance);
impl Drop for TestInstance {
    fn drop(&mut self) {
        var::clear_var_instance();
        let _ = unsafe {
            self.1.destroy()
        };
    }
}
impl ::std::ops::Deref for TestInstance {
    type Target = Instance;
    fn deref(&self) -> &Self::Target { &self.1 }
}

pub fn new_test_instance(args: Vec<(String, String)>) -> TestInstance {
    let module = global_module();
    let instance = module.create_instance(args);

    let instance = instance.expect("failed to create testing instance");
    TestInstance(module, instance)
}

#[test]
fn ppp_interface_called() {
    {
        let _instance = new_test_instance(Default::default());
    }
    assert!(ppp::queried_interfaces().interface_queried("PPP_Instance;1.1"));
}

#[test]
fn ppp_did_create_called() {
    let instance = new_test_instance(Default::default());
    instance.ping().unwrap();
    let calls = ppp_instance_calls().take_instance_calls(instance.id());
    assert_eq!(&calls[0], &PPPInstanceCall::CreateInstance { args: Default::default(), });
}

#[test]
fn ppp_did_destroy_called() {
    let id = {
        let instance = new_test_instance(Default::default());
        instance.id()
    };
    let calls = ppp_instance_calls().take_instance_calls(id);
    assert_eq!(calls.last(), Some(&PPPInstanceCall::DestroyInstance));
}
