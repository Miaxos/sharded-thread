use std::sync::atomic::{fence, AtomicUsize};
use std::sync::Arc;
use std::task::Poll;

use futures::task::AtomicWaker;
use futures::Stream;
use sharded_queue::ShardedQueue;

/// A queue that should be available on each thread.
pub struct SharedQueueThreaded<T> {
    queue: ShardedQueue<T>,
    task_queue: AtomicUsize,
    waker: AtomicWaker,
}

impl<T> SharedQueueThreaded<T> {
    /// Create a new `SharedQueueThreaded` by specifing the number of thread how
    /// can physically access the queue = the number of CPU core available
    /// for the application.
    pub fn new(
        max_concurrent_thread_count: usize,
    ) -> std::io::Result<Arc<Self>> {
        let waker = AtomicWaker::new();
        Ok(Arc::new(Self {
            queue: ShardedQueue::new(max_concurrent_thread_count),
            task_queue: AtomicUsize::new(0),
            waker,
        }))
    }
}

pub trait SharedQueueChannels<T> {
    fn unbounded(&self) -> (Sender<T>, Receiver<T>);

    fn sender(&self) -> Sender<T>;
}

impl<T> SharedQueueChannels<T> for Arc<SharedQueueThreaded<T>> {
    fn unbounded(&self) -> (Sender<T>, Receiver<T>) {
        let tx = self.sender();

        let rx = Receiver {
            queue: Arc::clone(self),
        };

        (tx, rx)
    }

    fn sender(&self) -> Sender<T> {
        Sender {
            queue: Arc::clone(self),
        }
    }
}

pub struct Sender<T> {
    queue: Arc<SharedQueueThreaded<T>>,
}

impl<T> Sender<T> {
    /// Attempts to send a value to the queue
    pub fn send(&self, item: T) {
        self.queue
            .task_queue
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.queue.queue.push_back(item);
        self.queue.waker.wake();
    }
}

#[derive(Clone)]
pub struct Receiver<T> {
    queue: Arc<SharedQueueThreaded<T>>,
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.queue.waker.register(cx.waker());

        let old = self
            .queue
            .task_queue
            .load(std::sync::atomic::Ordering::Relaxed);

        if old > 0 {
            self.queue
                .task_queue
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            let item = self.queue.queue.pop_front_or_spin_wait_item();
            Poll::Ready(Some(item))
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use futures::StreamExt;

    use super::{SharedQueueChannels, SharedQueueThreaded};

    #[monoio::test_all(timer_enabled = true)]
    async fn ensure_send_receive() {
        let queue = SharedQueueThreaded::<u8>::new(2).unwrap();

        let (tx, mut rx) = queue.unbounded();

        tx.send(1);
        tx.send(2);

        let val1 = rx.next().await.unwrap();
        let val2 = rx.next().await.unwrap();
        let val3 =
            monoio::time::timeout(Duration::from_millis(10), rx.next()).await;

        let mut merged = [val1, val2];
        merged.sort();
        let merged: Vec<u8> = merged.into_iter().collect();
        assert_eq!(merged, [1, 2]);
        assert!(val3.is_err());
    }
}
