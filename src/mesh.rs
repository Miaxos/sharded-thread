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
    pub channels: Vec<SharedQueueThreaded<T>>,
    pub position: Arc<AtomicUsize>,
}

impl<'a, T> MeshBuilder<T> {
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
            position: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Join the mesh means you can talk to other peer and peer can talk to you.
    pub fn join(&'a self) -> std::io::Result<Shard<'a, T>> {
        let pos = self
            .position
            .fetch_add(1, std::sync::atomic::Ordering::Acquire);

        assert!(pos < self.channels.len());

        let senders = self
            .channels
            .iter()
            .map(SharedQueueThreaded::sender)
            .collect();
        let (_, receiver) = self.channels[pos].unbounded();

        Ok(Shard {
            receiver: Cell::new(Some(receiver)),
            senders,
            max_shard: self.position.clone(),
            shard_id: pos,
        })
    }
}
