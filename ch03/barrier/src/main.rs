use std::{
    ffi::c_void,
    mem::MaybeUninit,
    process::{self, exit},
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

use libc::{pthread_create, pthread_join};

static NUM: AtomicUsize = AtomicUsize::new(0);

fn barrier(cnt: &AtomicUsize, max: usize) {
    cnt.fetch_add(1, Ordering::AcqRel);
    while cnt.load(Ordering::Relaxed) < max {}
}

extern "C" fn worker(_: *mut c_void) -> *mut c_void {
    barrier(&NUM, 10);

    let t_id = thread::current();
    println!("Start: {:?}", t_id);
    thread::sleep(std::time::Duration::from_secs(1));
    println!("End: {:?}", t_id);

    ptr::null_mut()
}

fn main() {
    let v = (0..10)
        .map(|_| {
            let mut th = MaybeUninit::uninit();
            unsafe {
                if pthread_create(th.as_mut_ptr(), ptr::null_mut(), worker, ptr::null_mut()) != 0 {
                    eprintln!("pthread_create");
                    exit(-1);
                }
                thread::sleep(std::time::Duration::from_secs(1));
                th.assume_init()
            }
        })
        .collect::<Vec<_>>();

    v.iter().for_each(|th| unsafe {
        if pthread_join(*th, ptr::null_mut()) != 0 {
            eprintln!("pthread_join");
            process::exit(-1);
        }
    });
}
