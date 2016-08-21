
pub fn get_utf8_var<T>(key: T) -> String
    where T: AsRef<str>,
{
    use std::env::{var};
    var(key.as_ref())
        .unwrap_or_else(|err| { panic!("ENV var `{}` contains non-utf8 text: `{:?}`", key.as_ref(), err) })
}

pub fn main() {
    use std::path::Path;
    let ppapi = get_utf8_var("LIBPPAPI");
    let ppapi = Path::new(&ppapi[..]);
    let search = ppapi.parent().unwrap();
    println!("cargo:rustc-link-search=native={}", search.display());
    //println!("cargo:rustc-link-lib=static={}", Path::new(ppapi.file_name().unwrap()).display());

    // force relinking if the ppapi modules archive changes
    println!("cargo:rerun-if-changed={}", ppapi.display());
}
