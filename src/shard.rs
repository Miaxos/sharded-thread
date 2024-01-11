use std::cell::Cell;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crate::queue::{Receiver, Sender};

#[derive(Debug, thiserror::Error)]
pub enum SenderError {
    #[error("You can't send the value to a shard that doesn't exist.")]
    WrongShard,
}

/// The structure which is used to communicate with other peers from the Mesh.
pub struct Shard<T> {
    pub(crate) receiver: Cell<Option<Receiver<T>>>,
    pub(crate) senders: Vec<Sender<T>>,
    /// Number of shard available
    pub(crate) max_shard: Arc<AtomicUsize>,
    /// Actual shard id
    #[allow(dead_code)]
    pub(crate) shard_id: usize,
}

impl<T> Shard<T> {
    /// Take the receiver of this shard.
    /// Shard are implemented using `mpsc` channels, so only one Receiver can
    /// receiving values from the other shards.
    pub fn receiver(&self) -> Option<Receiver<T>> {
        self.receiver.take()
    }

    /// Send a value to the proper shard
    ///
    /// Fail if this Shard did not join yet.
    pub fn send_to(&self, val: T, shard: usize) -> Result<(), SenderError> {
        let max_shard =
            self.max_shard.load(std::sync::atomic::Ordering::Acquire);

        if shard >= max_shard {
            return Err(SenderError::WrongShard);
        }

        let sender = self
            .senders
            .get(shard)
            .expect("the sender should have been here but he is not.");

        sender.send(val);
        Ok(())
    }

    /// Send a value to a shard
    pub fn send_to_unchecked(&self, val: T, shard: usize) -> () {
        let sender = self
            .senders
            .get(shard)
            .expect("the sender should have been here but he is not.");

        sender.send(val);
    }
}
