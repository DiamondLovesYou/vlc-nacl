
/// This modules tests the Rust interfaces, ie those used in the `vlc-nacl`
/// tests. It's more thorough than the C side tests, but doesn't test everything.

use std::path::{Path};

use ppapi::prelude::*;
use ppapi::FileSystem;
use ppapi::filesystem_manager::FileRefResource;
use ppapi::sys;

use super::super::*;

#[test]
fn track_var_live() {
    let i = new_test_instance(Default::default());

    let str: StringVar = From::from("test".to_string());

    let live_vars = i.get_live_vars().unwrap();
    assert_eq!(live_vars.len(), 1);
    assert_eq!(live_vars[0].id(), str.id());
}
#[test]
fn track_vars_live() {
    let i = new_test_instance(Default::default());

    let _one: StringVar = From::from("test".to_string());
    let _two: StringVar = From::from("test2".to_string());

    let live_vars = i.get_live_vars().unwrap();
    assert_eq!(live_vars.len(), 2);
}

#[test]
fn track_var_ref_count() {
    let _i = new_test_instance(Default::default());

    let str: StringVar = From::from("test".to_string());
    assert_eq!(str.inner().ref_count(), 1);
}

#[test]
fn track_var_dead() {
    let i = new_test_instance(Default::default());

    {
        let str: StringVar = From::from("test".to_string());

        let live_vars = i.get_live_vars().unwrap();
        assert_eq!(live_vars.len(), 1);
        assert_eq!(live_vars[0].id(), str.id());
    }

    let live_vars = i.get_live_vars().unwrap();
    assert_eq!(live_vars.len(), 0);
}

#[test]
fn string_var() {
    let str = "test";
    let sv = StringVar::new(str.to_string());
    assert_eq!(&*sv, str);
}
#[test]
fn array_var() {
    let a = ArrayVar::new();
    a.set_len(1);
    assert_eq!(a.len(), 1);

    assert_eq!(a.get(0), Default::default());

    let v = Var::Int(123);
    assert!(a.set(0, v.clone()));
    assert_eq!(a.get(0), v);

    assert!(!a.set(1, Default::default()));
}

#[test]
fn dict_var() {
    let d = DictVar::new();

    let k1: StringVar = From::from("key one".to_string());
    assert!(!d.has_key(&k1));

    let v: Var = Default::default();
    d.set(k1.clone(), v.clone());
    assert!(d.has_key(&k1));

    d.delete(&k1);
    assert!(!d.has_key(&k1));

    d.set(k1.clone(), k1.clone().into());
    assert_eq!(d.get(&k1), k1.into());
}

fn _create_filesystem_resource() -> (TestInstance, FileSystem) {
    let i = new_test_instance(Default::default());

    let fs = i.create_file_system();
    assert!(!fs.is_err());
    let fs = fs.unwrap();

    assert!(fs.created());
    assert!(!fs.opened());

    assert!(i.open_file_system(fs.id(), Default::default()).is_ok());

    assert!(fs.created());
    assert!(fs.opened());

    (i, fs)
}

#[test]
fn create_filesystem_resource() {
    _create_filesystem_resource();
}

#[test]
fn create_file_ref_resource() {
    use ppapi::sys::*;

    let (i, fs) = _create_filesystem_resource();

    let path = Path::new("/test/test").to_path_buf();

    let fr = i.create_file_ref(fs.id(), path);
    assert!(fr.is_ok());
    let fr = fr.unwrap();

    assert!(fr.file_type().unwrap() == PP_FILETYPE_OTHER);
    assert_eq!(fr.query(0), Err(Error::FileNotFound));
}

#[test]
fn file_ref_parent() {
    let (i, fs) = _create_filesystem_resource();

    let path = Path::new("/test/test").to_path_buf();

    let fr = i.create_file_ref(fs.id(), path);
    assert!(fr.is_ok());
    let fr = fr.unwrap();

    let parent_fr = fr.parent();
    assert!(parent_fr.is_ok());
    let parent_fr = parent_fr.unwrap();

    let parent_path = parent_fr.path();
    assert!(parent_path.is_ok());
    let parent_path = parent_path.unwrap();

    let check_parent = Path::new("/test").to_path_buf();

    assert_eq!(parent_path, check_parent);
}

#[test]
fn file_ref_id_is_same() {
    let (i, fs) = _create_filesystem_resource();

    let path = Path::new("/test-dir/test-file").to_path_buf();

    let fr = i.create_file_ref(fs.id(), path.clone());
    assert!(fr.is_ok());
    let fr = fr.unwrap();

    let fr2 = i.create_file_ref(fs.id(), path);
    assert!(fr2.is_ok());
    let fr2 = fr2.unwrap();

    assert_eq!(fr2, fr);
}

#[test]
fn file_ref_mkdir() {
    let (i, fs) = _create_filesystem_resource();

    fs.mkdir("/test-dir").unwrap();

    let d = i.create_file_ref(fs.id(), Path::new("/test-dir").to_path_buf()).unwrap();
    let q = d.query(0).unwrap();
    assert_eq!(q._type, sys::PP_FILETYPE_DIRECTORY);
}

#[test]
fn file_ref_parent_id_is_same() {
    let (i, fs) = _create_filesystem_resource();

    let path = Path::new("/test-dir/test-file").to_path_buf();

    let fr = i.create_file_ref(fs.id(), path.clone());
    assert!(fr.is_ok());
    let fr = fr.unwrap();

    let path = Path::new("/test-dir").to_path_buf();

    let parent = i.create_file_ref(fs.id(), path.clone());
    let parent = parent.unwrap();
    assert!(fr.parent() == Ok(parent));
}

#[test]
fn file_io_create_missing_dir() {
    let (i, fs) = _create_filesystem_resource();

    let path = Path::new("/test-dir/test-file").to_path_buf();
    let fr = i.create_file_ref(fs.id(), path);
    assert!(fr.is_ok());
    let fr = fr.unwrap();

    let io = i.create_file_io();
    assert!(io.is_ok());
    let io = io.unwrap();

    let open_flags =
        sys::PP_FILEOPENFLAG_READ |
        sys::PP_FILEOPENFLAG_WRITE |
        sys::PP_FILEOPENFLAG_CREATE;

    let open_result = i.open_file_io(io.id(), fr.id(),
                                     open_flags,
                                     Default::default());
    assert!(open_result.is_err());
}

#[test]
fn file_io_create_ref_count() {
    let (i, fs) = _create_filesystem_resource();

    fs.mkdir("/test-dir").unwrap();

    let path = Path::new("/test-dir/test-file").to_path_buf();
    let fr = i.create_file_ref(fs.id(), path);
    assert!(fr.is_ok());
    let _fr = fr.unwrap();

    let io = i.create_file_io();
    let io = io.unwrap();

    assert_eq!(io.ref_count(), 1);
}
#[test]
fn file_io_open_ref_count() {
    let (i, fs) = _create_filesystem_resource();

    fs.mkdir("/test-dir").unwrap();

    let path = Path::new("/test-dir/test-file").to_path_buf();
    let fr = i.create_file_ref(fs.id(), path);
    assert!(fr.is_ok());
    let fr = fr.unwrap();

    let io = i.create_file_io();
    let io = io.unwrap();

    let open_flags =
        sys::PP_FILEOPENFLAG_READ |
        sys::PP_FILEOPENFLAG_WRITE |
        sys::PP_FILEOPENFLAG_CREATE;

    let open_result = i.open_file_io(io.id(), fr.id(),
                                     open_flags,
                                     Default::default());
    let _open_result = open_result.unwrap();

    assert_eq!(io.ref_count(), 1);
}

#[test]
fn file_io_create() {
    let (i, fs) = _create_filesystem_resource();

    fs.mkdir("/test-dir").unwrap();

    let path = Path::new("/test-dir/test-file").to_path_buf();
    let fr = i.create_file_ref(fs.id(), path);
    assert!(fr.is_ok());
    let fr = fr.unwrap();

    let io = i.create_file_io();
    let io = io.unwrap();

    let open_flags =
        sys::PP_FILEOPENFLAG_READ |
        sys::PP_FILEOPENFLAG_WRITE |
        sys::PP_FILEOPENFLAG_CREATE;

    let open_result = i.open_file_io(io.id(), fr.id(),
                                     open_flags,
                                     Default::default());
    let _open_result = open_result.unwrap();


    assert_eq!(fr.uses().unwrap(), 1);

    let test_dir = fr.parent().unwrap();
    assert_eq!(test_dir.path(), Ok(Path::new("/test-dir").to_path_buf()));

    assert_eq!(test_dir.uses(), Ok(1));

    {
        let path = Path::new("/test-dir/test-file2").to_path_buf();
        let fr2 = i.create_file_ref(fs.id(), path);
        assert!(fr2.is_ok());
        let fr2 = fr2.unwrap();

        let io = i.create_file_io();
        assert!(io.is_ok());
        let io = io.unwrap();

        println!("io id `{}`", io.id());

        let open_result = i.open_file_io(io.id(), fr2.id(),
                                         open_flags,
                                         Default::default());
        assert!(open_result.is_ok());

        assert_eq!(fr2.uses().unwrap(), 1);

        let parent = fr2.parent().unwrap();
        assert_eq!(parent.id(), test_dir.id());

        assert_eq!(parent.uses(), Ok(2));
    }

    // force us to wait till the instance thread has processed the resource dtor msg.
    i.ping().unwrap();

    assert_eq!(test_dir.uses(), Ok(1));
}

#[test]
fn file_in_use() {
    let (_i, _fs) = _create_filesystem_resource();

    let path = Path::new("/test-dir/test-file").to_path_buf();
}
