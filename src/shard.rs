use std::{
    cell::Cell,
    sync::{atomic::AtomicUsize, Arc},
};

use crate::queue::{Receiver, Sender};

/// The structure which is used to communicate with other peers from the Mesh.
pub struct Shard<'a, T> {
    pub receiver: Cell<Option<Receiver<'a, T>>>,
    pub(crate) senders: Vec<Sender<'a, T>>,
    /// Number of shard available
    pub(crate) max_shard: Arc<AtomicUsize>,
    /// Actual shard id
    pub(crate) shard_id: usize,
}

impl<T> Shard<'_, T> {
    pub fn receiver(&self) -> Option<Receiver<'_, T>> {
        let a = self.receiver.take();
        a
    }
}
