# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

I will add links where possible, including retroactively if possible.

## [Unreleased](https://github.com/jewlexx/discord-presence/tree/trunk)

## [0.5.10] - Unreleased

### Added

- `block_until_event` function which blocks the current thread until a given event is fired

### Changed

- Use `AtomicBool` instead of `Mutex<bool>`

## [0.5.9](https://github.com/jewlexx/discord-presence/releases/tag/v0.5.9) - 2022-10-04

### Fixed

- Send/Receive loop would timeout indefinitely

### Changed

- Use [`tracing`](https://crates.io/crates/tracing) crate for logs

## [0.5.8](https://github.com/jewlexx/discord-presence/releases/tag/v0.5.8) - 2022-09-18

### Fixed

- party.id should be String, not u32 by @bigfarts in <https://github.com/jewlexx/discord-presence/pull/15>

### Changed

- Update actions/cache action to v3.0.8 by @renovate in <https://github.com/jewlexx/discord-presence/pull/14>

## [0.5.7](https://github.com/jewlexx/discord-presence/releases/tag/v0.5.7) - 2022-08-05

### Changed

- Downgrade compiler edition by @jewlexx in <https://github.com/jewlexx/discord-presence/pull/13>

## [0.5.6](https://github.com/jewlexx/discord-presence/releases/tag/v0.5.6) - 2022-08-01

### Fixed

- Minor bug fix relating to empty RPC pipe

### Changed

- Configure Renovate by @renovate in <https://github.com/jewlexx/discord-presence/pull/8>
- Update actions/cache action to v3.0.5 by @renovate in <https://github.com/jewlexx/discord-presence/pull/9>
- Update Rust crate bytes to 1.2 by @renovate in <https://github.com/jewlexx/discord-presence/pull/11>

## [0.5.5](https://github.com/jewlexx/discord-presence/releases/tag/v0.5.5) - 2022-07-27

Full Changelog: [v0.5.4...v0.5.5](https://github.com/jewlexx/discord-presence/compare/v0.5.4...v0.5.5)

## [0.5.4](https://github.com/jewlexx/discord-presence/releases/tag/discord-rpc%400.5.0) - 2022-06-19

### Fixed

- Fixed issues with timeouts on Discord connections
- Fixed issues with Unix connections

## [0.5.0](https://github.com/jewlexx/discord-presence/releases/tag/discord-rpc%400.5.0) - 2022-04-21

### Changed

- Removed `rich_presence` as a feature option, as it is redundant

## 0.4.2-0.4.4 - 2022-04-12

### Changed

- Updated Readme and metadata

## 0.4.1 - 2022-04-12

### Changed

- Minor bug fixes and performance improvements

## 0.4.0 - 2022-04-12

### Admin

- Under new ownership, forked by [Juliette Codor (jewlexx)](https://github.com/jewlexx)

### Changed

- Updated to newest Rust compiler edition (2021)

- Updated deps to latest version

- Fixed issues that came with the above changes

## 0.3.0 - 2018-12-06

### Changed

- Connection manager completely rewritten
- Allow cloning of clients

## [0.2.4] - 2018-12-04

### Changed

- No longer depends on `libc` for process id lookup

## [0.2.3] - 2018-04-08

### Added

- Connection manager with reconnection
- Method to clear the current Rich Presence state

### Changed

- Move rich presence code back into _models_
- Remove command payload and add generic one
- Timestamps are now 64 bit unsigned integers instead of 32 bit ([@Bond-009]) [6bbc9f8][c:6bbc9f8]

## [0.2.2] - 2018-04-03

### Changed

- Use a default socket connection for the current platform

## [0.2.1] - 2018-04-03

### Changed

- Move common connection methods into trait

## [0.2.0] - 2018-04-02

### Added

- Error type
- Windows support ([@Tenrys]) [620e9a6][c:620e9a6]

### Changed

- Convert OpCode with `try_from` instead of `try`
- Use Rust 1.25 style nested imports

## [0.1.5] - 2018-03-28

### Changed

- Opcode stored in Message is now an OpCode enum
- Rich Presence now lives in it's own submodule

## [0.1.4] - 2018-03-23

### Changed

- Opcodes are now represented as enum instead of integers

## [0.1.3] - 2018-03-23

### Added

- Contributing information

### Changed

- Use `libc::getpid` to allow builds with _stable_ instead of _nightly_
- Make client struct fields private
- Make models private again and add prelude
- Connections are now using a shared Connection trait

## [0.1.2] - 2018-03-22

### Added

- Logging support

## [0.1.1] - 2018-03-22

### Changed

- Make models publicly accessible

## [0.1.0] - 2018-03-22

### Added

- Setting Rich Presence status
- Unix socket connection support

<!-- links -->

[0.2.4]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.2.4
[0.2.3]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.2.3
[0.2.2]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.2.2
[0.2.1]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.2.1
[0.2.0]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.2.0
[0.1.5]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.1.5
[0.1.4]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.1.4
[0.1.3]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.1.3
[0.1.2]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.1.2
[0.1.1]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.1.1
[0.1.0]: https://gitlab.com/valeth/discord-rpc-client.rs/tree/v0.1.0
