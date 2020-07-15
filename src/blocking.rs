//! A logging implementation that blocks when writing data
//!
//! The logger is simple to set up, and it will accept as much data as you'd like to write. To log data,
//!
//! 1. Configure a UART peripheral with baud rates, parities, inversions, etc.
//! 2. Call [`init`](fn.init.html) with the UART transfer half, and a [`LoggingConfig`](struct.LoggingConfig.html).
//!    If the default logging behavior is acceptable, use `Default::default()` to skip logging configuration.
//! 3. Use the macros from the [`log`](https://crates.io/crates/log) crate to write data
//!
//! # Use-cases
//!
//! - Simply debugging programs and libraries
//! - Frequently logging large strings
//! - `panic!()` handlers, and printing-out panic messages
//! - Logging in interrupt and fault handlers
//!
//! # Implementation
//!
//! The implementation blocks, buffering data into the UART transfer FIFO, until the final
//! bytes are enqueued in the FIFO. The implementation logs data **in an interrupt free
//! critical section**. Interrupts **will not** preempt logging, and logging may reduce
//! the system's responsiveness. To evaluate some simple performance measurements, see
//! [Performance](../index.html#performance).
//!
//! Consider using the largest size transfer FIFO to support UART data transfer. See the example
//! for more details.
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
//!         peripherals.iomuxc.ad_b1.p02,
//!         peripherals.iomuxc.ad_b1.p03,
//!         115_200,
//!     )
//!     .unwrap();
//!
//! // Consider using a large TX FIFO size
//! uart.set_tx_fifo(core::num::NonZeroU8::new(4));
//! // Set other UART configurations...
//!
//! let (tx, rx) = uart.split();
//! imxrt_uart_log::blocking::init(tx, Default::default()).unwrap();
//!
//! // At this point, you may use log macros to write data.
//! log::info!("Hello world!");
//! ```

mod sink;
use sink::Sink;

use crate::{Filters, LoggingConfig, SetLoggerError};
use core::cell::RefCell;
use cortex_m::interrupt::{self, Mutex};

struct Logger {
    /// The peripheral
    uart: Mutex<RefCell<Sink>>,
    /// A collection of targets that we are expected
    /// to filter. If this is empty, we allow everything
    filters: Filters,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &::log::Metadata) -> bool {
        metadata.level() <= ::log::max_level() // The log level is appropriate
            && self.filters.is_enabled(metadata) // The target is in the filter list
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

/// Initialize the blocking logger with a UART's transfer half
///
/// `tx` should be an `imxrt_hal::uart::Tx` half, obtained by calling `split()`
/// on a configured `UART` peripheral. Returns an error if you've already called `init()`, or if
/// you've already specified a logger through another interface.
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
                filters: Filters(config.filters),
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
