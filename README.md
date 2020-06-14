# imxrt-uart-log

> :warning: This crate is built upon an unreleased [`imxrt-hal`] crate. To track the release progress, see [`imxrt-hal` #59](https://github.com/imxrt-rs/imxrt-rs/issues/59).

A logging implementation for i.MX RT processors, letting users log data over a serial interface.

1. Configure an [`imxrt-hal`] UART peripheral
2. Set the logger with `init()`
3. Use the macros from the [`log`] crate to write data

Built on the [`imxrt-hal`] hardware abstraction layer for i.MX RT processors, version 0.3. Compatible with [`log`] version 0.4.

[`imxrt-hal`]: https://crates.io/crates/imxrt-hal
[`log`]: https://crates.io/crates/log

```rust
use imxrt_hal;
use imxrt_uart_log;

let mut peripherals = imxrt_hal::Peripherals::take().unwrap();

let uarts = peripherals.uart.clock(
    &mut peripherals.ccm.handle,
    imxrt_hal::ccm::uart::ClockSelect::OSC,
    imxrt_hal::ccm::uart::PrescalarSelect::DIVIDE_1,
);

let mut uart = uarts
    .uart2
    .init(
        peripherals.iomuxc.gpio_ad_b1_02.alt2(),
        peripherals.iomuxc.gpio_ad_b1_03.alt2(),
        115_200,
    )
    .unwrap();

// Consider using a large TX FIFO size
uart.set_tx_fifo(core::num::NonZeroU8::new(4));
// Set other UART configurations...

let (tx, rx) = uart.split();
imxrt_uart_log::init(tx, Default::default()).unwrap();

// At this point, you may use log macros to write data.
```

## Use-cases

- Simply debugging applications and libraries
- `panic!()` handlers, and printing panic messages
- Unambiguously detecting interrupts and faults

## Implementation

The implementation blocks, buffering data into the UART transfer FIFO, until the final bytes are enqueued in the FIFO. The implementation logs data **in an interrupt free critical section**. Interrupts **will not** preempt logging. Logging may reduce the system's responsiveness.

## Performance

The table below describes the execution time for logging statements on a Teensy 4. For more information on the test setup, consult the crate documentation.

| Logging Invocation                                    | Execution Time (ms) |
| ----------------------------------------------------- | ------------------- |
| `log::info!("Hello world! 3 + 2 = {}", 3 + 2);`       | 3.12                |
| `log::info!("Hello world! 3 + 2 = 5");`               | 3.12                |
| `log::info!("");`                                     | 1.22                |
| `log::info!(/* 100 character string */);`             | 9.88                |

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.