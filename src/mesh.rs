//! The idea of the mesh is to be able to connected threads together through channels.

use std::{
    cell::Cell,
    sync::{atomic::AtomicUsize, Arc},
};

use crate::{
    queue::{SharedQueueChannels, SharedQueueThreaded},
    shard::{SenderError, Shard},
};

/// A Mesh is a structure which can be shared in every thread by reference to allow threads to join
/// the Mesh and talk to each others.
pub struct MeshBuilder<T> {
    #[allow(dead_code)]
    nr_peers: usize,
    pub(crate) channels: Vec<Arc<SharedQueueThreaded<T>>>,
    pub(crate) shared_joined: Arc<AtomicUsize>,
}

impl<T> MeshBuilder<T> {
    /// Create a new mesh between a number of peers.
    pub fn new(nr_peers: usize) -> std::io::Result<Self> {
        let nb_cpu = std::thread::available_parallelism()?.get();

        MeshBuilder::with_cpu(nr_peers, nb_cpu)
    }

    ///
    pub fn members(&self) -> usize {
        self.shared_joined
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn with_cpu(nr_peers: usize, nb_cpu: usize) -> std::io::Result<Self> {
        let mut channels = Vec::with_capacity(nr_peers);

        for _i in 0..nr_peers {
            channels.push(SharedQueueThreaded::<T>::new(nb_cpu)?);
        }

        Ok(Self {
            nr_peers,
            channels,
            shared_joined: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Try to send an item directly to a shard, you must know the id of the shard you want to send
    /// the item to.
    ///
    /// Fail if the shard is not registered.
    #[doc(hidden)]
    pub fn send_to(&self, pos: usize, item: T) -> Result<(), SenderError> {
        self.channels
            .get(pos)
            .ok_or(SenderError::WrongShard)?
            .sender()
            .send(item);
        Ok(())
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
