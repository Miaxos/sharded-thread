//! The idea of the mesh is to be able to connected threads together through channels.

use std::{
    cell::Cell,
    sync::{atomic::AtomicUsize, Arc},
};

use crate::{
    queue::{SharedQueueChannels, SharedQueueThreaded},
    shard::Shard,
};

/// A Mesh is a structure which can be shared in every thread by reference to allow threads to join
/// the Mesh and talk to each others.
pub struct MeshBuilder<T> {
    #[allow(dead_code)]
    nr_peers: usize,
    pub channels: Vec<Arc<SharedQueueThreaded<T>>>,
    pub shared_joined: Arc<AtomicUsize>,
}

impl<T> MeshBuilder<T> {
    /// Create a new mesh
    pub fn new() -> std::io::Result<Self> {
        let nr_peers = std::thread::available_parallelism()?.get();

        let mut channels = Vec::with_capacity(nr_peers);

        for _i in 0..nr_peers {
            channels.push(SharedQueueThreaded::<T>::new(nr_peers)?);
        }

        Ok(Self {
            nr_peers,
            channels,
            shared_joined: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Join the mesh means you can talk to other peer and peer can talk to you.
    ///
    /// You must assign yourself an id so other Shard will be able to talk with you using this ID
    pub fn join_with(&self, peer: usize) -> std::io::Result<Shard<T>> {
        self.shared_joined
            .fetch_add(1, std::sync::atomic::Ordering::Acquire);

        assert!(peer < self.channels.len());

        let senders = self
            .channels
            .iter()
            .map(SharedQueueChannels::sender)
            .collect();
        let (_, receiver) = self.channels[peer].unbounded();

        Ok(Shard {
            receiver: Cell::new(Some(receiver)),
            senders,
            max_shard: self.shared_joined.clone(),
            shard_id: peer,
        })
    }
}
