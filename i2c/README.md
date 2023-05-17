# I2C interface for display interface

This Rust crate contains a generic I2C implementation of a data/command
interface for displays over any I2C driver implementing the `embedded-hal`
`blocking::I2C::Write` trait.

## Crate features

Additional features can be enabled by adding the following features to your Cargo.toml.

 - `async`: enables `AsyncWriteOnlyDataCommand`. This feature uses `#[async_trait]` and can be
   used with rustc 1.56 and newer. Using this feature requires allocator support.
 - `nightly`: Enables support for nightly-only, unstable features. Together with `async`, this will
   enable the `async_fn_in_trait` and `impl_trait_projections` unstable features,
   and removes the `#[async_trait]` workaround as well as the allocator requirement.
   This feature requires a nightly Rust compiler released on or after 2022-11-17.

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
