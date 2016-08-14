
use super::interface::*;
use super::sys::*;

static CONSOLE_INTERFACE: PPB_Console_1_0 = PPB_Console_1_0 {
    Log: Some(log),
    LogWithSource: Some(log_with_source),
};

pub static INTERFACES: Interfaces = &[
    ("PPB_Console;1.0", interface_ptr(&CONSOLE_INTERFACE)),
];

extern "C" fn log(_instance: PP_Instance,
                  _level: PP_LogLevel,
                  _value: PP_Var) {
    // TODO
}
extern "C" fn log_with_source(_instance: PP_Instance,
                              _level: PP_LogLevel,
                              _source: PP_Var,
                              _value: PP_Var) {
    // TODO
}
