use std::{
    ffi::{c_int, c_void},
    mem::{self},
    process, ptr, thread,
    time::Duration,
};

use libc::{
    exit, perror, pthread_create, pthread_detach, pthread_join, pthread_mutex_lock,
    pthread_mutex_t, pthread_mutex_unlock, pthread_self, pthread_sigmask, sigaddset, sigemptyset,
    sigset_t, sigwait, PTHREAD_MUTEX_INITIALIZER, SIGUSR1, SIG_BLOCK,
};

static mut MUTEX: pthread_mutex_t = PTHREAD_MUTEX_INITIALIZER;
static mut SET: sigset_t = unsafe { mem::zeroed() };

#[allow(unreachable_code)]
extern "C" fn handler(_: *mut c_void) -> *mut c_void {
    unsafe {
        pthread_detach(pthread_self());

        let mut sig: c_int = mem::zeroed();

        loop {
            if sigwait(&raw const SET, &mut sig) != 0 {
                perror(c"sigwait".as_ptr());
                exit(-1);
            }
            println!("received signal: {}", sig);
            pthread_mutex_lock(&raw mut MUTEX);
            thread::sleep(Duration::from_secs(1));
            pthread_mutex_unlock(&raw mut MUTEX);
        }
    }

    ptr::null_mut()
}

extern "C" fn worker(_: *mut c_void) -> *mut c_void {
    (0..10).for_each(|_| unsafe {
        pthread_mutex_lock(&raw mut MUTEX);
        thread::sleep(Duration::from_secs(1));
        pthread_mutex_unlock(&raw mut MUTEX);
        thread::sleep(Duration::from_secs(1));
    });

    ptr::null_mut()
}

fn main() {
    let pid = process::id();
    println!("pid: {}", pid);

    unsafe {
        sigemptyset(&raw mut SET);
        sigaddset(&raw mut SET, SIGUSR1);
        if pthread_sigmask(SIG_BLOCK, &raw const SET, ptr::null_mut()) != 0 {
            perror(c"pthread_sigmask".as_ptr());
            exit(1);
        }

        let mut th = mem::zeroed();
        let mut wth = mem::zeroed();
        pthread_create(&mut th, ptr::null(), handler, ptr::null_mut());
        pthread_create(&mut wth, ptr::null(), worker, ptr::null_mut());
        pthread_join(wth, ptr::null_mut());
    }
}
