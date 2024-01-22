# sharded-thread

[![release](https://github.com/Miaxos/json-predicate/actions/workflows/release.yml/badge.svg)](https://github.com/Miaxos/sharded-thread/actions/workflows/release.yml)
[![Crates.io version](https://img.shields.io/crates/v/sharded-thread.svg)](https://crates.io/crates/sharded-thread)
[![dependency status](https://deps.rs/repo/github/miaxos/sharded-thread/status.svg)](https://deps.rs/repo/github/miaxos/sharded-thread)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/sharded-thread)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/miaxos/sharded-thread/compare)


"*Application tail latency is critical for services to meet their latency 
expectations. We have shown that the thread-per-core approach can reduce 
application tail latency of a key-value store by up to 71% compared to baseline 
Memcached running on commodity hardware and Linux.*"[^1]

[^1]: [The Impact of Thread-Per-Core Architecture on Application Tail Latency](https://helda.helsinki.fi/server/api/core/bitstreams/3142abaa-16e3-4ad0-beee-e62add589fc4/content)

## Introduction

This library is mainly made for `io-uring` and monoio. There are no dependency
on the runtime, so you should be able to use it with other runtime and also
without `io-uring`.

The purpose of this library is to have a performant way to send data between
thread when threads are following a `thread per core` architecture. Even if the
aim is to be performant remember it's a core to core passing, (or thread to
thread), which is really slow.

Thanks to [Glommio](https://github.com/DataDog/glommio/) for the inspiration.

## Example

Originally, the library was made when you had multiple thread listening to the
same `TcpStream` and depending on what is sent through the `TcpStream` you might
want to change the thread handling the stream.

<p align="center">
    <img src="./.github/ressources/sharded-thread.drawio.svg" width="60%" />
</p>

You can check some examples in the tests.

## Benchmarks

Those benchmarks are only indicative, they are running in GA. You should run
your own on the targeted hardware.

It shows that `sharded-thread` based on `utility.sharded_queue` is faster (~6%) than
if we built the mesh based on `flume`.


<p align="center">
<a href="https://bencher.dev/perf/sharded-thread?key=true&reports_per_page=4&branches_per_page=8&testbeds_per_page=8&benchmarks_per_page=8&reports_page=1&branches_page=1&testbeds_page=1&benchmarks_page=1&branches=abb3e39f-486d-45af-924f-3614316b6781&testbeds=dc83c4f4-db68-476e-8e8c-a0c787f60fdb&benchmarks=a56fc2d7-bb4f-49cb-8bfd-16223b326004%2Cdca64e65-40d9-484b-b0f7-e341c1adaff6%2C30af2c37-5269-4a84-a6bb-204c51da2ed5%2Ceed7d9fa-e955-4ccd-acee-80534dc4403c%2C960718ec-010a-4ff5-b74f-ab593e17b4fd%2C8a0b884d-740a-440e-990c-507d73eca2e1%2C3b79c28d-f1ab-4926-be75-e82433e2c842%2C4367f225-65b0-4b94-a70d-e51e45a208e0%2Cb4367643-de00-4e15-b3cd-cf47da3be762%2C3aace4e2-7658-4f8e-92b7-c701d4f5204f%2C5a2dbf32-b7c5-41f9-8e8c-a1bb90f338ce%2Cb358b1e8-eef2-4c3e-b967-36b69880bcfe%2C72919b2c-d10e-44e4-9435-fff39cd74755%2C7068ef9b-69b7-4ddb-bb1e-30094d412534%2C6ac4c447-a9a6-40ef-aae1-d24ca5311e10%2C70fceb0c-ef3c-48b1-bcfe-2147aeca8968%2Cea2f2c04-4a69-4d52-a506-cee48b974851&measures=0e7c206f-e0e3-4672-be1a-b3820dc2789a&start_time=1703312681000&end_time=1705909309000&clear=true&tab=benchmarks"><img src="https://api.bencher.dev/v0/projects/sharded-thread/perf/img?branches=abb3e39f-486d-45af-924f-3614316b6781&testbeds=dc83c4f4-db68-476e-8e8c-a0c787f60fdb&benchmarks=3aace4e2-7658-4f8e-92b7-c701d4f5204f%2Ceed7d9fa-e955-4ccd-acee-80534dc4403c&measures=0e7c206f-e0e3-4672-be1a-b3820dc2789a&start_time=1703312681000&end_time=1705909309000&title=Flume+vs+Sharded-thread" title="Flume vs Sharded-thread" alt="Flume vs Sharded-thread for sharded-thread - Bencher" width="80%" /></a>
</p>

## References

- [Glommio example on their sharding](https://github.com/DataDog/glommio/blob/master/examples/sharding.rs)
- [The original monoio issue](https://github.com/bytedance/monoio/issues/213)
- [Sharded Queue - the fastest concurrent collection](https://github.com/ivanivanyuk1993/utility.sharded_queue)

## License

Licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT) at your option.
