use std::cell::UnsafeCell;

use crate::{memory::Memory, read_trans::ReadTrans, write_trans::WriteTrans};

pub enum STMResult<T> {
    Ok(T),
    Retry,
    Abort,
}

pub struct STM {
    mem: UnsafeCell<Memory>,
}

unsafe impl Send for STM {}
unsafe impl Sync for STM {}

impl STM {
    pub fn new() -> STM {
        STM {
            mem: UnsafeCell::new(Memory::new()),
        }
    }

    pub fn read_transaction<F, R>(&self, f: F) -> Option<R>
    where
        F: Fn(&mut ReadTrans) -> STMResult<R>,
    {
        loop {
            // global version clock 읽기
            let mut tr = ReadTrans::new(unsafe { &*self.mem.get() });

            // 투기적 실행
            match f(&mut tr) {
                STMResult::Ok(val) => {
                    if tr.is_abort {
                        continue;
                    }
                    return Some(val);
                }
                STMResult::Retry => {
                    if tr.is_abort {
                        continue;
                    }
                    return None;
                }
                STMResult::Abort => return None,
            }
        }
    }

    // 쓰기 트랜잭션
    pub fn write_transaction<F, R>(&self, f: F) -> Option<R>
    where
        F: Fn(&mut WriteTrans) -> STMResult<R>,
    {
        loop {
            let mut tr = WriteTrans::new(unsafe { &mut *self.mem.get() });

            // 투기적 실행
            let result = match f(&mut tr) {
                STMResult::Ok(val) => {
                    if tr.is_abort {
                        continue;
                    }
                    val
                }
                STMResult::Retry => {
                    if tr.is_abort {
                        continue;
                    }
                    return None;
                }
                STMResult::Abort => return None,
            };

            if !tr.lock_write_set() {
                continue;
            }

            let ver = 1 + tr.mem.inc_global_clock();

            if tr.read_ver + 1 != ver && !tr.validate_read_set() {
                continue;
            }

            tr.commit(ver);

            return Some(result);
        }
    }
}

impl Default for STM {
    fn default() -> Self {
        Self::new()
    }
}
