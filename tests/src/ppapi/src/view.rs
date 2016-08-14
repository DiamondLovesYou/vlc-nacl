
use super::interface::*;
use super::sys::PPB_View_1_2;

static VIEW_INTERFACE: PPB_View_1_2 = PPB_View_1_2 {
    IsView: Some(ret_false_stub),
    GetRect: Some(ret_false_stub),
    IsFullscreen: Some(ret_false_stub),
    IsVisible: Some(ret_false_stub),
    IsPageVisible: Some(ret_false_stub),
    GetClipRect: Some(ret_false_stub),
    GetDeviceScale: Some(ret_default_stub),
    GetCSSScale: Some(ret_default_stub),
    GetScrollOffset: Some(ret_default_stub),
};
pub static INTERFACES: Interfaces = &[
    ("PPB_View;1.2", interface_ptr(&VIEW_INTERFACE)),
];
