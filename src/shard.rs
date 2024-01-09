use std::{
    cell::Cell,
    sync::{atomic::AtomicUsize, Arc},
};

use crate::queue::{Receiver, Sender};

#[derive(Debug, thiserror::Error)]
pub enum SenderError {
    #[error("You can't send the value to a shard that doesn't exist.")]
    WrongShard,
}

/// The structure which is used to communicate with other peers from the Mesh.
pub struct Shard<'a, T> {
    pub(crate) receiver: Cell<Option<Receiver<'a, T>>>,
    pub(crate) senders: Vec<Sender<'a, T>>,
    /// Number of shard available
    pub(crate) max_shard: Arc<AtomicUsize>,
    /// Actual shard id
    pub(crate) shard_id: usize,
}

impl<T> Shard<'_, T> {
    /// Take the receiver of this shard.
    /// Shard are implemented using `mpsc` channels, so only one Receiver can receiving values from
    /// the other shards.
    pub fn receiver(&self) -> Option<Receiver<'_, T>> {
        self.receiver.take()
    }

    /// Send a value to the proper shard
    pub fn send_to(&self, val: T, shard: usize) -> Result<(), SenderError> {
        let max_shard = self.max_shard.load(std::sync::atomic::Ordering::Acquire);

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
}
