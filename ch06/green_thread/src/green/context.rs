use std::{
    alloc::{self, Layout},
    ffi::c_void,
    process::exit,
};

use super::{Entry, Registers, PAGE_SIZE};

pub struct Context {
    regs: Registers,
    stack: *mut u8,
    stack_layout: Layout,
    entry: Entry,
    id: u64,
}

impl Context {
    // 레지스터 정보로 포인터 가져오기
    pub fn get_regs_mut(&mut self) -> *mut Registers {
        &mut self.regs
    }

    pub fn get_regs(&self) -> *const Registers {
        &self.regs
    }

    pub fn entry(&self) -> Entry {
        self.entry
    }

    pub fn stack(&self) -> *mut u8 {
        self.stack
    }

    pub fn stack_layout(&self) -> Layout {
        self.stack_layout
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn new(func: Entry, stack_size: usize, id: u64) -> Self {
        let layout = Layout::from_size_align(stack_size, PAGE_SIZE).unwrap();
        let stack = unsafe { alloc::alloc(layout) };

        // 가드 페이지 설정
        unsafe {
            if libc::mprotect(stack as *mut c_void, PAGE_SIZE, libc::PROT_NONE) == -1 {
                libc::perror(c"mprotect error:".as_ptr());
                exit(-1);
            }
        }

        let regs = Registers::new(stack as u64 + stack_size as u64);

        Context {
            regs,
            stack,
            stack_layout: layout,
            entry: func,
            id,
        }
    }
}
