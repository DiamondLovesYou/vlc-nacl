
use super::interface::*;
use super::sys::*;

static GRAPHICS_3D: PPB_Graphics3D_1_0 = PPB_Graphics3D_1_0 {
    GetAttribMaxValue: Some(ret_default_stub::<_>),
    Create: Some(ret_default_stub::<_>),
    IsGraphics3D: Some(ret_default_stub::<_>),
    GetAttribs: Some(ret_not_supported_stub),
    SetAttribs: Some(ret_not_supported_stub),
    GetError: Some(ret_not_supported_stub),
    ResizeBuffers: Some(ret_not_supported_stub),
    SwapBuffers: Some(ret_not_supported_stub),
};

pub static INTERFACES: Interfaces = &[
    ("PPB_Graphics3D;1.0", interface_ptr(&GRAPHICS_3D)),
];

#[no_mangle] #[allow(non_snake_case)]
pub extern "C" fn glInitializePPAPI(_: GetInterface) -> PP_Bool {
    PP_TRUE
}
