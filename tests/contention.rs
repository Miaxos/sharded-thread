use std::time::Duration;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        type Driver = monoio::IoUringDriver;
    } else {
        type Driver = monoio::LegacyDriver;
    }
}

#[test]
fn contention() {
    use std::sync::Arc;

    use futures::StreamExt;
    use sharded_thread::mesh::MeshBuilder;
    use sharded_thread::shard::Shard;

    // Message to send
    type Msg = usize;

    let cpus = 4;
    let mesh = Arc::new(MeshBuilder::<Msg>::new(cpus).unwrap());

    let mut handles = Vec::new();
    for peer in 0..cpus {
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
                    shard.send_to_unchecked(peer, send_to);

                    let result = monoio::time::timeout(
                        Duration::from_millis(20),
                        async move {
                            while let Some(val) = receiver.next().await {
                                println!("Received {val} on CPU {peer}");
                                // In this example we break at the begining
                                return;
                            }
                        },
                    )
                    .await;

                    assert!(result.is_ok());
                });
                handle.await
            })
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
