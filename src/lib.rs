//! Logging over an i.MX RT serial interface
//!
//! The crate provides a [`log`](https://crates.io/crates/log) implementation that
//! transfers data over UART. It's an extension to the [`imxrt-hal`](https://crates.io/crates/imxrt-hal)
//! crate. The crate provides two logging implementations:
//!
//! - a simple, [blocking](blocking/index.html) interface. Useful for simple logging, and for safely logging in interrupt, fault, and
//!   panic handlers.
//! - a [DMA-based](dma/index.html), non-blocking interface. Useful for logging that needs to happen quickly, but does
//!   not need to be responsive. More complicated to set up than the blocking interface.
//!
//! The module-level documentation provides examples and recommended use-cases. To see some comparisons between the two,
//! see [Performance](#performance).
//!
//! # i.MX RT Compatibility
//!
//! This crate supports all of the same i.MX RT variants as the `imxrt-hal` crate.
//! To see the supported i.MX RT variants, check the [HAL's feature support](https://github.com/imxrt-rs/imxrt-rs#hal) list.
//!
//! **Note**: As of this writing, the HAL only supports one i.MX RT variant, the `"imxrt1062"`.
//! For convenience, the `"imxrt1062"` feature is enabled **by default**. This may change in the
//! future.
//!
//! # Performance
//!
//! We measured logging execution on a Teensy 4. We configured a UART peripheral following the examples in each module.
//! We measured the CPU clock cycles required to execute each logging statement.
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
//! CPU clock cycles measure the elapse of cycle counts before and after a `log::info!()` macro.
//!
//! | Logging Invocation                                    | CPU Clock Cycles, Blocking | CPU Clock Cycles, DMA |
//! | ----------------------------------------------------- | -------------------------- | --------------------- |
//! | `log::info!("Hello world! 3 + 2 = {}", 3 + 2);`       | 2341330                    | 1871                  |
//! | `log::info!("Hello world! 3 + 2 = 5");`               | 2341273                    | 1683                  |
//! | `log::info!("");`                                     | 1197246                    | 1428                  |
//! | `log::info!(/* 100 character string */);`             | 6397302                    | 2463                  |
//!
//! Use this crate's examples to reproduce these measurements.

#![no_std]

pub mod blocking;
pub mod dma;
mod filters;

pub use filters::Filter;
use filters::Filters;

/// Logging configuration
///
/// Allows a user to specify certain configurations of the logging
/// system. By default, the max log level is the log level set at
/// compile time. See the [compile time filters](https://docs.rs/log/0.4.8/log/#compile-time-filters)
/// section for more information. We also enable logging for all targets.
/// Set the `filters` collection to specify log targets of interest.
///
/// If the default configuration is good for you, use `Default::default()`.
///
/// ```
/// use imxrt_uart_log::{Filter, LoggingConfig};
///
/// const I2C_LOGGING: Filter = ("i2c", None);
/// const SPI_LOGGING: Filter = ("spi", Some(log::LevelFilter::Warn));
/// const MOTOR_LOGGING: Filter = ("motor", Some(log::LevelFilter::Trace));
///
/// let config = LoggingConfig {
///     max_level: log::LevelFilter::Debug,
///     filters: &[
///         I2C_LOGGING,
///         SPI_LOGGING,
///         MOTOR_LOGGING,
///     ]
/// };
/// ```
pub struct LoggingConfig {
    /// The max log level for *all* logging
    ///
    /// By default, we select the static max level. Users may
    /// override this if they'd like to bypass the statically-assigned
    /// max level
    pub max_level: ::log::LevelFilter,
    /// A list of filtered targets to log.
    ///
    /// If set to an empty slice (default), the logger performs no
    /// filtering. Otherwise, we filter the specified targets by
    /// the accompanying log level. See [`Filter`](type.Filter.html) for
    /// more information.
    pub filters: &'static [Filter],
}

impl Default for LoggingConfig {
    fn default() -> LoggingConfig {
        LoggingConfig {
            max_level: ::log::STATIC_MAX_LEVEL,
            filters: &[],
        }
    }
}

/// An error that indicates the logger is already set
///
/// The error could propagate from one of the `init()` functions.
/// Or, it could propagate if the underlying logger was set through another logging
/// interface.
#[derive(Debug)]
pub struct SetLoggerError(());

impl From<::log::SetLoggerError> for SetLoggerError {
    fn from(_: ::log::SetLoggerError) -> SetLoggerError {
        SetLoggerError(())
    }
}
