use std::{
    mem::MaybeUninit,
    sync::atomic::{self, fence},
};

use crate::{memory::Memory, ADDR_CHECK_MASK, STRIPE_SIZE};

pub struct ReadTrans<'a> {
    read_ver: u64,
    // 경함을 감지하면 true
    pub is_abort: bool,
    mem: &'a Memory,
}

impl ReadTrans<'_> {
    pub fn new(mem: &Memory) -> ReadTrans {
        ReadTrans {
            read_ver: mem.global_clock.load(atomic::Ordering::Acquire),
            is_abort: false,
            mem,
        }
    }

    // 메모리 읽기 함수
    pub fn load(&mut self, addr: usize) -> Option<[u8; STRIPE_SIZE]> {
        // 경합을 감지하면 종료
        if self.is_abort {
            return None;
        }

        // 주소가 스트라이프의 자릿수와 맞는지 확인
        assert_eq!(addr & ADDR_CHECK_MASK, 0);

        // 읽기 메모리가 락 되어 있거나 read_version 이상이면 반환
        if !self.mem.test_not_modify(addr, self.read_ver) {
            self.is_abort = true;
            return None;
        }

        fence(atomic::Ordering::Acquire);
        let mem = unsafe {
            let mut mem: MaybeUninit<[u8; 8]> = MaybeUninit::uninit();
            let mem_ptr = mem.as_mut_ptr() as *mut u8;
            mem_ptr.copy_from_nonoverlapping(self.mem.mem.as_ptr().add(addr), STRIPE_SIZE);
            mem.assume_init()
        };
        fence(atomic::Ordering::SeqCst);

        if !self.mem.test_not_modify(addr, self.read_ver) {
            self.is_abort = true;
            return None;
        }

        Some(mem)
    }
}
