
use std::collections::HashSet;
use std::env::{var};
use std::fs::OpenOptions;
use std::io::{Read};
use std::path::{Path, PathBuf};

extern crate bindgen;
extern crate gcc;
extern crate syntex_syntax as syntax;

pub fn get_utf8_var<T>(key: T) -> String
    where T: AsRef<str>,
{
    var(key.as_ref())
        .unwrap_or_else(|err| { panic!("ENV var `{}` contains non-utf8 text: `{:?}`", key.as_ref(), err) })
}

fn bindgen_c() -> String {
    format!("{}/src/bindgen.c", get_utf8_var("CARGO_MANIFEST_DIR"))
}

fn build_sys_rs(libs: Vec<String>) {
    use bindgen::*;
    use syntax::ast::{self};
    use syntax::codemap::{DUMMY_SP};
    use syntax::print::{pp, pprust};

    let bindgen_c = bindgen_c();

    let builder = Builder::new(bindgen_c);
    let builder = get_utf8_var("CFLAGS")
        .split_whitespace()
        .fold(builder, |mut b, s| {
            b.clang_arg(s);
            b
        });
    let bindings = libs.into_iter()
        .fold(builder, |mut b, l| {
            b.link(l, LinkType::Dynamic);
            b
        })
        .match_pat("vlc_objects.h")
        .match_pat("vlc_common.h")
        .match_pat("vlc_access.h")
        .match_pat("vlc_variables.h")
        .match_pat("stdarg.h") // for va_args
        .convert_macros(true)
        .builtins()
        .generate()
        .unwrap();

    let module = ast::Mod {
        inner: DUMMY_SP,
        items: bindings.into_ast(),
    };


    let out_dir = get_utf8_var("OUT_DIR");
    let out = Path::new(&out_dir[..]).join("sys.rs");
    let out = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(out)
        .unwrap();

    let mut ps = pprust::rust_printer(Box::new(out));
    ps.print_mod(&module, &[]).unwrap();
    ps.print_remaining_comments().unwrap();
    pp::eof(&mut ps.s).unwrap();
    ps.s.out.flush().unwrap();
}

fn build_support_c() {
    use gcc::*;

    let bindgen_c = bindgen_c();

    Config::new()
        .file(bindgen_c)
        .include(get_utf8_var("VLC_INCLUDE_DIR"))
        .cpp(true)
        .compile("libvlc-sys.a");
}

struct Lib {
    name: String,
    dep_libs: Vec<String>,
    dep_dirs: Vec<String>,
}
impl Lib {
    fn parse(path: &String, new_dep_las: &mut HashSet<String>) -> Lib {

        println!("parsing `{}`", path);

        #[derive(Debug, Default)]
        struct La {
            old_library: Option<String>,
            dependency_libs: Option<String>,
        }

        let s = {
            let mut f = OpenOptions::new()
                .read(true)
                .open(&path[..]).unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            s
        };

        let mut la: La = Default::default();
        for line in s.lines() {
            let line = if let Some(from_end) = line.rfind('#') {
                &line[..from_end]
            } else {
                line
            };

            let line = if let Some(from_start) = line.find(|c: char| {
                !c.is_whitespace()
            }) {
                &line[from_start..]
            } else {
                line
            };

            if line.len() == 0 { continue; }

            //println!("\tparsing line `{}`", line);

            let (key, rest) = if let Some(from_start) = line.find('=') {
                (&line[..from_start], &line[from_start + 1..])
            } else {
                println!("line `{}` has no key! ignoring.", line);
                continue;
            };
            //println!("\tkey `{}`", key);

            match key {
                "old_library" |
                "dependency_libs" => {},
                _ => continue,
            }

            let value = if let Some(start) = rest.find('\'') {
                let end = rest[start + 1..].find('\'')
                    .expect("missing '") + 1;
                rest[start + 1..end].trim()
            } else {
                // Not a string, ignore.
                continue;
            };

            match key {
                "old_library" => {
                    if la.old_library.is_some() {
                        panic!("unexpected duplicate key: `old_library`!");
                    }

                    la.old_library = Some(value.to_string());
                },
                "dependency_libs" => {
                    if la.dependency_libs.is_some() {
                        panic!("unexpected duplicate key: `dependency_libs`!");
                    }

                    la.dependency_libs = Some(value.to_string());
                },

                _ => unreachable!(),
            }
        }

        //println!("finished parsing `{}`: {:?}", path, la);

        if la.dependency_libs.is_none() ||
            la.old_library.is_none()
        {
            panic!("missing key/value on `{}`", path);
        }

        let dl = la.dependency_libs.take().unwrap();
        let ol = la.old_library.take().unwrap();

        let mut lib = Lib {
            name: ol["lib".len()..ol.len() - ".a".len()].to_string(),
            dep_libs: vec!(),
            dep_dirs: vec!(),
        };

        for arg in dl.split_whitespace() {
            if arg.starts_with("-L") {
                lib.dep_dirs.push(arg["-L".len()..].to_string());
                continue;
            }
            if arg.starts_with("-l") {
                lib.dep_libs.push(arg["-l".len()..].to_string());
                continue;
            }

            assert_eq!("la", Path::new(arg).extension().unwrap());

            new_dep_las.insert(arg.to_string());
        }

        lib
    }
}

pub fn main() {
    /*let vlc_module_count: usize = get_utf8_var("VLC_MODULE_COUNT")
        .parse()
        .unwrap();

    let search_dir_count: usize = get_utf8_var("SEARCH_DIR_COUNT")
        .parse()
        .unwrap();

    let mut lib_dirs = HashSet::new();
    for idx in 0..search_dir_count {
        let key = format!("SEARCH_DIR_{}", idx);
        let value = get_utf8_var(key);
        lib_dirs.insert(value);
    }

    let mut libs = HashSet::new();
    for idx in 0..vlc_module_count {
        let libs_key = format!("VLC_MODULE_{}_LIBS", idx);
        let libs_value = get_utf8_var(&libs_key[..]);

        let libs_len = libs.len();
        libs.extend(libs_value.split_whitespace().map(|s| s.to_string() ));
        if libs.len() == libs_len {
            //panic!("key `{}` is empty!", libs_key);
        }
    }

    for lib_dir in lib_dirs.into_iter() {
        println!("cargo:rustc-link-search=native={}", lib_dir);
    }
    for lib in libs.iter() {
        println!("cargo:rustc-link-lib=static={}", lib);
    }*/

    let compile = get_utf8_var("COMPILE_LOC");
    println!("cargo:rerun-if-changed={}", compile);

    let blacklist: HashSet<_> = get_utf8_var("VLC_MODULE_BLACKLIST")
        .split_whitespace()
        .map(|s| s.to_string() )
        .collect();

    let mut new_deps = HashSet::new();

    let vlc_build_dir = get_utf8_var("VLC_BUILD_DIR");

    fn check_dir(dir: PathBuf, new_deps: &mut HashSet<String>, blacklist: &HashSet<String>) {
        let entries = dir.read_dir().unwrap();
        for entry in entries {
            let entry = entry.unwrap();

            let file_name = entry.file_name().into_string().expect("not utf8");
            //println!("on file_name => `{}`", file_name);

            let file_type = entry.file_type().unwrap();
            if file_type.is_dir() {
                check_dir(entry.path(), new_deps, blacklist);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }

            if !file_name.starts_with("lib") { continue; }
            if !file_name.ends_with("_plugin.la") { continue; }
            let core_name = &file_name["lib".len()..file_name.len() -
                                       "_plugin.la".len()];
            if blacklist.contains(core_name) {
                continue;
            }

            let path = entry.path().into_os_string().into_string()
                .expect("not utf8");

            new_deps.insert(path);
        }
    }

    let vlc_modules = Path::new(&vlc_build_dir[..])
        .join("modules")
        .to_path_buf();
    check_dir(vlc_modules, &mut new_deps, &blacklist);


    let mut all = HashSet::new();
    let mut libs = Vec::new();

    fn parse_modules(all: &mut HashSet<String>,
                     libs: &mut Vec<Lib>,
                     deps: Vec<String>)
    {
        if deps.len() == 0 { return; }

        let mut new_deps = HashSet::new();

        for dep in deps.into_iter() {
            let lib = Lib::parse(&dep, &mut new_deps);
            println!("cargo:rerun-if-changed={}", dep);
            assert!(all.insert(dep));
            libs.push(lib);
        }

        if new_deps.len() == 0 {
            return;
        }

        let new_deps = new_deps.difference(all).cloned().collect();
        parse_modules(all, libs, new_deps);
    }
    parse_modules(&mut all, &mut libs, new_deps.into_iter().collect());

    let libvlc = format!("{}/lib/libvlc.la", &vlc_build_dir[..]);
    parse_modules(&mut all, &mut libs, vec!(libvlc));

    let mut blacklist_libs = HashSet::new();
    blacklist_libs.insert("GL".to_string());
    let blacklist_libs = blacklist_libs;

    let mut bindgen_libs = vec!("c".to_string());
    for Lib {
        name, dep_libs, dep_dirs,
    } in libs.into_iter() {
        for dir in dep_dirs.into_iter() {
            println!("cargo:rustc-link-search=native={}", dir);
        }

        bindgen_libs.push(name.clone());
        for lib in dep_libs.into_iter() {
            if !blacklist_libs.contains(&lib[..]) {
                bindgen_libs.push(lib);
            }
        }
    }
    bindgen_libs.push("compat".to_string());

    let vlc_build_dir = Path::new(&vlc_build_dir[..]).to_path_buf();
    println!("cargo:rustc-link-search=native={}",
             vlc_build_dir.join("modules/.libs").display());
    println!("cargo:rustc-link-search=native={}",
             vlc_build_dir.join("src/.libs").display());
    println!("cargo:rustc-link-search=native={}",
             vlc_build_dir.join("lib/.libs").display());
    println!("cargo:rustc-link-search=native={}",
             vlc_build_dir.join("compat/.libs").display());
    println!("cargo:rustc-link-search=native=/usr/lib/i386-linux-gnu");


    build_sys_rs(bindgen_libs);
    build_support_c();
}
