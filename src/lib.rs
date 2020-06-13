//! Log from an i.MX RT processor over UART
//!
//! The crate provides a [`log`](https://crates.io/crates/log) implementation that
//! pipes data over UART. Users are expected to configure a UART peripheral with baud
//! rates, parities, inversions, etc. After configuring the peripheral, users should
//! call [`init`](fn.init.html) to prepare the logger.
//!
//! The implementation blocks, buffering data into the UART transfer FIFO, until the final
//! bytes are enqueued in the FIFO. The implementation logs data **in an interrupt free
//! critical section**. Logging will not be preempted by an interrupt; logging may reduce
//! your system's responsiveness. It is safe to log from interrupt, fault, or panic handlers.
//!
//! Specify your maximum log level, and filter messages, using a
//! [`LoggingConfig`](struct.LoggingConfig.html).
//!
//! # Example
//!
//! ```no_run
//! use imxrt_hal;
//! use imxrt_uart_log as log;
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
//! // Recommended to use a larger TX FIFO size
//! uart.set_tx_fifo(core::num::NonZeroU8::new(4));
//! // Set other UART configurations...
//!
//! let (tx, rx) = uart.split();
//! log::init(tx, log::LoggingConfig::default()).unwrap();
//!
//! // At this point, you may use log macros to write data.
//! ```

#![no_std]

use core::{cell::RefCell, fmt};
use cortex_m::interrupt::{self, Mutex};
use embedded_hal::blocking::serial::Write;
use imxrt_hal::uart;

// The implementation has a few `expects()` that assume no errors
// from the UART write operations. This statically asserts that the
// assumptions hold (at least for UART1...).
type _Error = <uart::UART<uart::module::_1> as Write<u8>>::Error;
trait _IsInfallible {
    const VALUE: bool = false;
}
impl _IsInfallible for core::convert::Infallible {
    const VALUE: bool = true;
}
const _UART_ERROR_INFALLIBLE: [u8; 1] = [0; <_Error as _IsInfallible>::VALUE as usize];

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
    /// the accompanying log level. If there is no level, we default
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

#[doc(hidden)] // Required in public interface, but an implementation detail
pub enum Sink {
    _1(uart::Tx<uart::module::_1>),
    _2(uart::Tx<uart::module::_2>),
    _3(uart::Tx<uart::module::_3>),
    _4(uart::Tx<uart::module::_4>),
    _5(uart::Tx<uart::module::_5>),
    _6(uart::Tx<uart::module::_6>),
    _7(uart::Tx<uart::module::_7>),
    _8(uart::Tx<uart::module::_8>),
}

impl fmt::Write for Sink {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        match self {
            Sink::_1(uart) => uart.bwrite_all(string.as_bytes()).map_err(|_| fmt::Error),
            Sink::_2(uart) => uart.bwrite_all(string.as_bytes()).map_err(|_| fmt::Error),
            Sink::_3(uart) => uart.bwrite_all(string.as_bytes()).map_err(|_| fmt::Error),
            Sink::_4(uart) => uart.bwrite_all(string.as_bytes()).map_err(|_| fmt::Error),
            Sink::_5(uart) => uart.bwrite_all(string.as_bytes()).map_err(|_| fmt::Error),
            Sink::_6(uart) => uart.bwrite_all(string.as_bytes()).map_err(|_| fmt::Error),
            Sink::_7(uart) => uart.bwrite_all(string.as_bytes()).map_err(|_| fmt::Error),
            Sink::_8(uart) => uart.bwrite_all(string.as_bytes()).map_err(|_| fmt::Error),
        }
    }
}

impl Sink {
    fn flush(&mut self) {
        match self {
            Sink::_1(uart) => uart.bflush(),
            Sink::_2(uart) => uart.bflush(),
            Sink::_3(uart) => uart.bflush(),
            Sink::_4(uart) => uart.bflush(),
            Sink::_5(uart) => uart.bflush(),
            Sink::_6(uart) => uart.bflush(),
            Sink::_7(uart) => uart.bflush(),
            Sink::_8(uart) => uart.bflush(),
        }
        .expect("flush never fails");
    }
}

impl From<uart::Tx<uart::module::_1>> for Sink {
    fn from(tx: uart::Tx<uart::module::_1>) -> Self {
        Sink::_1(tx)
    }
}

impl From<uart::Tx<uart::module::_2>> for Sink {
    fn from(tx: uart::Tx<uart::module::_2>) -> Self {
        Sink::_2(tx)
    }
}

impl From<uart::Tx<uart::module::_3>> for Sink {
    fn from(tx: uart::Tx<uart::module::_3>) -> Self {
        Sink::_3(tx)
    }
}

impl From<uart::Tx<uart::module::_4>> for Sink {
    fn from(tx: uart::Tx<uart::module::_4>) -> Self {
        Sink::_4(tx)
    }
}

impl From<uart::Tx<uart::module::_5>> for Sink {
    fn from(tx: uart::Tx<uart::module::_5>) -> Self {
        Sink::_5(tx)
    }
}

impl From<uart::Tx<uart::module::_6>> for Sink {
    fn from(tx: uart::Tx<uart::module::_6>) -> Self {
        Sink::_6(tx)
    }
}

impl From<uart::Tx<uart::module::_7>> for Sink {
    fn from(tx: uart::Tx<uart::module::_7>) -> Self {
        Sink::_7(tx)
    }
}

impl From<uart::Tx<uart::module::_8>> for Sink {
    fn from(tx: uart::Tx<uart::module::_8>) -> Self {
        Sink::_8(tx)
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

/// Initialize the transfer half of a UART peripheral to be the logging sink
///
/// `tx` should be an `imxrt_hal::uart::Tx` half, obtained by calling `split()`
/// on a `UART` peripheral. Returns an error if you've already called `init()`.
///
/// See the [module-level example](index.html#example) for more information.
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
