# imxrt-uart-log

> :warning: This crate is built upon an unreleased [`imxrt-hal`] crate. To track the release progress, see [`imxrt-hal` #59](https://github.com/imxrt-rs/imxrt-rs/issues/59).

A logging implementation for i.MX RT processors, letting users log data over a serial interface.

1. Configure an [`imxrt-hal`] UART peripheral
2. Set the logger with `init()`
3. Use the macros from the [`log`] crate to write data

Built on the [`imxrt-hal`] hardware abstraction layer for i.MX RT processors, version 0.3. Compatible with [`log`] version 0.4.

[`imxrt-hal`]: https://crates.io/crates/imxrt-hal
[`log`]: https://crates.io/crates/log

## Use-cases

- Simply debugging applications and libraries
- `panic!()` handlers, and printing panic messages
- Unambiguously detecting interrupts and faults

## i.MX RT Compatibility

This crate supports all of the same i.MX RT variants as the [`imxrt-hal`] crate. To see the supported i.MX RT variants, check the [HAL's feature support](https://github.com/imxrt-rs/imxrt-rs#hal) list.

> :information_source: As of this writing, the HAL only supports one i.MX RT variant, the `"imxrt1062"`. For convenience, the `"imxrt1062"` feature is this crate's **default** feature. This default feature may change in future releases.

## Implementation

The implementation blocks, buffering data into the UART transfer FIFO, until the final bytes are enqueued in the FIFO. The implementation logs data **in an interrupt free critical section**. Interrupts **will not** preempt logging. Logging may reduce the system's responsiveness.

## Performance

The table below describes the execution time for logging statements on a Teensy 4. For more information on the test setup, consult the crate documentation, or see the [`log_uart.rs` example](examples/log_uart.rs).

| Logging Invocation                                    | Execution Time (ms) |
| ----------------------------------------------------- | ------------------- |
| `log::info!("Hello world! 3 + 2 = {}", 3 + 2);`       | 3.12                |
| `log::info!("Hello world! 3 + 2 = 5");`               | 3.12                |
| `log::info!("");`                                     | 1.22                |
| `log::info!(/* 100 character string */);`             | 9.88                |

## Testing

The crate's examples run on hardware. See the documentation at the top of each example for more information.

For examples that run on a Teensy 4, you'll need the build dependencies described in the [`teensy4-rs` project](https://github.com/mciantyre/teensy4-rs#dependencies).

Use `make` to build an example:

```
make log_uart
```

When building an example for the Teensy 4, the build will print the location of the `*.hex` file. You may download the file to a Teensy using either the [Teensy Loader Application](https://www.pjrc.com/teensy/loader.html) or the [`teensy_loader_cli`](https://github.com/PaulStoffregen/teensy_loader_cli) command-line Teensy loader.

To run this crate's unit tests, and to check documentation examples, use `make test`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.