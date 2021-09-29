use std::sync::{Arc, Mutex};
use std::task::Context;
use std::{
    future::Future,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
};

use futures::future::BoxFuture;
use futures::task::{waker_ref, ArcWake};
use futures::FutureExt;

pub struct Executor {
    receiver: Receiver<Arc<Task>>,
}

impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.receiver.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let cx = &mut Context::from_waker(&waker);
                if future.as_mut().poll(cx).is_pending() {
                    *future_slot = Some(future);
                }
            }
        }
    }
}

pub struct Spawner {
    sync_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            sync_sender: self.sync_sender.clone(),
        });
        self.sync_sender.send(task).expect("too many tasks queued");
    }
}

pub struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    sync_sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &std::sync::Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self
            .sync_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (sync_sender, receiver) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { receiver }, Spawner { sync_sender })
}
