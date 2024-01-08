# sharded-thread

[![release](https://github.com/Miaxos/json-predicate/actions/workflows/release.yml/badge.svg)](https://github.com/Miaxos/sharded-thread/actions/workflows/release.yml)
[![Crates.io version](https://img.shields.io/crates/v/sharded-thread.svg)](https://crates.io/crates/sharded-thread)
[![dependency status](https://deps.rs/repo/github/miaxos/sharded-thread/status.svg)](https://deps.rs/repo/github/miaxos/sharded-thread)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/sharded-thread)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/miaxos/sharded-thread/compare)


## Introduction

Sometimes, you want to be able to shard data between thread, because only this
thread is able to handle the data or you want to load balance between thread.

This is an experiment to try to tackle this issue: https://github.com/bytedance/monoio/issues/213.

## References

- https://github.com/DataDog/glommio/blob/master/examples/sharding.rs
- https://github.com/bytedance/monoio/issues/213

## License

Licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT) at your option.
