use futures::task::AtomicWaker;
use futures::Stream;
use sharded_queue::ShardedQueue;
use std::sync::atomic::AtomicUsize;

use std::task::Poll;

/// A queue that should be available on each thread.
pub struct SharedQueueThreaded<T> {
    queue: ShardedQueue<T>,
    task_queue: AtomicUsize,
    waker: AtomicWaker,
}

impl<T> SharedQueueThreaded<T> {
    pub fn new(max_concurrent_thread_count: usize) -> std::io::Result<Self> {
        let waker = AtomicWaker::new();
        Ok(Self {
            queue: ShardedQueue::new(max_concurrent_thread_count),
            task_queue: AtomicUsize::new(0),
            waker,
        })
    }
}

pub trait SharedQueueChannels<'a, T> {
    fn unbounded(&'a self) -> (Sender<'a, T>, Receiver<'a, T>);

    fn sender(&'a self) -> Sender<'a, T>;
}

impl<'a, T> SharedQueueChannels<'a, T> for SharedQueueThreaded<T> {
    fn unbounded(&'a self) -> (Sender<'a, T>, Receiver<'a, T>) {
        let tx = self.sender();

        let rx = Receiver { queue: self };

        (tx, rx)
    }

    fn sender(&'a self) -> Sender<'a, T> {
        Sender { queue: self }
    }
}

pub struct Sender<'a, T> {
    queue: &'a SharedQueueThreaded<T>,
}

impl<'a, T> Sender<'a, T> {
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
pub struct Receiver<'a, T> {
    queue: &'a SharedQueueThreaded<T>,
}

impl<'a, T> Stream for Receiver<'a, T> {
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

    use super::SharedQueueChannels;
    use super::SharedQueueThreaded;
    use futures::StreamExt;

    #[monoio::test_all(timer_enabled = true)]
    async fn ensure_send_receive() {
        let queue = SharedQueueThreaded::<u8>::new(2).unwrap();

        let (tx, mut rx) = queue.unbounded();

        tx.send(1);
        tx.send(2);

        let val1 = rx.next().await.unwrap();
        let val2 = rx.next().await.unwrap();
        let val3 = monoio::time::timeout(Duration::from_millis(10), rx.next()).await;

        let mut merged = [val1, val2];
        merged.sort();
        let merged: Vec<u8> = merged.into_iter().collect();
        assert_eq!(merged, [1, 2]);
        assert!(val3.is_err());
    }
}
