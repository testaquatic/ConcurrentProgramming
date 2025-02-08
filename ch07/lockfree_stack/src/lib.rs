use std::{
    ptr,
    sync::atomic::{self, AtomicPtr},
};

// 스택의 노드
struct Node<T> {
    next: AtomicPtr<Node<T>>,
    data: T,
}

pub struct StackBad<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> StackBad<T> {
    pub fn new() -> Self {
        StackBad {
            head: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    pub fn push(&self, data: T) {
        let new_node = Box::new(Node {
            next: AtomicPtr::new(ptr::null_mut()),
            data,
        });

        let ptr = Box::into_raw(new_node);

        unsafe {
            loop {
                // head값 취득
                let head = self.head.load(atomic::Ordering::Relaxed);

                (*ptr).next.store(head, atomic::Ordering::Relaxed);

                if self
                    .head
                    .compare_exchange_weak(
                        head,
                        ptr,
                        atomic::Ordering::Release,
                        atomic::Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    break;
                }
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        unsafe {
            loop {
                let head = self.head.load(atomic::Ordering::Relaxed);

                if head.is_null() {
                    return None;
                }

                let next = (*head).next.load(atomic::Ordering::Relaxed);

                if self
                    .head
                    .compare_exchange_weak(
                        head,
                        next,
                        atomic::Ordering::Acquire,
                        atomic::Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    let h = Box::from_raw(head);
                    return Some((*h).data);
                }
            }
        }
    }
}

impl<T> Drop for StackBad<T> {
    fn drop(&mut self) {
        let mut node = self.head.load(atomic::Ordering::Relaxed);
        while !node.is_null() {
            let n = unsafe { Box::from_raw(node) };
            node = n.next.load(atomic::Ordering::Relaxed);
        }
    }
}
