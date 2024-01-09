use std::thread::scope;

use futures::StreamExt;

use sharded_thread::queue::SharedQueueChannels;
use sharded_thread::{mesh::MeshBuilder, shard::Shard};

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        type Driver = monoio::IoUringDriver;
    } else {
        type Driver = monoio::LegacyDriver;
    }
}

#[test]
fn ensure_messages_are_sent_through_the_shard() {
    type Msg = i32;

    let mesh = MeshBuilder::<Msg>::new().unwrap();
    scope(|scope| {
        let cpus: usize = 0;
        let mesh = &mesh;

        let mut handles = Vec::new();
        for cpu in 0..cpus {
            let handle = scope.spawn(move || {
                monoio::utils::bind_to_cpu_set(Some(cpu)).unwrap();
                let mut rt = monoio::RuntimeBuilder::<Driver>::new()
                    .with_entries(1024)
                    .enable_timer()
                    .build()
                    .expect("Cannot build runtime");

                // TODO: maybe change it to a Arc instead and we'll have to wait for scoped
                // lifetime for async
                //
                // It's not unsafe because we know Shard is going to live more than the thread as
                // everything is scoped.
                let shard: Shard<'static, Msg> = unsafe {
                    std::mem::transmute::<Shard<'_, Msg>, Shard<'static, Msg>>(mesh.join().unwrap())
                };

                rt.block_on(async move {
                    let handle = monoio::spawn(async move {
                        let r = shard.receiver();
                        let mut r = r.unwrap();

                        let val = r.next().await.unwrap();
                        assert_eq!(val, 12);
                        let val = r.next().await.unwrap();
                        assert_eq!(val, 1);
                    });
                    handle.await
                })
            });

            handles.push(handle);
        }

        for i in handles {
            let r = i.join();
            assert!(r.is_ok());
        }

        let pos = mesh.position.load(std::sync::atomic::Ordering::Acquire);
        assert_eq!(pos, cpus);

        let chan = &mesh.channels[0].sender();
        chan.send(12);
        chan.send(1);
    });
}
