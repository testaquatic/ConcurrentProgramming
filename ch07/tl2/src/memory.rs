use std::sync::atomic::{self, AtomicU64};

use crate::{LOCK_MASK, MEM_SIZE, STRIPE_SIZE, VER_MASK};

// 메모리 타입
pub struct Memory {
    pub mem: Vec<u8>,
    pub lock_ver: Vec<AtomicU64>,
    pub global_clock: AtomicU64,

    // 주소에서 스트라이프 번호로 변환하기 위한 이동량
    // 2 ^ n 이어야 한다.
    pub shift_size: u32,
}

impl Memory {
    pub fn new() -> Memory {
        let mem = vec![0; MEM_SIZE];
        let shift_size = STRIPE_SIZE.trailing_zeros();
        // 컴파일러에서 최적화할 가능성이 높으므로 가독성을 위해서 비트시프트 연산을 사용하지 않는다.
        let lock_ver = (0..MEM_SIZE >> shift_size)
            .map(|_| AtomicU64::new(0))
            .collect();

        Memory {
            mem,
            lock_ver,
            global_clock: AtomicU64::new(0),
            shift_size,
        }
    }

    pub fn inc_global_clock(&self) -> u64 {
        self.global_clock.fetch_add(1, atomic::Ordering::AcqRel)
    }

    // 대상 주소의 버전 취득
    pub fn get_addr_ver(&self, addr: usize) -> u64 {
        let idx = addr >> self.shift_size;
        let n = self.lock_ver[idx].load(atomic::Ordering::Relaxed);
        // 최상위 비트는 락용 비트이다.
        n & VER_MASK
    }

    // 대상 주소의 버전이 rv 이하로 락되어 있지 않은지 확인
    pub fn test_not_modify(&self, addr: usize, rv: u64) -> bool {
        let idx = addr >> self.shift_size;
        let n = self.lock_ver[idx].load(atomic::Ordering::Relaxed);
        // 최상위 비트는 락용 비트이다.
        n <= rv
    }

    // 대상 주소의 락 획득
    // 락을 획득했다면 true를 설정한다.
    pub fn lock_addr(&self, addr: usize) -> bool {
        let idx = addr >> self.shift_size;
        self.lock_ver[idx]
            .fetch_update(
                atomic::Ordering::Relaxed,
                atomic::Ordering::Relaxed,
                |val| {
                    // 락이 되었는지 확인
                    if val & LOCK_MASK == 0 {
                        Some(val | LOCK_MASK)
                    } else {
                        None
                    }
                },
            )
            .is_ok()
    }

    // 대상 주소의 락 해제
    pub fn unlock_addr(&self, addr: usize) {
        let idx = addr >> self.shift_size;
        self.lock_ver[idx].fetch_and(VER_MASK, atomic::Ordering::Relaxed);
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
