use std::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use clap::{Arg, Command};
use lib_compare_lock::{barrier, lock::do_lock};
use libc::{pthread_create, pthread_detach, pthread_join};

struct CommandArgs {
    num_thread: usize,
    hold_time: usize,
}

fn get_command_args() -> CommandArgs {
    let matches = Command::new("compare_lock")
        .arg(
            Arg::new("num_thread")
                .short('n')
                .long("num_thread")
                .value_name("NUM_THREAD")
                .num_args(1)
                .help("Number of threads to run")
                .required(true)
                .value_parser(clap::value_parser!(usize))
                .default_value("8"),
        )
        .arg(
            Arg::new("hold_time")
                .short('t')
                .long("hold_time")
                .value_name("HOLD_TIME")
                .num_args(1)
                .required(true)
                .value_parser(clap::value_parser!(usize))
                .default_value("1000"),
        )
        .get_matches();

    CommandArgs {
        num_thread: matches.get_one("num_thread").cloned().unwrap(),
        hold_time: matches.get_one("hold_time").cloned().unwrap(),
    }
}

struct WorkerArgs {
    num_thread: usize,
    hold_time: usize,
    count: Arc<Mutex<Vec<usize>>>,
}

static WAITING1: AtomicUsize = AtomicUsize::new(0);
static WAITING2: AtomicUsize = AtomicUsize::new(0);
static FALG: AtomicUsize = AtomicUsize::new(0);

extern "C" fn worker(arg: *mut c_void) -> *mut c_void {
    let arg = arg as *mut WorkerArgs;
    let num_thread = unsafe { (*arg).num_thread };
    let hold_time = unsafe { (*arg).hold_time };
    let count = unsafe { (*arg).count.clone() };
    barrier(&WAITING1, num_thread);

    let mut n = 0_usize;
    while FALG.load(Ordering::Acquire) == 0 {
        do_lock(hold_time);
        n += 1;
    }

    {
        let mut count = count.lock().unwrap();
        count.push(n);
    }
    barrier(&WAITING2, num_thread);

    ptr::null_mut()
}

extern "C" fn timer(arg: *mut c_void) -> *mut c_void {
    let arg = arg as *mut WorkerArgs;
    let num_thread = unsafe { (*arg).num_thread };
    let count = unsafe { (*arg).count.clone() };
    barrier(&WAITING1, num_thread);

    thread::sleep(Duration::from_secs(30));
    FALG.store(1, Ordering::Release);

    barrier(&WAITING2, num_thread);

    let count = count.lock().unwrap();
    count.iter().enumerate().for_each(|(i, count)| {
        println!("thread {} count: {}", i, count);
    });

    ptr::null_mut()
}

fn main() {
    let command_args = get_command_args();

    let count = Arc::new(Mutex::new(Vec::with_capacity(command_args.num_thread - 1)));

    (0..command_args.num_thread - 1).for_each(|_| {
        let mut th = MaybeUninit::uninit();
        let work_args = WorkerArgs {
            num_thread: command_args.num_thread,
            hold_time: command_args.hold_time,
            count: count.clone(),
        };

        let work_args = &raw const work_args as *mut c_void;
        unsafe {
            pthread_create(th.as_mut_ptr(), ptr::null(), worker, work_args);
            let th = th.assume_init();
            pthread_detach(th);
        }
    });

    let mut th = MaybeUninit::uninit();
    let timer_args = WorkerArgs {
        num_thread: command_args.num_thread,
        hold_time: command_args.hold_time,
        count: count.clone(),
    };
    let timer_args = &raw const timer_args as *mut c_void;
    unsafe {
        pthread_create(th.as_mut_ptr(), ptr::null(), timer, timer_args);
        let th = th.assume_init();
        pthread_join(th, ptr::null_mut());
    }
}
