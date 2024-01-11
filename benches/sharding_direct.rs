use std::sync::Arc;
use std::thread::JoinHandle;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use futures::StreamExt;
use sharded_thread::mesh::MeshBuilder;

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

fn start_threads() -> (Vec<JoinHandle<()>>, Arc<MeshBuilder<usize>>) {
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

fn execute_round(
    mesh: Arc<MeshBuilder<usize>>,
    count_max: usize,
) -> JoinHandle<()> {
    let handle = std::thread::spawn(move || {
        // We lock the thread for the core
        monoio::utils::bind_to_cpu_set(Some(2)).unwrap();
        let mut rt = monoio::RuntimeBuilder::<Driver>::new()
            .with_entries(1024)
            .enable_timer()
            .build()
            .expect("Cannot build runtime");

        let shard = mesh.join_with(2).unwrap();
        // shard.send_to_unchecked(12345, 2);

        rt.block_on(async move {
            let handle = monoio::spawn(async move {
                let mut receiver = shard.receiver().unwrap();

                let mut count = 0;
                while let Some(val) = receiver.next().await {
                    if count > count_max {
                        return;
                    }
                    shard.send_to_unchecked(val, 0);
                    count += 1;
                }
            });
            handle.await
        })
    });
    handle
}

fn bench_round(c: &mut Criterion) {
    let (_, mesh) = start_threads();

    c.bench_function("send_a_value_between_two_cpu_1", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 0)),
            |(mesh, handle)| {
                mesh.send_to(1, 12345).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });

    c.bench_function("rotate_a_usize_between_3_cpu_1", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 1)),
            |(mesh, handle)| {
                mesh.send_to(2, 12345).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });

    c.bench_function("rotate_a_usize_between_3_cpu_10", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 10)),
            |(mesh, handle)| {
                mesh.send_to(2, 12345).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });

    c.bench_function("rotate_a_usize_between_3_cpu_100", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 100)),
            |(mesh, handle)| {
                mesh.send_to(2, 12345).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });

    c.bench_function("rotate_a_usize_between_3_cpu_1000", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 1_000)),
            |(mesh, handle)| {
                mesh.send_to(2, 12345).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });

    c.bench_function("rotate_a_usize_between_3_cpu_10_000", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 1_000)),
            |(mesh, handle)| {
                mesh.send_to(2, 12345).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_round
}
criterion_main!(benches);
