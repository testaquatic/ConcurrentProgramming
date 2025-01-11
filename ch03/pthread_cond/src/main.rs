#![allow(static_mut_refs)]

use std::{
    ffi::c_void,
    io::{self, Write},
    ptr::{self, read_volatile},
    sync::LazyLock,
};

use lib_pthread_cond::{PCond, PMutex, PThread};

static mut BUF: String = String::new();
static MUTEX: LazyLock<PMutex> = LazyLock::new(PMutex::new);
static COND: LazyLock<PCond> = LazyLock::new(PCond::new);
static mut READY: bool = false;

extern "C" fn producer(_: *mut c_void) -> *mut c_void {
    print!("producer: ");
    io::stdout().flush().unwrap();
    unsafe {
        BUF = io::stdin().lines().next().unwrap().unwrap();
    }
    MUTEX.lock().unwrap();
    unsafe {
        READY = true;
    }

    COND.broadcast().unwrap();
    MUTEX.unlock().unwrap();

    ptr::null_mut()
}

extern "C" fn consumer(_: *mut c_void) -> *mut c_void {
    MUTEX.lock().unwrap();
    while !unsafe { read_volatile(&raw const READY) } {
        COND.wait(&MUTEX).unwrap();
    }

    MUTEX.unlock().unwrap();
    unsafe {
        println!("consumer: {BUF}");
    }

    ptr::null_mut()
}

fn main() {
    unsafe {
        let pr = PThread::create(ptr::null_mut(), producer, ptr::null_mut()).unwrap();
        let cn = PThread::create(ptr::null_mut(), consumer, ptr::null_mut()).unwrap();

        pr.join(ptr::null_mut()).unwrap();
        cn.join(ptr::null_mut()).unwrap();
    }
}
