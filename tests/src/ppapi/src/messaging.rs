
use libc;

use super::sys::*;
use super::interface::*;
use super::resource::{get_resource};
use super::var::Var;
use super::result::{ResultCode, Error};

use super::ModuleInterface;

static INTERFACE: PPB_Messaging_1_2 = PPB_Messaging_1_2 {
    PostMessage: Some(ppb_messaging_post_message),
    RegisterMessageHandler: Some(ppb_messaging_register_message_handler),
    UnregisterMessageHandler: Some(ppb_messaging_unregister_message_handler),
};

pub static INTERFACES: Interfaces = &[
    ("PPB_Messaging;1.2", interface_ptr(&INTERFACE)),
];

extern "C" fn ppb_messaging_post_message(instance: PP_Instance,
                                         message: PP_Var) {
    let i = ModuleInterface::get_instance_interface(instance);
    if i.is_err() { return; }
    let i = i.unwrap();

    let msg = Var::from(message);
    if msg.is_err() { return; }
    let msg = msg.unwrap();
    msg.up_ref();

    i.post_message(msg);
}
extern "C" fn ppb_messaging_register_message_handler(instance: PP_Instance,
                                                     user_data: *mut libc::c_void,
                                                     handler: *const PPP_MessageHandler_0_2,
                                                     message_loop: PP_Resource) -> libc::int32_t {
    let i = ModuleInterface::get_instance_interface(instance);
    if i.is_err() { return Error::BadInstance.into(); }
    let i = i.unwrap();

    let handler = unsafe { handler.as_ref() };
    if handler.is_none() { return Error::BadArgument.into(); }
    let handler = handler.unwrap();

    let ml = get_resource(message_loop);
    if let Err(e) = ml { return e.into(); }
    let ml = ml.unwrap();

    i.register_message_handler(user_data, handler, ml)
        .into_code()
}
extern "C" fn ppb_messaging_unregister_message_handler(instance: PP_Instance) {
    let i = ModuleInterface::get_instance_interface(instance);
    if i.is_err() {
        error!("missing instance: `{}`", instance);
        return;
    }
    let i = i.unwrap();

    i.unregister_message_handler();
}
