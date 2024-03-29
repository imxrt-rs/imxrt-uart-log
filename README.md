# imxrt-uart-log

**Replaced by [`imxrt-log`].** imxrt-hal 0.5 users should use `imxrt-log`,
since this package only supports imxrt-hal 0.4 and will not be developed
further.

[`imxrt-log`]: https://crates.io/crates/imxrt-log

If you have a bug fix for this package that you would like released, [contact]
an imxrt-rs maintainer.

[Contact]: https://imxrt-rs.github.io/book/

---

[![Crates.io][crates-io-badge]][crates-io-url]
[![Build][build-badge]][build-url]

[crates-io-badge]: https://img.shields.io/crates/v/imxrt-uart-log
[crates-io-url]: https://crates.io/crates/imxrt-uart-log
[build-badge]:
https://github.com/imxrt-rs/imxrt-uart-log/workflows/All%20Checks/badge.svg
[build-url]:
https://github.com/imxrt-rs/imxrt-uart-log/actions?query=workflow%3A%22All+Checks%22

#### [API Docs](https://docs.rs/imxrt-uart-log/latest/imxrt_uart_log/)

Log data over a serial interface. There are two logging implementations for 
NXP's i.MX RT processors:

- a simple, blocking logger. Useful for basic logging throughout the software 
stack, including interrupt, fault, and panic handlers.
- a DMA-based, non-blocking interface. Useful for logging that needs to happen 
quickly. Uses a default buffer, with an option for a user-supplied DMA buffer.

Built on the [`imxrt-hal`] hardware abstraction layer for i.MX RT processors, 
version 0.4. Compatible with [`log`] version 0.4.

[`imxrt-hal`]: https://crates.io/crates/imxrt-hal
[`log`]: https://crates.io/crates/log

## i.MX RT Compatibility

This crate supports all of the same i.MX RT variants as the [`imxrt-hal`] 
crate. To see the supported i.MX RT variants, check the [HAL's feature 
support](https://github.com/imxrt-rs/imxrt-rs#hal) list.

> :information_source: As of this writing, the HAL only supports one i.MX RT 
variant, the `"imxrt1062"`. For convenience, the `"imxrt1062"` feature is this 
crate's **default** feature. This default feature may change in future releases.

## Testing

The crate's examples run on hardware. See the documentation at the top of each 
example for more information.

For examples that run on a Teensy 4, you'll need the build dependencies 
described in the [`teensy4-rs` 
project](https://github.com/mciantyre/teensy4-rs#dependencies).

Use `make` to build an example for the Teensy 4:

```
make t4_blocking
```

When building an example for the Teensy 4, the build will print the location of 
the `*.hex` file. You may download the file to a Teensy using either the 
[Teensy Loader Application](https://www.pjrc.com/teensy/loader.html) or the 
[`teensy_loader_cli`](https://github.com/PaulStoffregen/teensy_loader_cli) 
command-line Teensy loader.

To run this crate's unit tests, and to check documentation examples, use `make 
test`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.