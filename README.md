# Display interface

This Rust crate contains a `no_std` compatible interface in form of traits to
bridge between a bus driver and a display driver. The goal here is to allow
display drivers to be written in a hardware interface agnostic way and prevent
code duplication and missing implementations.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
