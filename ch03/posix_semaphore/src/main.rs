use std::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr,
    sync::atomic::{self, AtomicUsize},
};

use libc::{
    exit, perror, pthread_create, pthread_join, sem_close, sem_open, sem_post, sem_unlink,
    sem_wait, usleep, O_CREAT, O_EXCL, SEM_FAILED,
};

const NUM_THREAD: usize = 10;
const NUM_LOOP: usize = 10;

static COUNT: AtomicUsize = AtomicUsize::new(0);

extern "C" fn th(_arg: *mut c_void) -> *mut c_void {
    unsafe {
        let s = sem_open(c"/mysemaphore".as_ptr(), O_EXCL);
        if s == SEM_FAILED {
            perror(c"sem_open".as_ptr());
            exit(-1);
        }

        (0..NUM_LOOP).for_each(|_| {
            if sem_wait(s) == -1 {
                perror(c"sem_wait".as_ptr());
                exit(-1);
            }

            COUNT.fetch_add(1, atomic::Ordering::AcqRel);
            println!("count = {}", COUNT.load(atomic::Ordering::Relaxed));

            usleep(10000);

            COUNT.fetch_sub(1, atomic::Ordering::AcqRel);

            if sem_post(s) == -1 {
                perror(c"sem_post".as_ptr());
                exit(-1);
            }
        });

        if sem_close(s) == -1 {
            perror(c"sem_close".as_ptr());
            exit(-1);
        }
    }

    ptr::null_mut()
}

fn main() {
    unsafe {
        let s = sem_open(c"/mysemaphore".as_ptr(), O_CREAT, 0o660, 3);
        if s == SEM_FAILED {
            perror(c"sem_open".as_ptr());
            exit(-1);
        }

        // 스레드 생성
        let v = (0..NUM_THREAD)
            .map(|_| {
                let mut v = MaybeUninit::uninit();
                if pthread_create(v.as_mut_ptr(), ptr::null(), th, ptr::null_mut()) != 0 {
                    perror(c"pthread_create".as_ptr());
                    exit(-1);
                }
                v.assume_init()
            })
            .collect::<Vec<_>>();

        v.iter().for_each(|v| {
            pthread_join(*v, ptr::null_mut());
        });

        // 세마포어를 닫는다.
        if sem_close(s) == -1 {
            perror(c"sem_close".as_ptr());
            exit(-1);
        }

        // 세마포어 파기
        if sem_unlink(c"/mysemaphore".as_ptr()) == -1 {
            perror(c"sem_unlink".as_ptr());
        }
    }
}
