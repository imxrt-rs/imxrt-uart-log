# imxrt-uart-log

Log data over a serial interface. There are two logging implementations for NXP's i.MX RT processors:

- a simple, blocking logger. Useful for basic logging throughout the software stack, including interrupt, fault, and panic handlers.
- a DMA-based, non-blocking interface. Useful for logging that needs to happen quickly in thread mode. Uses a default buffer, with an option for a user-supplied DMA buffer.

Built on the [`imxrt-hal`] hardware abstraction layer for i.MX RT processors, version 0.3. Compatible with [`log`] version 0.4.

[`imxrt-hal`]: https://crates.io/crates/imxrt-hal
[`log`]: https://crates.io/crates/log

See the documentation for recommended use-cases, implementation descriptions, and examples:

```
cargo doc --open
```

## i.MX RT Compatibility

This crate supports all of the same i.MX RT variants as the [`imxrt-hal`] crate. To see the supported i.MX RT variants, check the [HAL's feature support](https://github.com/imxrt-rs/imxrt-rs#hal) list.

> :information_source: As of this writing, the HAL only supports one i.MX RT variant, the `"imxrt1062"`. For convenience, the `"imxrt1062"` feature is this crate's **default** feature. This default feature may change in future releases.

## Performance

The table below describes the execution time for logging statements on a Teensy 4. For more information on the test setup, consult the crate documentation. See the two examples to reproduce the test.

| Logging Invocation                                    | Execution Time, Blocking (us) | Execution Time, DMA (us) |
| ----------------------------------------------------- | ----------------------------- | ------------------------ |
| `log::info!("Hello world! 3 + 2 = {}", 3 + 2);`       | 3120                          | 3.16                     |
| `log::info!("Hello world! 3 + 2 = 5");`               | 3120                          | 2.84                     |
| `log::info!("");`                                     | 1220                          | 2.36                     |
| `log::info!(/* 100 character string */);`             | 9880                          | 4.12                     |

## Testing

The crate's examples run on hardware. See the documentation at the top of each example for more information.

For examples that run on a Teensy 4, you'll need the build dependencies described in the [`teensy4-rs` project](https://github.com/mciantyre/teensy4-rs#dependencies).

Use `make` to build an example:

```
make t4_blocking
```

When building an example for the Teensy 4, the build will print the location of the `*.hex` file. You may download the file to a Teensy using either the [Teensy Loader Application](https://www.pjrc.com/teensy/loader.html) or the [`teensy_loader_cli`](https://github.com/PaulStoffregen/teensy_loader_cli) command-line Teensy loader.

To run this crate's unit tests, and to check documentation examples, use `make test`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.