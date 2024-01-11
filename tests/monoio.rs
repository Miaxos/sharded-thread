use std::os::fd::{FromRawFd, IntoRawFd, RawFd};
use std::sync::Arc;
use std::thread::scope;
use std::time::Duration;

use futures::StreamExt;
use monoio::io::{AsyncReadRentExt, AsyncWriteRent};
use monoio::net::{TcpListener, TcpStream};
use sharded_thread::mesh::MeshBuilder;
use sharded_thread::shard::Shard;

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

    let mesh = Arc::new(MeshBuilder::<Msg>::new(1).unwrap());

    let cpus: usize = 0;

    let mut handles = Vec::new();
    for cpu in 0..cpus {
        let mesh = mesh.clone();
        let handle = std::thread::spawn(move || {
            monoio::utils::bind_to_cpu_set(Some(cpu)).unwrap();

            let mut rt = monoio::RuntimeBuilder::<Driver>::new()
                .with_entries(1024)
                .enable_timer()
                .build()
                .expect("Cannot build runtime");

            let shard: Shard<Msg> = mesh.join_with(cpu).unwrap();

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

    let pos = mesh.members();
    assert_eq!(pos, cpus);

    mesh.send_to(0, 12).unwrap();
    mesh.send_to(0, 1).unwrap();
}

#[test]
fn load_balance_tcp() {
    type Msg = RawFd;

    let mesh = MeshBuilder::<Msg>::new(3).unwrap();
    scope(|scope| {
        let cpus: usize = 3;
        let mesh = &mesh;

        let addr = "127.0.0.1:12345";

        let mut handles = Vec::new();
        // We will run monoio on 3 separate cpu & 3 separate thread.
        // - One Tcp server which will accept the connection and send the fd to
        //   the proper thread
        // - One tcp client which will connect with the tcp server
        // - One which will receive the fd and respond to the client and close
        //   the connection.
        for cpu in 0..cpus {
            let handle = scope.spawn(move || {
                monoio::utils::bind_to_cpu_set(Some(cpu)).unwrap();
                let mut rt = monoio::RuntimeBuilder::<Driver>::new()
                    .with_entries(1024)
                    .enable_timer()
                    .build()
                    .expect("Cannot build runtime");

                let shard: Shard<Msg> = mesh.join_with(cpu).unwrap();

                if cpu == 2 {
                    // - One tcp client which will connect with the tcp server
                    rt.block_on(async move {
                        let result = monoio::time::timeout(
                            Duration::from_secs(3),
                            async move {
                                loop {
                                    if let Ok(mut client_stream) =
                                        TcpStream::connect(addr).await
                                    {
                                        // client_stream.set_nodelay(true).
                                        // unwrap();
                                        let buf = Box::new([0u8; 9]);
                                        let (_, buf) =
                                            client_stream.read_exact(buf).await;

                                        assert_eq!(
                                            buf.as_slice(),
                                            b"hello mom"
                                        );
                                        break;
                                    }
                                }
                            },
                        )
                        .await;

                        assert!(result.is_ok());
                    });
                } else {
                    rt.block_on(async move {
                        let handle = monoio::spawn(async move {
                            if cpu == 0 {
                                // - One Tcp server which will accept the
                                //   connection and send the fd to the proper
                                //   thread
                                let handle = monoio::spawn(async move {
                                    let srv = TcpListener::bind(addr).unwrap();
                                    let (server_stream, _) =
                                        srv.accept().await.unwrap();
                                    let fd = server_stream.into_raw_fd();

                                    // We send the fd to the other thread on the
                                    // other CPU.
                                    shard.send_to(fd, 1).unwrap();
                                });

                                monoio::time::timeout(
                                    Duration::from_millis(3000),
                                    handle,
                                )
                                .await
                                .unwrap();
                            } else {
                                // - One which will receive the fd and respond
                                //   to the client and close the connection.
                                // cpu = 1
                                //
                                let receiver = shard.receiver();
                                let mut receiver = receiver.unwrap();
                                let fd = receiver.next().await.unwrap();

                                let mut tcp = TcpStream::from_std(unsafe {
                                    std::net::TcpStream::from_raw_fd(fd)
                                })
                                .unwrap();

                                tcp.write(b"hello mom").await.0.unwrap();
                                tcp.flush().await.unwrap();
                            }
                        });
                        handle.await
                    })
                }
            });

            handles.push(handle);
        }

        for i in handles {
            let r = i.join();
            assert!(r.is_ok());
        }
    });
}
