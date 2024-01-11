use std::sync::Arc;
use std::thread::JoinHandle;
use std::u128;

use criterion::{criterion_group, criterion_main, Criterion};
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
fn start_threads<Msg: Send + 'static>(
) -> (Vec<JoinHandle<()>>, Arc<MeshBuilder<Msg>>) {
    use std::sync::Arc;

    use futures::StreamExt;
    use sharded_thread::mesh::MeshBuilder;
    use sharded_thread::shard::Shard;

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

fn execute_round<T: Send + 'static>(
    mesh: Arc<MeshBuilder<T>>,
    count_max: usize,
) -> JoinHandle<()> {
    
    std::thread::spawn(move || {
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
    })
}

fn bench_round(c: &mut Criterion) {
    let (_, mesh) = start_threads::<usize>();

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

struct WrapperSendStruct {
    val: usize,
    thing: String,
    foo0: u128,
    foo1: u128,
    foo2: u128,
    thing2: String,
}

impl Default for WrapperSendStruct {
    fn default() -> Self {
        Self {
            val: 12345,
            thing: "Something awesome is here, do you want to know about \
                    this? no? Are you really sure about it?"
                .to_string(),
            thing2: "SOMETHING AWESOME IS HERE, DO YOU WANT TO KNOW ABOUT \
                     THIS? NO? ARE YOU REALLY SURE ABOUT IT?"
                .to_string(),
            foo0: 123456786543245643,
            foo1: 523456786543245643,
            foo2: 323356786543245643,
        }
    }
}

// TODO: MAcro to generate benches
fn bench_round_struct(c: &mut Criterion) {
    let (_, mesh) = start_threads::<WrapperSendStruct>();

    c.bench_function("struct_send_a_value_between_two_cpu_1", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 0)),
            |(mesh, handle)| {
                mesh.send_to(1, WrapperSendStruct::default()).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });

    c.bench_function("struct_rotate_a_usize_between_3_cpu_1", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 1)),
            |(mesh, handle)| {
                mesh.send_to(1, WrapperSendStruct::default()).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });

    c.bench_function("struct_rotate_a_usize_between_3_cpu_10", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 10)),
            |(mesh, handle)| {
                mesh.send_to(1, WrapperSendStruct::default()).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });

    c.bench_function("struct_rotate_a_usize_between_3_cpu_100", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 100)),
            |(mesh, handle)| {
                mesh.send_to(1, WrapperSendStruct::default()).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });

    c.bench_function("struct_rotate_a_usize_between_3_cpu_1000", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 1_000)),
            |(mesh, handle)| {
                mesh.send_to(1, WrapperSendStruct::default()).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });

    c.bench_function("struct_rotate_a_usize_between_3_cpu_10_000", |b| {
        b.iter_batched(
            || (Arc::clone(&mesh), execute_round(Arc::clone(&mesh), 1_000)),
            |(mesh, handle)| {
                mesh.send_to(1, WrapperSendStruct::default()).unwrap();
                handle.join().unwrap();
            },
            criterion::BatchSize::PerIteration,
        )
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_round, bench_round_struct
}
criterion_main!(benches);
