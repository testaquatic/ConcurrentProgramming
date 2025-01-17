use std::{ffi::c_int, process};

use libc::{
    pthread_cond_t, pthread_cond_wait, pthread_mutex_lock, pthread_mutex_t, pthread_mutex_unlock,
    signal, PTHREAD_COND_INITIALIZER, PTHREAD_MUTEX_INITIALIZER, SIGUSR1,
};

static mut MUTEX: pthread_mutex_t = PTHREAD_MUTEX_INITIALIZER;
static mut COND: pthread_cond_t = PTHREAD_COND_INITIALIZER;

extern "C" fn handler(sig: c_int) {
    println!("Received signal {}", sig);
}

fn main() {
    let pid = process::id() as c_int;
    println!("pid: {}", pid);

    unsafe {
        signal(SIGUSR1, handler as usize);
        pthread_mutex_lock(&raw mut MUTEX);
        if pthread_cond_wait(&raw mut COND, &raw mut MUTEX) != 0 {
            println!("pthread_cond_wait failed");
            process::exit(1);
        }
        println!("sprious wake up");
        pthread_mutex_unlock(&raw mut MUTEX);
    }
}
