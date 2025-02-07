pub mod memory;
pub mod read_trans;
pub mod stm;
pub mod write_trans;

// 스트라이프 크기
// 8 바이트
const STRIPE_SIZE: usize = 8;

// 주소가 스트라이프의 자릿수와 맞는지 확인할 때 사용하는 상수
const ADDR_CHECK_MASK: usize = STRIPE_SIZE - 1;

// 메모리 크기
// 64개의 스트라이프를 이용할 수 있다.
const MEM_SIZE: usize = 512;

const LOCK_MASK: u64 = 0x8000_0000_0000_0000;
const VER_MASK: u64 = !LOCK_MASK;
