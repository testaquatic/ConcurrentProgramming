use std::{
    ffi::{c_char, c_void, CStr},
    mem::MaybeUninit,
    process::exit,
    ptr,
};

use libc::{perror, pthread_create, pthread_join, sleep};

const NUM_THREAD: usize = 10;

// 스레드용 함수
extern "C" fn thread_func(arg: *mut c_void) -> *mut c_void {
    let id = arg as usize;
    (0..5).for_each(|i| {
        println!("id = {id}, i = {i}");
        unsafe {
            sleep(1);
        }
    });

    c"finished!".as_ptr() as *mut c_void
}

fn main() {
    unsafe {
        let v = (0..NUM_THREAD)
            .map(|i| {
                let mut v = MaybeUninit::uninit();
                if pthread_create(
                    v.as_mut_ptr(),
                    ptr::null_mut(),
                    thread_func,
                    i as *mut c_void,
                ) != 0
                {
                    perror(c"pthread_create".as_ptr());
                    exit(-1);
                }

                v.assume_init()
            })
            .collect::<Vec<_>>();

        v.iter().for_each(|v| {
            let mut ptr = MaybeUninit::<*mut c_void>::uninit();
            if pthread_join(*v, ptr.as_mut_ptr()) == 0 {
                let ret = ptr.assume_init();
                println!(
                    "msg = {}",
                    CStr::from_ptr(ret as *const c_char).to_str().unwrap()
                );
            } else {
                perror(c"pthread_join".as_ptr());
                exit(-1);
            }
        });
    }
}
