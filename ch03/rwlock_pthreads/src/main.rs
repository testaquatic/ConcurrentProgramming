use std::{
    ffi::c_void,
    mem::MaybeUninit,
    process, ptr,
    thread::{self},
};

use libc::{
    pthread_create, pthread_join, pthread_rwlock_destroy, pthread_rwlock_rdlock, pthread_rwlock_t,
    pthread_rwlock_unlock, pthread_rwlock_wrlock, PTHREAD_RWLOCK_INITIALIZER,
};

static mut RWLOCK: pthread_rwlock_t = PTHREAD_RWLOCK_INITIALIZER;
static mut NUM: usize = 0;

extern "C" fn reader(_: *mut c_void) -> *mut c_void {
    unsafe {
        if pthread_rwlock_rdlock(&raw mut RWLOCK) != 0 {
            eprintln!("pthread_rwlock_rdlock");
            process::exit(-1);
        }
    }

    println!("RS: {:?}", thread::current().id());
    println!("RR: {}", unsafe { NUM });
    println!("RE: {:?}", thread::current().id());

    unsafe {
        if pthread_rwlock_unlock(&raw mut RWLOCK) != 0 {
            eprintln!("pthread_rwlock_unlock");
            process::exit(-1);
        }
    }

    ptr::null_mut()
}

extern "C" fn writer(_: *mut c_void) -> *mut c_void {
    unsafe {
        if pthread_rwlock_wrlock(&raw mut RWLOCK) != 0 {
            eprintln!("pthread_rwlock_wrlock");
            process::exit(-1);
        }
    }

    println!("WS: {:?}", thread::current().id());
    unsafe {
        NUM += 1;
    }
    println!("WR: {} -> {}", unsafe { NUM - 1 }, unsafe { NUM });
    println!("WE: {:?}", thread::current().id());

    unsafe {
        if pthread_rwlock_unlock(&raw mut RWLOCK) != 0 {
            eprintln!("pthread_rwlock_unlock");
            process::exit(-1);
        }
    }

    ptr::null_mut()
}

fn main() {
    let mut rd = MaybeUninit::uninit();
    let mut wr = MaybeUninit::uninit();

    unsafe {
        if pthread_create(rd.as_mut_ptr(), ptr::null(), reader, ptr::null_mut()) != 0 {
            eprintln!("pthread_create");
            process::exit(-1);
        }
        if pthread_create(wr.as_mut_ptr(), ptr::null(), writer, ptr::null_mut()) != 0 {
            eprintln!("pthread_create");
            process::exit(-1);
        }
    }
    let rd = unsafe { rd.assume_init() };
    let wr = unsafe { wr.assume_init() };

    unsafe {
        if pthread_join(rd, ptr::null_mut()) != 0 {
            eprintln!("pthread_join");
            process::exit(-1);
        }
        if pthread_join(wr, ptr::null_mut()) != 0 {
            eprintln!("pthread_join");
            process::exit(-1);
        }

        if pthread_rwlock_destroy(&raw mut RWLOCK) != 0 {
            eprintln!("pthread_rwlock_destroy");
            process::exit(-1);
        }
    }
}
