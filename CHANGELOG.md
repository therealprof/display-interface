# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.4.1] - 2021-05-10

### Added

- New `DisplayError` variant `RSError` to use with problems with the display's reset signal
- New `DisplayError` variant `OutOfBoundsError` to use when writing to a non-existing pixel outside the display's bounds
- parallel-gpio (0.5.0): New `OutputBus` trait
- parallel-gpio (0.5.0): Added `Generic8BitBus`, an implementation of `OutputBus`

### Changed

- Return `DCError` instead of `BusWriteError` on errors (de-)asserting the DC signal in 8-bit GPIO interfaces
- **Breaking** parallel-gpio (0.5.0): `PGPIO8BitInterface` now uses any 8-bit impementation of `OutputBus` instead of 8 individual pins

## [v0.4.0] - 2020-05-25

### Added

- Support for 8bit and 16bit iterators as data format
- Support for 16bit slice data format with target endian
- Deconstructors for included display-interface implementations

### Changed

- Make enums non-exhaustive and added a DataFormatNotImplemented fallback error

## [v0.3.0] - 2020-05-11

### Added

- 16 bit data width support for 8 bit parallel-gpio and SPI impls

### Changed

- Data width is provided via custom enum (breaking change)

## [v0.2.1] - 2020-04-16

### Added

- Added prelude

## [v0.2.0] - 2020-04-01

### Changed

- Made data width generic (breaking change)

## [v0.1.1] - 2020-03-29

### Fixed

- Crate metadata

### Removed

- Examples requiring additional driver crates

## 0.1.0 - 2020-03-29

First version

[Unreleased]: https://github.com/therealprof/display-interface/compare/v0.4.1...HEAD
[v0.4.1]: https://github.com/therealprof/display-interface/compare/v0.4.0...v0.4.1
[v0.4.0]: https://github.com/therealprof/display-interface/compare/v0.3.0...v0.4.0
[v0.3.0]: https://github.com/therealprof/display-interface/compare/v0.2.1...v0.3.0
[v0.2.1]: https://github.com/therealprof/display-interface/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/therealprof/display-interface/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/therealprof/display-interface/compare/v0.1.0...v0.1.1
