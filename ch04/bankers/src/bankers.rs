use std::{
    ops::ControlFlow,
    sync::{Arc, Mutex},
};

struct Resource<const NRES: usize, const NTH: usize> {
    // 이용 가능한 리소스
    available: [usize; NRES],
    // 스레드 i가 확보 중인 리소스
    allocation: [[usize; NRES]; NTH],
    // 스레드 i가 필요로 하는 리소스의 최댓값
    max: [[usize; NRES]; NTH],
}

impl<const NRES: usize, const NTH: usize> Resource<NRES, NTH> {
    fn new(available: [usize; NRES], max: [[usize; NRES]; NTH]) -> Self {
        Self {
            available,
            allocation: [[0; NRES]; NTH],
            max,
        }
    }

    // 현재 상태가 데드락을 발생시키지 않는지 확인
    fn is_safe(&self) -> bool {
        // 스레드 i는 리소스 획득과 반환에 성공했는가?
        let mut finish = [false; NTH];
        // 이용 가능한 리소스의 시뮬레이션 값
        let mut work = self.available.clone();

        loop {
            // 모든 스레드 i와 리소스 j에 대해서 finished[i] == false && work[j] >= (self.max[i][j] - self.allocation[i][j])을 만족하는 스레드를 찾는다.
            let mut found = false;
            let mut num_true = 0;

            self.allocation.iter().enumerate().try_for_each(|(i, alc)| {
                if finish[i] {
                    num_true += 1;
                    return ControlFlow::Continue(());
                }

                // need[j] = self.max[i][j] - self.allocation[i][j]를 계산하고 모든 리소스 j에 대해 work[j] >= need[j]인지 판정한다.
                let need = self.max[i].iter().zip(alc).map(|(max, alloc)| max - alloc);
                let is_avail = work.iter().zip(need).all(|(w, n)| *w >= n);
                if is_avail {
                    // 스레드 i가 리소스 확보 가능
                    found = true;
                    finish[i] = true;
                    work.iter_mut().zip(alc).for_each(|(w, a)| *w += *a);
                    return ControlFlow::Break(i);
                }

                ControlFlow::Continue(())
            });

            if num_true == NTH {
                return true;
            }

            if !found {
                break;
            }
        }
        false
    }

    // id번째 스레드가 resource를 하나 얻음
    fn take(&mut self, id: usize, resource: usize) -> bool {
        if id >= NTH || resource >= NRES || self.available[resource] == 0 {
            return false;
        }

        self.allocation[id][resource] += 1;
        self.available[resource] -= 1;

        if self.is_safe() {
            true
        } else {
            self.allocation[id][resource] -= 1;
            self.available[resource] += 1;
            false
        }
    }

    // id번째 스레드가 resource를 하나 반환
    fn release(&mut self, id: usize, resource: usize) {
        if id >= NTH || resource >= NRES || self.allocation[id][resource] == 0 {
            return;
        }

        self.allocation[id][resource] -= 1;
        self.available[resource] += 1;
    }
}

#[derive(Clone)]
pub struct Bankers<const NRES: usize, const NTH: usize> {
    resource: Arc<Mutex<Resource<NRES, NTH>>>,
}

impl<const NRES: usize, const NTH: usize> Bankers<NRES, NTH> {
    pub fn new(available: [usize; NRES], max: [[usize; NRES]; NTH]) -> Self {
        Self {
            resource: Arc::new(Mutex::new(Resource::new(available, max))),
        }
    }

    pub fn take(&self, id: usize, resource: usize) -> bool {
        let mut r = self.resource.lock().unwrap();
        r.take(id, resource)
    }

    pub fn release(&self, id: usize, resource: usize) {
        let mut r = self.resource.lock().unwrap();
        r.release(id, resource);
    }
}
