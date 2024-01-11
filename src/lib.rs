//! `sharded_thread` is a module to provides any runtime efficient
//! channels-like abstractions.

/// A mesh to connect multiple executor together.
///
/// A mesh is composed of multiple peers and a way to communicate between each of these peers.
/// Everyone is the mesh can communicate with the others peer.
///
/// # Examples
///
/// A thread-per-core architecture with `monoio`.
///
/// ```rust
/// use sharded_thread::{mesh::MeshBuilder, shard::Shard};
/// use futures::StreamExt;
/// use std::sync::Arc;
///
/// cfg_if::cfg_if! {
///     if #[cfg(target_os = "linux")] {
///         type Driver = monoio::IoUringDriver;
///     } else {
///         type Driver = monoio::LegacyDriver;
///     }
/// }
///
/// // Message to send
/// type Msg = usize;
///
/// let cpus = 4;
/// let mesh = Arc::new(MeshBuilder::<Msg>::new(cpus).unwrap());
///
/// let mut handles = Vec::new();
/// for peer in 0..cpus {
///   let mesh = mesh.clone();
///   let handle = std::thread::spawn(move || {
///       // We lock the thread for the core
///       monoio::utils::bind_to_cpu_set(Some(peer)).unwrap();
///
///       let mut rt = monoio::RuntimeBuilder::<Driver>::new()
///           .with_entries(1024)
///           .enable_timer()
///           .build()
///           .expect("Cannot build runtime");
///
///       let shard: Shard<Msg> = mesh.join_with(peer).unwrap();
///
///       rt.block_on(async move {
///           let handle = monoio::spawn(async move {
///               let mut receiver = shard.receiver().unwrap();
///
///               // We send it unchecked because we are not sure every shard joined when we send
///               // it.
///               // Even if the shard did not join, we can buffer it inside the internal channel.
///               // It's not unsafe, it's unchecked.
///               shard.send_to_unchecked(peer, (peer + 1) % cpus);
///
///               while let Some(val) = receiver.next().await {
///                 println!("Received {val} on CPU {peer}");
///                 // In this example we break at the begining
///                 return ();
///               }
///           });
///           handle.await
///       })
///   });
///
///   handles.push(handle);
/// }
///
/// for handle in handles {
///   handle.join();
/// }
/// ```
pub mod mesh;
pub(crate) mod queue;

/// Sharding utilities built on top of a mesh.
pub mod shard;
