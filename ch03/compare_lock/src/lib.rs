use std::{
    process::{self},
    sync::atomic::{AtomicUsize, Ordering},
};

use libc::{
    pthread_cond_broadcast, pthread_cond_t, pthread_cond_wait, pthread_mutex_lock, pthread_mutex_t,
    pthread_mutex_unlock, PTHREAD_COND_INITIALIZER, PTHREAD_MUTEX_INITIALIZER,
};

#[cfg(all(feature = "empty", not(feature = "not_empty")))]
pub mod lock {
    use std::arch::asm;

    pub fn do_lock(holdtime: usize) {
        (0..holdtime).for_each(|_| unsafe {
            asm!("nop");
        });
    }
}

#[cfg(feature = "mutex")]
pub mod lock {
    use std::arch::asm;

    use libc::{pthread_mutex_t, PTHREAD_MUTEX_INITIALIZER};

    static mut LOCK: pthread_mutex_t = PTHREAD_MUTEX_INITIALIZER;
    pub fn do_lock(holdtime: usize) {
        unsafe {
            libc::pthread_mutex_lock(&raw mut LOCK);
            (0..holdtime).for_each(|_| asm!("nop"));
            libc::pthread_mutex_unlock(&raw mut LOCK);
        }
    }
}

#[cfg(feature = "rwlock")]
pub mod lock {
    use std::arch::asm;

    use libc::{pthread_rwlock_t, PTHREAD_RWLOCK_INITIALIZER};

    static mut LOCK: pthread_rwlock_t = PTHREAD_RWLOCK_INITIALIZER;
    pub fn do_lock(holdtime: usize) {
        unsafe {
            libc::pthread_rwlock_rdlock(&raw mut LOCK);
            (0..holdtime).for_each(|_| asm!("nop"));
            libc::pthread_rwlock_unlock(&raw mut LOCK);
        }
    }
}

#[cfg(feature = "rwlock_wc")]
pub mod lock {
    use std::arch::asm;

    use libc::{pthread_rwlock_t, PTHREAD_RWLOCK_INITIALIZER};

    static mut LOCK: pthread_rwlock_t = PTHREAD_RWLOCK_INITIALIZER;
    pub fn do_lock(holdtime: usize) {
        unsafe {
            libc::pthread_rwlock_wrlock(&raw mut LOCK);
            (0..holdtime).for_each(|_| asm!("nop"));
            libc::pthread_rwlock_unlock(&raw mut LOCK);
        }
    }
}

static mut BARRIER_MUT: pthread_mutex_t = PTHREAD_MUTEX_INITIALIZER;
static mut BARRIER_COND: pthread_cond_t = PTHREAD_COND_INITIALIZER;

pub fn barrier(cnt: &AtomicUsize, max: usize) {
    unsafe {
        if pthread_mutex_lock(&raw mut BARRIER_MUT) != 0 {
            eprintln!("pthread_mutex_lock");
            process::exit(-1);
        }

        cnt.fetch_add(1, Ordering::Relaxed);

        if cnt.load(Ordering::SeqCst) == max {
            if pthread_cond_broadcast(&raw mut BARRIER_COND) != 0 {
                eprintln!("pthread_cond_broadcast");
                process::exit(-1);
            }
        } else {
            while cnt.load(Ordering::Relaxed) < max {
                if pthread_cond_wait(&raw mut BARRIER_COND, &raw mut BARRIER_MUT) != 0 {
                    eprintln!("pthread_cond_wait");
                    process::exit(-1);
                }
            }
        }

        if pthread_mutex_unlock(&raw mut BARRIER_MUT) != 0 {
            eprintln!("pthread_mutex_unlock");
            process::exit(-1);
        }
    }
}
