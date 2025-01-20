use std::{
    future::Future,
    pin::Pin,
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    task::{Context, Wake, Waker},
};

struct Task {
    // 실행하는 코루틴
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + Sync>>>,
    // Executor에 스케줄링하기 위한 채널
    sender: SyncSender<Arc<Task>>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let self0 = self.clone();
        self.sender.send(self0).unwrap();
    }
}

pub struct Executor {
    sender: SyncSender<Arc<Task>>,
    receiver: Receiver<Arc<Task>>,
}

impl Executor {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::sync_channel(1024);
        Self { sender, receiver }
    }

    pub fn get_spawner(&self) -> Spawner {
        Spawner {
            sender: self.sender.clone(),
        }
    }

    pub fn run(&self) {
        while let Ok(task) = self.receiver.recv() {
            let mut future = task.future.lock().unwrap();
            let waker = Waker::from(task.clone());
            let mut ctx = Context::from_waker(&waker);
            let _ = future.as_mut().poll(&mut ctx);
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Spawner {
    sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + Send + Sync + 'static) {
        let future = Box::pin(future);
        let task = Arc::new(Task {
            future: Mutex::new(future),
            sender: self.sender.clone(),
        });

        self.sender.send(task).unwrap();
    }
}
