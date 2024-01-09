use std::{
    cell::Cell,
    sync::{atomic::AtomicUsize, Arc},
};

use crate::queue::{Receiver, Sender};

/// The structure which is used to communicate with other peers from the Mesh.
pub struct Shard<'a, T> {
    pub receiver: Cell<Option<Receiver<'a, T>>>,
    #[allow(dead_code)]
    pub(crate) senders: Vec<Sender<'a, T>>,
    /// Number of shard available
    #[allow(dead_code)]
    pub(crate) max_shard: Arc<AtomicUsize>,
    /// Actual shard id
    #[allow(dead_code)]
    pub(crate) shard_id: usize,
}

impl<T> Shard<'_, T> {
    pub fn receiver(&self) -> Option<Receiver<'_, T>> {
        self.receiver.take()
    }
}
