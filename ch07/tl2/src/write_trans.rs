use std::{
    collections::{HashMap, HashSet},
    mem::MaybeUninit,
    sync::atomic::{self, fence},
};

use crate::{memory::Memory, ADDR_CHECK_MASK, STRIPE_SIZE};

pub struct WriteTrans<'a> {
    pub read_ver: u64,
    read_set: HashSet<usize>,
    write_set: HashMap<usize, [u8; STRIPE_SIZE]>,
    /// 락 완료 주소
    locked: Vec<usize>,
    /// 경합을 감지하면 true
    pub is_abort: bool,
    pub mem: &'a mut Memory,
}

impl Drop for WriteTrans<'_> {
    fn drop(&mut self) {
        self.locked.iter().for_each(|addr| {
            self.mem.unlock_addr(*addr);
        });
    }
}

impl WriteTrans<'_> {
    pub fn new(mem: &mut Memory) -> WriteTrans {
        WriteTrans {
            read_ver: mem.global_clock.load(atomic::Ordering::Acquire),
            read_set: HashSet::new(),
            write_set: HashMap::new(),
            locked: Vec::new(),
            is_abort: false,
            mem,
        }
    }

    pub fn store(&mut self, addr: usize, val: [u8; STRIPE_SIZE]) {
        assert_eq!(addr & ADDR_CHECK_MASK, 0);
        self.write_set.insert(addr, val);
    }

    pub fn load(&mut self, addr: usize) -> Option<[u8; STRIPE_SIZE]> {
        // 경합을 감지한 경우 종료
        if self.is_abort {
            return None;
        }

        // 주소가 스트라이프 자릿수와 맞는지 확인
        assert_eq!(addr & ADDR_CHECK_MASK, 0);

        // 읽기 주소 저장
        self.read_set.insert(addr);

        // write_set에 있다면 이를 읽음
        if let Some(m) = self.write_set.get(&addr) {
            return Some(*m);
        }

        if !self.mem.test_not_modify(addr, self.read_ver) {
            self.is_abort = true;
            return None;
        }

        fence(atomic::Ordering::Acquire);

        let mem = unsafe {
            let mut mem: MaybeUninit<[u8; STRIPE_SIZE]> = MaybeUninit::uninit();
            let mem_ptr = mem.as_mut_ptr() as *mut u8;
            mem_ptr.copy_from_nonoverlapping(self.mem.mem.as_ptr().add(addr), STRIPE_SIZE);
            mem.assume_init()
        };

        fence(atomic::Ordering::SeqCst);

        // 읽기 메모리가 락되어 있지 않고 read_version 이하인지 확인
        if !self.mem.test_not_modify(addr, self.read_ver) {
            self.is_abort = true;
            return None;
        }

        Some(mem)
    }

    /// write_set 안의 주소를 락
    /// 모든 주소의 락을 획득 할 수 있는 경우 true를 반환
    pub fn lock_write_set(&mut self) -> bool {
        for (addr, _) in self.write_set.iter() {
            if self.mem.lock_addr(*addr) {
                self.locked.push(*addr);
            } else {
                return false;
            }
        }

        true
    }

    /// read_set 검증
    pub fn validate_read_set(&self) -> bool {
        self.read_set.iter().all(|addr| {
            // write_set 안에 있는 주소인 경우에는 자기 스레드가 락을 획득한 상태임
            if self.write_set.contains_key(addr) {
                let ver = self.mem.get_addr_ver(*addr);
                if ver > self.read_ver {
                    return false;
                }
            } else if !self.mem.test_not_modify(*addr, self.read_ver) {
                return false;
            }

            true
        })
    }

    /// 커밋
    pub fn commit(&mut self, ver: u64) {
        self.write_set.iter().for_each(|(addr, val)| {
            let addr = *addr;
            self.mem
                .mem
                .get_mut(addr..addr + STRIPE_SIZE)
                .unwrap()
                .copy_from_slice(val);
        });

        fence(atomic::Ordering::Release);

        // 모든 주소의 락 해제 및 버전 업데이트
        self.write_set.iter().for_each(|(addr, _)| {
            let idx = addr >> self.mem.shift_size;
            self.mem
                .lock_ver
                .get(idx)
                .unwrap()
                .store(ver, atomic::Ordering::Relaxed);
        });

        // 락 완료 주소 집합 초기화
        self.locked.clear();
    }
}
