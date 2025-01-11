#![allow(clippy::missing_safety_doc)]

use core::fmt;
use std::{error::Error, ffi::c_void, mem::MaybeUninit};

use libc::{
    pthread_attr_t, pthread_cond_broadcast, pthread_cond_destroy, pthread_cond_t,
    pthread_cond_wait, pthread_create, pthread_join, pthread_mutex_destroy, pthread_mutex_lock,
    pthread_mutex_t, pthread_mutex_unlock, pthread_t, PTHREAD_COND_INITIALIZER,
    PTHREAD_MUTEX_INITIALIZER,
};

pub struct PMutex {
    p_mutex: pthread_mutex_t,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PError(String);

impl fmt::Display for PError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PError {}

impl PMutex {
    pub fn new() -> PMutex {
        PMutex {
            p_mutex: PTHREAD_MUTEX_INITIALIZER,
        }
    }

    fn mtx_mut_ptr(&self) -> *mut pthread_mutex_t {
        &raw const self.p_mutex as *mut pthread_mutex_t
    }

    pub fn lock(&self) -> Result<(), PError> {
        unsafe {
            match pthread_mutex_lock(self.mtx_mut_ptr()) {
                0 => Ok(()),
                _ => Err(PError("PMutex::lock".to_string())),
            }
        }
    }
    pub fn unlock(&self) -> Result<(), PError> {
        unsafe {
            match pthread_mutex_unlock(self.mtx_mut_ptr()) {
                0 => Ok(()),
                _ => Err(PError("PMutex::unlock".to_string())),
            }
        }
    }
}

impl Drop for PMutex {
    fn drop(&mut self) {
        unsafe {
            if pthread_mutex_destroy(self.mtx_mut_ptr()) != 0 {
                panic!("Failed to destroy mutex");
            }
        }
    }
}

impl Default for PMutex {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PCond {
    p_cond: pthread_cond_t,
}

impl PCond {
    pub fn new() -> PCond {
        PCond {
            p_cond: PTHREAD_COND_INITIALIZER,
        }
    }

    fn cond_mut_ptr(&self) -> *mut pthread_cond_t {
        &raw const self.p_cond as *mut pthread_cond_t
    }

    pub fn wait(&self, mtx: &PMutex) -> Result<(), PError> {
        unsafe {
            match pthread_cond_wait(self.cond_mut_ptr(), mtx.mtx_mut_ptr()) {
                0 => Ok(()),
                _ => Err(PError("PCond::wait".to_string())),
            }
        }
    }

    pub fn broadcast(&self) -> Result<(), PError> {
        unsafe {
            match pthread_cond_broadcast(self.cond_mut_ptr()) {
                0 => Ok(()),
                _ => Err(PError("PCond::broadcast".to_string())),
            }
        }
    }
}

impl Drop for PCond {
    fn drop(&mut self) {
        unsafe {
            if pthread_cond_destroy(self.cond_mut_ptr()) != 0 {
                panic!("Failed to destroy condition variable");
            }
        }
    }
}

impl Default for PCond {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PThread {
    pthread_t: pthread_t,
}

impl PThread {
    pub unsafe fn create(
        attr: *const pthread_attr_t,
        f: extern "C" fn(*mut c_void) -> *mut c_void,
        value: *mut c_void,
    ) -> Result<PThread, PError> {
        let mut th = MaybeUninit::<pthread_t>::uninit();
        match pthread_create(th.as_mut_ptr(), attr, f, value) {
            0 => Ok(PThread {
                pthread_t: th.assume_init(),
            }),
            _ => Err(PError("PThread::create".to_string())),
        }
    }

    pub unsafe fn join(&self, value: *mut *mut c_void) -> Result<(), PError> {
        match pthread_join(self.pthread_t, value) {
            0 => Ok(()),
            _ => Err(PError("PThread::join".to_string())),
        }
    }
}
