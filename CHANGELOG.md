# Changelog

## [Unreleased]

## [0.2.0] - 2020-08-29

### Added

- Support for using `dma::poll()` in an event loop.
- Return values from `dma::poll()`, allowing users to block until the logger
  is idle. See the documentation for usage tips.
- Documentation describing usage patterns for the async logger.
- Support DMA transfer rescheduling on loging calls. See the documentation for
  when this might be appropriate.

### Fixes

- Note that `dma::poll()` will panic if there is no logger.

### Changes

- Requires `imxrt-hal` version 0.4, which introduces breaking changes.
  See the [`imxrt-hal` release notes](https://github.com/imxrt-rs/imxrt-rs/releases)
  for more information.
- Measure logger performance in clock cycles, rather than elapsed time.

## [0.1.1] - 2020-07-07

### Fixes

Update documentation to describe limitations of the 0.1 release. See #2 for
more information.

## [0.1.0] - 2020-06-18

First release

### Added

- Blocking UART logger that writes data in a critical section
- Async DMA logger, with option to specify your own DMA buffer
- Configurable with module and log-level filters
- Documentation, examples, and performance testing results

[Unreleased]: https://github.com/imxrt-rs/imxrt-uart-log/compare/v0.2.0..HEAD
[0.2.0]: https://github.com/imxrt-rs/imxrt-uart-log/compare/v0.1.1..v0.2.0
[0.1.1]: https://github.com/imxrt-rs/imxrt-uart-log/compare/v0.1.0..v0.1.1
[0.1.0]: https://github.com/imxrt-rs/imxrt-uart-log/releases/tag/v0.1.0