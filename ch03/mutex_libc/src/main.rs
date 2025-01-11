use std::{
    ffi::{c_char, c_void},
    mem, process, ptr, thread, time,
};

use libc::{
    perror, pthread_create, pthread_join, pthread_mutex_destroy, pthread_mutex_lock,
    pthread_mutex_t, pthread_mutex_unlock, pthread_t, PTHREAD_MUTEX_INITIALIZER,
};

// 뮤텍스 초기화
static mut MUTEX: pthread_mutex_t = PTHREAD_MUTEX_INITIALIZER;

extern "C" fn some_func(_arg: *mut c_void) -> *mut c_void {
    unsafe {
        // 락 획득
        if pthread_mutex_lock(&raw mut MUTEX) != 0 {
            perror("pthread_mutex_lock".as_ptr() as *const c_char);
            libc::exit(-1);
        }
    }

    // 크리티컬 섹션
    println!("start working");
    thread::sleep(time::Duration::from_secs(1));
    println!("end working");

    unsafe {
        // 락 해제
        if pthread_mutex_unlock(&raw mut MUTEX) != 0 {
            perror("pthread_mutex_unlock".as_ptr() as *const c_char);
            libc::exit(-1);
        }
    }

    std::ptr::null_mut()
}

fn main() {
    let th1 = unsafe { mem::zeroed::<pthread_t>() };
    let th2 = unsafe { mem::zeroed::<pthread_t>() };

    unsafe {
        if pthread_create(
            &raw const th1 as *mut pthread_t,
            ptr::null_mut(),
            some_func,
            ptr::null_mut(),
        ) != 0
        {
            perror("pthread_create".as_ptr() as *const c_char);
            process::exit(-1);
        }

        if pthread_create(
            &raw const th2 as *mut pthread_t,
            ptr::null_mut(),
            some_func,
            ptr::null_mut(),
        ) != 0
        {
            perror("pthread_create".as_ptr() as *const c_char);
            process::exit(-1);
        }

        if pthread_join(th1, ptr::null_mut()) != 0 {
            perror("pthread_join".as_ptr() as *const c_char);
            process::exit(-1);
        }

        if pthread_join(th2, ptr::null_mut()) != 0 {
            perror("pthread_join".as_ptr() as *const c_char);
            process::exit(-1);
        }

        // 뮤텍스 해제
        if pthread_mutex_destroy(&raw mut MUTEX) != 0 {
            perror("pthread_mutex_destroy".as_ptr() as *const c_char);
            process::exit(-1);
        }
    }
}
