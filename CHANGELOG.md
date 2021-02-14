# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- New `DisplayError` variant `RSError` to use with problems with the display's reset signal

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

[Unreleased]: https://github.com/therealprof/display-interface/compare/v0.4.0...HEAD
[v0.4.0]: https://github.com/therealprof/display-interface/compare/v0.3.0...v0.4.0
[v0.3.0]: https://github.com/therealprof/display-interface/compare/v0.2.1...v0.3.0
[v0.2.1]: https://github.com/therealprof/display-interface/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/therealprof/display-interface/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/therealprof/display-interface/compare/v0.1.0...v0.1.1
