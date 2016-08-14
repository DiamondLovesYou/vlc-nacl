
use libc;

pub fn global_singleton<T>(f: fn() -> T) -> &'static T
    where T: Sync,
{
    use std::sync::atomic::{AtomicPtr, Ordering};
    use std::sync::{Once, ONCE_INIT};

    static G: AtomicPtr<libc::c_void> = AtomicPtr::new(0 as _);
    static I: Once = ONCE_INIT;

    let v: Option<&T> = unsafe {
        let typeless = G.load(Ordering::SeqCst);
        let typed = typeless as *const T;
        typed.as_ref()
    };
    if let Some(g) = v {
        g
    } else {
        I.call_once(|| {
            let v = f();
            let v = Box::new(v);
            let v = Box::into_raw(v);

            G.store(v as *mut libc::c_void, Ordering::SeqCst);
        });

        global_singleton::<T>(f)
    }
}
pub fn global_singleton_default<T>() -> &'static T
    where T: Default + Sync,
{
    fn init<T>() -> T
        where T: Default,
    {
        Default::default()
    }

    global_singleton::<T>(init::<T>)
}
