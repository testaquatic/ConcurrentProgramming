mod context;
mod mapped_list;

use core::panic;
use std::{
    alloc::{dealloc, Layout},
    collections::{HashMap, HashSet, LinkedList},
    ffi::c_void,
    process::exit,
    ptr,
};

use context::Context;
use mapped_list::MappedList;

extern "C" {
    fn set_context(ctx: *mut Registers) -> u64;
    fn switch_context(ctx: *const Registers) -> !;
}

#[repr(C)]
struct Registers {
    rbx: u64,
    rbp: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rsp: u64,
    rdx: u64,
}

impl Registers {
    fn new(rsp: u64) -> Self {
        Registers {
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rsp,
            #[allow(clippy::fn_to_numeric_cast)]
            rdx: entry_point as u64,
        }
    }
}

// 스레드 개시 시 실행하는 함수 타입
type Entry = fn();

// 페이지 크기, 리눅스에서는 4KB
pub const PAGE_SIZE: usize = 4 * 1024;

// 모든 스레드 종료 시 돌아올 위치
static mut CTX_MAIN: Option<Box<Registers>> = None;

// 불필요한 스택 영역
static mut UNUSED_STACK: (*mut u8, Layout) = (ptr::null_mut(), Layout::new::<u8>());

// 스레드 실행 큐
static mut CONTEXTS: LinkedList<Box<Context>> = LinkedList::new();

// 스레드 ID 집합
static mut ID: *mut HashSet<u64> = ptr::null_mut();

static mut MESSAGES: *mut MappedList<u64> = ptr::null_mut();

static mut WAITING: *mut HashMap<u64, Box<Context>> = ptr::null_mut();

fn get_id() -> u64 {
    loop {
        let rnd = rand::random::<u64>();
        unsafe {
            if !(*ID).contains(&rnd) {
                (*ID).insert(rnd);
                return rnd;
            }
        }
    }
}

pub fn spawn(func: Entry, stack_size: usize) -> u64 {
    let id = get_id();
    unsafe {
        #[allow(static_mut_refs)]
        CONTEXTS.push_back(Box::new(Context::new(func, stack_size, id)));
        schedule();
    }
    id
}

#[allow(static_mut_refs)]
pub fn schedule() {
    unsafe {
        if CONTEXTS.len() == 1 {
            return;
        }

        let mut ctx = CONTEXTS.pop_front().unwrap();
        let regs = ctx.get_regs_mut();
        CONTEXTS.push_back(ctx);

        if set_context(regs) == 0 {
            // 다음 스레드로 컨텍스트 스위칭
            let next = CONTEXTS.front().unwrap();
            switch_context((**next).get_regs());
        }

        rm_unused_stack();
    }
}

#[allow(static_mut_refs)]
extern "C" fn entry_point() {
    unsafe {
        let ctx = CONTEXTS.front().unwrap();
        ctx.entry()();

        let ctx = CONTEXTS.pop_front().unwrap();

        (*ID).remove(&ctx.id());

        UNUSED_STACK = ((*ctx).stack(), (*ctx).stack_layout());

        match CONTEXTS.front() {
            Some(c) => switch_context((**c).get_regs()),
            None => {
                if let Some(c) = &CTX_MAIN {
                    switch_context(&**c as *const Registers);
                }
            }
        }
    }
    panic!("entry_point");
}

#[allow(static_mut_refs)]
pub fn spawn_from_main(func: Entry, stack_size: usize) {
    unsafe {
        if CTX_MAIN.is_some() {
            panic!("spawn_from_main is called twice");
        }

        CTX_MAIN = Some(Box::new(Registers::new(0)));
        if let Some(ctx) = &mut CTX_MAIN {
            let mut msgs = MappedList::new();
            MESSAGES = &raw mut msgs;

            let mut waiting = HashMap::new();
            WAITING = &raw mut waiting;

            let mut ids = HashSet::new();
            ID = &raw mut ids;

            if set_context(&mut **ctx) == 0 {
                CONTEXTS.push_back(Box::new(Context::new(func, stack_size, get_id())));
                let first = CONTEXTS.front().unwrap();
                switch_context(first.get_regs());
            }

            rm_unused_stack();

            CTX_MAIN = None;
            CONTEXTS.clear();
            MESSAGES = ptr::null_mut();
            WAITING = ptr::null_mut();
            ID = ptr::null_mut();

            msgs.clear();
            waiting.clear();
            ids.clear();
        }
    }
}

unsafe fn rm_unused_stack() {
    if !UNUSED_STACK.0.is_null() {
        if libc::mprotect(
            UNUSED_STACK.0 as *mut c_void,
            PAGE_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
        ) == -1
        {
            libc::perror(c"mprotect error".as_ptr());
            exit(-1);
        }
        dealloc(UNUSED_STACK.0, UNUSED_STACK.1);
        UNUSED_STACK = (ptr::null_mut(), Layout::new::<u8>());
    }
}

pub fn send(key: u64, msg: u64) {
    unsafe {
        (*MESSAGES).push_back(key, msg);

        if let Some(ctx) = (*WAITING).remove(&key) {
            #[allow(static_mut_refs)]
            CONTEXTS.push_back(ctx);
        }
    }

    schedule();
}

#[allow(static_mut_refs)]
pub fn recv() -> Option<u64> {
    unsafe {
        let key = CONTEXTS.front().unwrap().id();

        if let Some(msg) = (*MESSAGES).pop_front(key) {
            return Some(msg);
        }

        #[allow(static_mut_refs)]
        if CONTEXTS.len() == 1 {
            panic!("deadlock");
        }

        let mut ctx = CONTEXTS.pop_front().unwrap();
        let regs = ctx.get_regs_mut();
        (*WAITING).insert(key, ctx);

        if set_context(regs) == 0 {
            let next = CONTEXTS.front().unwrap();
            switch_context((**next).get_regs());
        }

        rm_unused_stack();

        (*MESSAGES).pop_front(key)
    }
}

pub fn producer() {
    let id = spawn(consumer, 2 * 1024 * 1024);
    (0..10).for_each(|i| {
        send(id, i);
    });
}

pub fn consumer() {
    (0..10).for_each(|_| {
        let msg = recv().unwrap();
        println!("received: count = {}", msg);
    });
}
