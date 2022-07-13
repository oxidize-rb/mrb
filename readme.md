# `mrb`

[![Join the discussion](https://img.shields.io/badge/slack-chat-blue.svg)](https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw)
[![.github/workflows/ci.yml](https://github.com/oxidize-rb/mrb/actions/workflows/ci.yml/badge.svg)](https://github.com/oxidize-rb/mrb/actions/workflows/ci.yml)

The primary goal of `mrb` is to make building native Ruby extensions in Rust **easier** than it would be in C. If it's not easy, it's a bug.

- [Rust bindings (`mrb-sys` crate)](./crates/mrb-sys/readme.md)
- [Compile mruby for you Rust project (`mrb-src` crate)](./crates/mrb-src/readme.md)

## Features

- Auto-generated Rust bindings for mruby C API
- Cross compilation of mruby

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
