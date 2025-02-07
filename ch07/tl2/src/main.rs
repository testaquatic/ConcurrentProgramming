use core::panic;
use std::{hint, sync::Arc, thread, time};

use libtl2::stm::STMResult;

macro_rules! load {
    ($t:ident, $a:expr) => {
        if let Some(v) = ($t).load($a) {
            v
        } else {
            return libtl2::stm::STMResult::Retry;
        }
    };
}

macro_rules! store {
    ($t:ident, $a:expr, $v:expr) => {
        $t.store($a, $v);
    };
}

const NUM_PHILOSOPHERS: usize = 8;

fn philosopher(stm: Arc<libtl2::stm::STM>, n: usize) {
    let left = 8 * n;
    let right = 8 * ((n + 1) % NUM_PHILOSOPHERS);

    (0..500_000).for_each(|_| {
        while !stm
            .write_transaction(|tr| {
                let mut f1 = load!(tr, left);
                let mut f2 = load!(tr, right);
                if f1[0] == 0 && f2[0] == 0 {
                    f1[0] = 1;
                    f2[0] = 1;
                    store!(tr, left, f1);
                    store!(tr, right, f2);
                    STMResult::Ok(true)
                } else {
                    STMResult::Ok(false)
                }
            })
            .unwrap()
        {
            hint::spin_loop();
        }

        stm.write_transaction(|tr| {
            let mut f1 = load!(tr, left);
            let mut f2 = load!(tr, right);
            f1[0] = 0;
            f2[0] = 0;
            store!(tr, left, f1);
            store!(tr, right, f2);
            STMResult::Ok(())
        });
    });
}

fn observer(stm: Arc<libtl2::stm::STM>) {
    (0..10_000).for_each(|_| {
        let chopsticks = stm
            .read_transaction(|tr| {
                let mut v = [0; NUM_PHILOSOPHERS];

                for i in 0..NUM_PHILOSOPHERS {
                    v[i] = load!(tr, 8 * i)[0];
                }

                STMResult::Ok(v)
            })
            .unwrap();

        println!("{:?}", chopsticks);

        // 들고 있는 포크 수가 홀수면 올바르지 않음
        let n = chopsticks.iter().filter(|c| *c / 2 == 0).count();

        if n % 2 != 0 {
            panic!("inconsistent")
        }

        // 100 마이크로초 동안 슬립
        let us = time::Duration::from_micros(100);
        thread::sleep(us);
    });
}

fn main() {
    let stm = Arc::new(libtl2::stm::STM::new());
    let v = (0..NUM_PHILOSOPHERS)
        .map(|i| {
            let s = stm.clone();
            std::thread::spawn(move || philosopher(s, i))
        })
        .collect::<Vec<_>>();
    let obs = std::thread::spawn(move || observer(stm.clone()));

    v.into_iter().for_each(|th| th.join().unwrap());
    obs.join().unwrap();
}
