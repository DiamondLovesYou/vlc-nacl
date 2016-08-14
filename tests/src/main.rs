#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]

#![allow(unused_mut)]

/// Note: Don't use items from ppapi without specifically using
/// `ppapi::`. There's a implicit prelude related rustc bug that will eat all of
/// your RAM, caused by the ppapi crate.

use ppapi::filesystem_manager::FileSystem;

extern crate ppapi;
extern crate vlc;

struct Main {
    inst: ppapi::Instance,

    fs: Option<FileSystem>,
}
impl Main {
    pub fn fs(&mut self) -> FileSystem {
        if self.fs.is_none() {
            let fs = self.inst.create_file_system()
                .expect("couldn't create filesystem!");
            self.fs = Some(fs);
        }

        self.fs.as_ref().unwrap().clone()
    }


}

impl Drop for Main {
    fn drop(&mut self) {
        self.fs.take();

        ppapi::var::clear_var_instance();
        let _ = unsafe {
            self.inst.destroy()
        };
    }
}
impl Default for Main {
    fn default() -> Main {
        let args = vec!();

        let module = ppapi::global_module();
        let instance = module.create_instance(args);

        let instance = instance.expect("failed to create testing instance");
        instance.ping().unwrap();

        Main {
            inst: instance,

            fs: None,
        }
    }
}

describe! basics {
    before_each {
        let mut m = ::Main::default();

        let mut expected_live_vars = Some(0);
    }
    after_each {
        if let Some(expected_live_vars) = expected_live_vars {
            let live_vars = m.inst.get_live_vars().unwrap();
            assert_eq!(live_vars.len(), expected_live_vars);
        }
    }

    it "vlc_did_create" {
        println!("{}", m.inst.id());
    }

    describe! access {
        before_each {
            let fs = m.fs();
        }

        it "open-close" {
        }

    }
}

#[cfg(not(test))]
fn main() {
    panic!("Don't run this.");
}
