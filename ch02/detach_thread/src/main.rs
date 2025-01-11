use std::{ffi::c_void, mem::MaybeUninit, ptr, thread, time::Duration};

use libc::{
    exit, perror, pthread_attr_destroy, pthread_attr_init, pthread_attr_setdetachstate,
    pthread_create, PTHREAD_CREATE_DETACHED,
};

extern "C" fn thread_func(_: *mut c_void) -> *mut c_void {
    (0..5).for_each(|i| {
        println!("i = {i}");
        thread::sleep(Duration::from_secs(1));
    });

    ptr::null_mut()
}

fn main() {
    let mut attr = MaybeUninit::uninit();
    unsafe {
        if pthread_attr_init(attr.as_mut_ptr()) != 0 {
            perror(c"pthread_attr_init".as_ptr());
            exit(-1);
        }
        let mut attr = attr.assume_init();
        // 디태치 스레드로 설정
        if pthread_attr_setdetachstate(&raw mut attr, PTHREAD_CREATE_DETACHED) != 0 {
            perror(c"pthread_attr_setdetachstate".as_ptr());
            exit(-1);
        }

        // 어트리뷰터를 지정해 스레드 생성
        let mut th = MaybeUninit::uninit();
        if pthread_create(th.as_mut_ptr(), &raw mut attr, thread_func, ptr::null_mut()) != 0 {
            perror(c"pthread_create".as_ptr());
            exit(-1);
        }
        let _th = th.assume_init();

        // 어트리뷰트 파기
        if pthread_attr_destroy(&raw mut attr) != 0 {
            perror(c"pthread_attr_destroy".as_ptr());
            exit(-1);
        }
    }

    thread::sleep(Duration::from_secs(7));
}
