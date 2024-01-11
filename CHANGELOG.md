# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/Miaxos/sharded-thread/compare/v0.1.2...v1.0.0) - 2024-01-11

### Added
- remove useless dependencies
- add some documentation and work on some methods

### Other
- add a ref to sharded queue
- clippy
- fmt
- udpate readme

## [0.1.2](https://github.com/Miaxos/sharded-thread/compare/v0.1.1...v0.1.2) - 2024-01-10

### Other
- remove hi_mom fn

## [0.1.1](https://github.com/Miaxos/sharded-thread/compare/v0.1.0...v0.1.1) - 2024-01-10

### Added
- remove lifetime with shard by replacing ref with Arc
- first impl

### Fixed
- tcp example without any std::mem::forget
- clippy

### Other
- disable some cross tests
- add some citations
- update README
- add some comments
- add test to load balance tcp (bis)
- add test to load balance tcp
- fmt tests
- add toolchain
- add some info in the readme
- first commit
