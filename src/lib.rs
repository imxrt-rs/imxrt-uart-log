//! Log over UART from an i.MX RT processor
//!
//! The crate provides a [`log`](https://crates.io/crates/log) implementation that
//! pipes data over UART. It's an extension to the [`imxrt-hal`](https://crates.io/crates/imxrt-hal)
//! crate. To log data over UART,
//!
//! 1. Configure a UART peripheral with baud rates, parities, inversions, etc.
//! 2. Call [`init`](fn.init.html) with a [`LoggingConfig`](struct.LoggingConfig.html). If the default
//!    logging behavior is acceptable, use `Default::default()` to skip logging configuration.
//! 3. Use the macros from the [`log`](https://crates.io/crates/log) crate to write data over UART
//!
//! The implementation blocks, buffering data into the UART transfer FIFO, until the final
//! bytes are enqueued in the FIFO. The implementation logs data **in an interrupt free
//! critical section**. Interrupts **will not** preempt logging, and logging may reduce
//! the system's responsiveness. To evaluate some simple performance measurements, see
//! [Performance](#performance).
//!
//! # Example
//!
//! ```no_run
//! use imxrt_hal;
//! use imxrt_uart_log;
//!
//! let mut peripherals = imxrt_hal::Peripherals::take().unwrap();
//!
//! let uarts = peripherals.uart.clock(
//!     &mut peripherals.ccm.handle,
//!     imxrt_hal::ccm::uart::ClockSelect::OSC,
//!     imxrt_hal::ccm::uart::PrescalarSelect::DIVIDE_1,
//! );
//!
//! let mut uart = uarts
//!     .uart2
//!     .init(
//!         peripherals.iomuxc.gpio_ad_b1_02.alt2(),
//!         peripherals.iomuxc.gpio_ad_b1_03.alt2(),
//!         115_200,
//!     )
//!     .unwrap();
//!
//! // Consider using a large TX FIFO size
//! uart.set_tx_fifo(core::num::NonZeroU8::new(4));
//! // Set other UART configurations...
//!
//! let (tx, rx) = uart.split();
//! imxrt_uart_log::init(tx, Default::default()).unwrap();
//!
//! // At this point, you may use log macros to write data.
//! ```
//!
//! # Performance
//!
//! We measured logging execution on a Teensy 4, with a 600MHz ARM clock. We configured
//! a UART peripheral following the example above. Using a general purpose timer (GPT),
//! we measured the time required to write various log messages. We verified GPT timings
//! with a logic analyzer, which observed a pulse on a GPIO.
//!
//! By default, a logging call resembling
//!
//! ```text
//! log::info!("Hello world! 3 + 2 = {}", 3 + 2);
//! ```
//!
//! Produces a message resembling
//!
//! ```text
//! [INFO log_uart]: Hello world! 3 + 2 = 5
//! ```
//!
//! where `INFO` describes the log level, `log_uart` describes the module, and the remainder
//! of the message is the serialized content.
//!
//! | Logging Invocation                                    | Execution Time (ms) |
//! | ----------------------------------------------------- | ------------------- |
//! | `log::info!("Hello world! 3 + 2 = {}", 3 + 2);`       | 3.12                |
//! | `log::info!("Hello world! 3 + 2 = 5");`               | 3.12                |
//! | `log::info!("");`                                     | 1.22                |
//! | `log::info!(/* 100 character string */);`             | 9.88                |

#![no_std]

mod sink;
use sink::Sink;

use core::cell::RefCell;
use cortex_m::interrupt::{self, Mutex};

/// Logging configuration
///
/// Allows a user to specify certain configurations of the logging
/// system. By default, the max log level is the log level set at
/// compile time. See the [compile time filters](https://docs.rs/log/0.4.8/log/#compile-time-filters)
/// section for more information. We also enable logging for all targets.
/// Set the `filters` collection to specify log targets of interest.
///
/// If the default configuration is good for you, use `Default::default()` with
/// [`init`](fn.init.html).
pub struct LoggingConfig {
    /// The max log level
    ///
    /// By default, we select the static max level. Users may
    /// override this if they'd like to bypass the statically-assigned
    /// max level
    pub max_level: ::log::LevelFilter,
    /// A list of filtered targets to log.
    ///
    /// If set to an empty slice (default), the logger performs no
    /// filtering. Otherwise, we filter the specified targets by
    /// the accompanying log level. If there is no level, we allow
    /// all levels.
    pub filters: &'static [(&'static str, Option<::log::LevelFilter>)],
}

impl Default for LoggingConfig {
    fn default() -> LoggingConfig {
        LoggingConfig {
            max_level: ::log::STATIC_MAX_LEVEL,
            filters: &[],
        }
    }
}

struct Logger {
    /// The peripheral
    uart: Mutex<RefCell<Sink>>,
    /// A collection of targets that we are expected
    /// to filter. If this is empty, we allow everything
    filters: &'static [(&'static str, Option<::log::LevelFilter>)],
}

impl Logger {
    /// Returns true if the target is in the filter, else false if the target is
    /// not in the list of kept targets. If the filter collection is empty, return
    /// true.
    fn filtered(&self, metadata: &::log::Metadata) -> bool {
        if self.filters.is_empty() {
            true
        } else if let Some(idx) = self
            .filters
            .iter()
            .position(|&(target, _)| target == metadata.target())
        {
            let (_, lvl) = self.filters[idx];
            lvl.is_none() || lvl.filter(|lvl| metadata.level() <= *lvl).is_some()
        } else {
            false
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &::log::Metadata) -> bool {
        metadata.level() <= ::log::max_level() // The log level is appropriate
            && self.filtered(metadata) // The target is in the filter list
    }

    fn log(&self, record: &::log::Record) {
        if self.enabled(record.metadata()) {
            interrupt::free(|cs| {
                let uart = self.uart.borrow(cs);
                let mut uart = uart.borrow_mut();
                use core::fmt::Write;
                write!(
                    uart,
                    "[{} {}]: {}\r\n",
                    record.level(),
                    record.target(),
                    record.args()
                )
                .expect("write never fails");
            });
        }
    }

    fn flush(&self) {
        interrupt::free(|cs| {
            let uart = self.uart.borrow(cs);
            let mut uart = uart.borrow_mut();
            uart.flush();
        })
    }
}

/// An error that indicates the logger is already set
///
/// The error could propagate from this crate's [`init()`](fn.init.html) function.
/// Or, it could propagate if the underlying logger was set through another logging
/// interface.
#[derive(Debug)]
pub struct SetLoggerError(());

impl From<::log::SetLoggerError> for SetLoggerError {
    fn from(_: ::log::SetLoggerError) -> SetLoggerError {
        SetLoggerError(())
    }
}

/// Initialize the logger with a UART's transfer half
///
/// `tx` should be an `imxrt_hal::uart::Tx` half, obtained by calling `split()`
/// on a `UART` peripheral. Returns an error if you've already called `init()`.
///
/// See the [module-level documentation](index.html#example) for an example.
pub fn init<S>(tx: S, config: LoggingConfig) -> Result<(), SetLoggerError>
where
    S: Into<Sink>,
{
    static LOGGER: Mutex<RefCell<Option<Logger>>> = Mutex::new(RefCell::new(None));
    interrupt::free(|cs| {
        let logger = LOGGER.borrow(cs);
        let mut logger = logger.borrow_mut();
        if logger.is_none() {
            *logger = Some(Logger {
                uart: Mutex::new(RefCell::new(tx.into())),
                filters: config.filters,
            });
        }

        // Safety: transmute from limited lifetime 'a to 'static lifetime
        // is OK, since the derived memory has 'static lifetime. The need
        // for this comes from the `interrupt::free()` and `Mutex::borrow()`
        // interplay. The two require any references to be tied to the
        // lifetime of the critical section.
        let logger: &'static Logger = unsafe { core::mem::transmute(logger.as_ref().unwrap()) };
        ::log::set_logger(logger)
            .map(|_| ::log::set_max_level(config.max_level))
            .map_err(From::from)
    })
}
