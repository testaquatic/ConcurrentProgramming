#![allow(static_mut_refs)]

use std::{
    ffi::c_void,
    mem::MaybeUninit,
    process::{self, exit},
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

use libc::{
    pthread_cond_broadcast, pthread_cond_t, pthread_cond_wait, pthread_create, pthread_join,
    pthread_mutex_lock, pthread_mutex_t, pthread_mutex_unlock, PTHREAD_COND_INITIALIZER,
    PTHREAD_MUTEX_INITIALIZER,
};

static mut BARRIER_MUT: pthread_mutex_t = PTHREAD_MUTEX_INITIALIZER;
static mut BARRIER_COND: pthread_cond_t = PTHREAD_COND_INITIALIZER;
static NUM: AtomicUsize = AtomicUsize::new(0);

fn barrier(cnt: &AtomicUsize, max: usize) {
    unsafe {
        if pthread_mutex_lock(&raw mut BARRIER_MUT) != 0 {
            eprintln!("pthread_mutex_lock");
            exit(-1);
        }

        cnt.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if cnt.load(Ordering::SeqCst) == max {
            if pthread_cond_broadcast(&raw mut BARRIER_COND) != 0 {
                eprintln!("pthread_cond_broadcast");
                exit(-1);
            }
        } else {
            while cnt.load(Ordering::Relaxed) < max {
                if pthread_cond_wait(&raw mut BARRIER_COND, &raw mut BARRIER_MUT) != 0 {
                    eprintln!("pthread_cond_wait");
                    exit(-1);
                }
            }
        }

        if pthread_mutex_unlock(&raw mut BARRIER_MUT) != 0 {
            eprintln!("pthread_mutex_unlock");
            exit(-1);
        }
    }
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
