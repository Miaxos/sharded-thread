use std::io::Cursor;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        type Driver = monoio::IoUringDriver;
    } else {
        type Driver = monoio::LegacyDriver;
    }
}

// For this test we need 3 core:
//
// 3: Bencher
// 1-2: Passthrough
//
// We send a value from 3 -> 1, then 1 -> 2, then 2 -> 3.

fn contention() {
    use std::sync::Arc;

    use futures::StreamExt;
    use sharded_thread::mesh::MeshBuilder;
    use sharded_thread::shard::Shard;

    // Message to send
    type Msg = usize;

    let cpus = 3;
    let mesh = Arc::new(MeshBuilder::<Msg>::new(cpus).unwrap());

    let mut handles: Vec<std::thread::JoinHandle<()>> = Vec::new();
    for peer in 0..2 {
        let mesh = mesh.clone();
        let handle = std::thread::spawn(move || {
            // We lock the thread for the core
            monoio::utils::bind_to_cpu_set(Some(peer)).unwrap();

            let mut rt = monoio::RuntimeBuilder::<Driver>::new()
                .with_entries(1024)
                .enable_timer()
                .build()
                .expect("Cannot build runtime");

            let shard: Shard<Msg> = mesh.join_with(peer).unwrap();

            rt.block_on(async move {
                let handle = monoio::spawn(async move {
                    let mut receiver = shard.receiver().unwrap();

                    let send_to = (peer + 1) % cpus;

                    while let Some(val) = receiver.next().await {
                        shard.send_to_unchecked(val, send_to);
                    }
                });
                handle.await
            })
        });

        handles.push(handle);
    }

    (handles, mesh)
}

fn parse(c: &mut Criterion) {
    let ping_test = b"*1\r\n$4\r\nPING\r\n";
}
