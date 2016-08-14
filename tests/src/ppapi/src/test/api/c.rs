
/// This module, for the most part, just checks that the interfaces we need are
/// present. It doesn't try to test every possible edge case.

use libc;

fn get_interface(name: &str) -> *const libc::c_void {
    // Ensure the global module is initialized first.
    let _ = ::ppapi::global_module();


    let c_name = format!("{}\0", name);
    let gi = unsafe { super::super::ppp::GET_INTERFACE.unwrap() };
    let c_str: &[u8] = c_name.as_ref();
    let ptr = gi(c_str.as_ptr() as *const _);
    ptr
}

fn check_get_interface(name: &str) -> bool {
    let ptr = get_interface(name);

    unsafe { ptr.as_ref().is_some() }
}

#[test]
fn invalid_interface_not_present() {
    assert!(!check_get_interface("NOT AN INTERFACE, BRAH!"));
}

#[test]
fn audio_interface_present() {
    assert!(check_get_interface("PPB_Audio;1.1"));
}
#[test]
fn audio_config_interface_present() {
    assert!(check_get_interface("PPB_AudioConfig;1.1"));
}
#[test]
fn console_interface_present() {
    assert!(check_get_interface("PPB_Console;1.0"));
}
#[test]
fn core_interface_present() {
    assert!(check_get_interface("PPB_Core;1.0"));
}
#[test]
fn file_io_interface_present() {
    assert!(check_get_interface("PPB_FileIO;1.1"));
}
#[test]
fn file_ref_interface_present() {
    assert!(check_get_interface("PPB_FileRef;1.2"));
}
#[test]
fn file_system_interface_present() {
    assert!(check_get_interface("PPB_FileSystem;1.0"));
}

#[ignore] // Not actually queried by VLC.
#[test]
fn graphics2d_interface_present() {
    assert!(check_get_interface("PPB_Graphics2D;1.1"));
}
#[test]
fn graphics3d_interface_present() {
    assert!(check_get_interface("PPB_Graphics3D;1.0"));
}

#[ignore]
#[test]
fn image_data_interface_present() {
    assert!(check_get_interface("PPB_ImageData;1.0"));
}
#[test]
fn instance_interface_present() {
    assert!(check_get_interface("PPB_Instance;1.0"));
}
#[test]
fn message_loop_interface_present() {
    assert!(check_get_interface("PPB_MessageLoop;1.0"));
}
#[test]
fn messaging_interface_present() {
    assert!(check_get_interface("PPB_Messaging;1.2"));
}
#[test]
fn mouse_cursor_interface_present() {
    assert!(check_get_interface("PPB_MouseCursor;1.0"));
}
#[test]
fn url_loader_interface_present() {
    assert!(check_get_interface("PPB_URLLoader;1.0"));
}
#[test]
fn url_request_info_interface_present() {
    assert!(check_get_interface("PPB_URLRequestInfo;1.0"));
}
#[test]
fn url_response_info_interface_present() {
    assert!(check_get_interface("PPB_URLResponseInfo;1.0"));
}
#[test]
fn var_interface_present() {
    assert!(check_get_interface("PPB_Var;1.2"));
}
#[test]
fn var_array_interface_present() {
    assert!(check_get_interface("PPB_VarArray;1.0"));
}
#[test]
fn var_dict_interface_present() {
    assert!(check_get_interface("PPB_VarDictionary;1.0"));
}
#[test]
fn view_interface_present() {
    assert!(check_get_interface("PPB_View;1.2"));
}
