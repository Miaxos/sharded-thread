# sharded-thread

[![release](https://github.com/Miaxos/json-predicate/actions/workflows/release.yml/badge.svg)](https://github.com/Miaxos/sharded-thread/actions/workflows/release.yml)
[![Crates.io version](https://img.shields.io/crates/v/sharded-thread.svg)](https://crates.io/crates/sharded-thread)
[![dependency status](https://deps.rs/repo/github/miaxos/sharded-thread/status.svg)](https://deps.rs/repo/github/miaxos/sharded-thread)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/sharded-thread)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/miaxos/sharded-thread/compare)


## Introduction

This library is mainly made for `io-uring` and monoio. There are no dependency
on the runtime, so you should be able to use it with other runtime and also
without `io-uring`.

The purpose of this library is to have a performant way to send data between
thread when threads are following a `thread per core` architecture.

## Example

Originally, the library was made when you had multiple thread listening to the
same `TcpStream` and depending on what is sent through the `TcpStream` you might
want to change the thread handling the stream.

<p align="center">
    <img src="./.github/ressources/sharded-thread.drawio.svg" width="60%" />
</p>

You can check the code for this example here: WIP

## References

- https://github.com/DataDog/glommio/blob/master/examples/sharding.rs
- https://github.com/bytedance/monoio/issues/213

## License

Licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT) at your option.
