//! Asynchronous logging using DMA
//!
//! This logger is slightly more complicated to set up. To log data,
//!
//! 1. Configure a UART peripheral with baud rates, parities, inversions, etc.
//! 2. Select a DMA channel. Take note of the DMA channel number.
//! 3. Implement the DMA channel's interrupt handler, and call [`poll()`](fn.poll.html)
//!    in the implementation. Or, prepare an event loop that calls `poll()` repeatedly.
//! 4. Unmask the interrupt via an `unsafe` call to `cortex_m::interrupt::unmask()`.
//! 5. Call [`init`](fn.init.html) with all of
//!   - a UART transfer half,
//!   - a DMA channel
//!   - a logging configuration
//! 6. Use the macros from the [`log`](https://crates.io/crates/log) crate to write data
//!
//! # Use-cases
//!
//! - Infrequently Logging smaller messages with minimal delay
//! - Logging only in thread mode (no guaranteed interrupt of fault handler logging)
//!
//! # Implementation
//!
//! The implementation minimizes the time it takes to call and return from a `log` macro.
//! The caller incurs the time it takes to perform any string interpolation and a copy into
//! a circular buffer. Both string interpolation and copying occur in a critical section. Then,
//!
//! - if there is no active DMA transfer, the logger schedules a DMA transfer, and returns.
//! - if there is an active DMA transfer, the logger returns immediately. When the active DMA transfer completes,
//!   any content written to the buffer during the transfer will be transferred.
//!
//! By default, the implementation relies on a 2KiB statically-allocated circular buffer. If you saturate
//! the buffer before the next transfer is scheduled, the data that cannot be copied into the buffer
//! **will be dropped.** Either keep messages small, or keep messages infrequent, to avoid circular buffer saturation.
//!
//! # Example
//!
//! In this example, we select DMA channel 7 to use for logging transfers. We implement the `DMA7_DMA23` interrupt to
//! service DMA transfers. We need to `unmask` the `DMA7_DMA23` interrupt for proper operation. See the comments for
//! more information
//!
//! ```no_run
//! use imxrt_hal::ral::interrupt;
//!
//! // Assume that DMA7_DMA23 is registered in the vector table
//! fn DMA7_DMA23() {
//!     imxrt_uart_log::dma::poll();
//! }
//!
//! let mut peripherals = imxrt_hal::Peripherals::take().unwrap();
//!
//! let (_, ipg_hz) = peripherals.ccm.pll1.set_arm_clock(
//!     imxrt_hal::ccm::PLL1::ARM_HZ,
//!     &mut peripherals.ccm.handle,
//!     &mut peripherals.dcdc,
//! );
//!
//! let mut dma_channels = peripherals.dma.clock(&mut peripherals.ccm.handle);
//! let channel = dma_channels[7].take().unwrap();
//!
//! let uarts = peripherals.uart.clock(
//!     &mut peripherals.ccm.handle,
//!     imxrt_hal::ccm::uart::ClockSelect::OSC,
//!     imxrt_hal::ccm::uart::PrescalarSelect::DIVIDE_1,
//! );
//! let uart = uarts
//!     .uart2
//!     .init(
//!         peripherals.iomuxc.gpio_ad_b1_02.alt2(),
//!         peripherals.iomuxc.gpio_ad_b1_03.alt2(),
//!         115_200,
//!     )
//!     .unwrap();
//!
//! let (tx, _) = uart.split();
//! imxrt_uart_log::dma::init(tx, channel, Default::default()).unwrap();
//!
//! // Don't forget to unmask the interrupt!
//! unsafe {
//!     cortex_m::peripheral::NVIC::unmask(interrupt::DMA7_DMA23);
//! }
//!
//! // At this point, you may use log macros to write data.
//! log::info!("Hello world!");
//! ```

mod sink;
mod writer;
use sink::{IntoSink, Sink};
use writer::Writer;

use crate::{Filters, LoggingConfig, SetLoggerError};
use core::{cell::RefCell, fmt::Write};
use cortex_m::interrupt::{self, Mutex};
use imxrt_hal::dma::{Channel, Circular};

struct Inner {
    sink: Sink,
    buffer: Option<Circular<u8>>,
}

struct Logger {
    filters: Filters,
    inner: Mutex<RefCell<Inner>>,
}

static LOGGER: Mutex<RefCell<Option<Logger>>> = Mutex::new(RefCell::new(None));

impl ::log::Log for Logger {
    fn enabled(&self, metadata: &::log::Metadata) -> bool {
        metadata.level() <= ::log::max_level() // The log level is appropriate
            && self.filters.is_enabled(metadata) // The target is in the filter list
    }

    fn flush(&self) { /* Nothing to do */
    }

    fn log(&self, record: &::log::Record) {
        if self.enabled(record.metadata()) {
            // TODO could perform string interpolation outside of critical section,
            // at the cost of additional memory usage...
            interrupt::free(|cs| {
                let logger = self.inner.borrow(cs);
                let mut logger = logger.borrow_mut();

                if let Some(mut buffer) = logger.buffer.take() {
                    // We have the buffer here, so there's not an active transfer
                    write!(
                        Writer::Circular(&mut buffer),
                        "[{} {}]: {}\r\n",
                        record.level(),
                        record.target(),
                        record.args()
                    )
                    .expect("never fails");
                    // Start the transfer
                    logger.sink.start_transfer(buffer);
                } else {
                    // There's an active transfer; find the buffer in the peripheral,
                    // and fill it with data
                    let mut buffer = logger.sink.write_half().unwrap();
                    write!(
                        Writer::WriteHalf(&mut buffer),
                        "[{} {}]: {}\r\n",
                        record.level(),
                        record.target(),
                        record.args()
                    )
                    .expect("never fails");
                }
            })
        }
    }
}

/// Drives DMA-based logging over serial
///
/// You *must* call this repeatedly to drive the DMA-based logging. We recommend that
/// you enable the interrupt for a DMA channel, and call `poll()` in
/// the interrupt handler. Or, you may call this repeatedly in an event loop.
/// If the transfer is not complete, `poll()` does nothing.
#[inline]
pub fn poll() {
    interrupt::free(|cs| {
        let logger = LOGGER.borrow(cs);
        let mut logger = logger.borrow_mut();
        let logger = logger.as_mut().unwrap();

        let logger = logger.inner.borrow(cs);
        let mut logger = logger.borrow_mut();

        if logger.sink.is_transfer_interrupt() {
            logger.sink.transfer_clear_interrupt();
            let buffer = logger.sink.transfer_complete().unwrap();
            if !buffer.is_empty() {
                // There's pending data to send
                logger.sink.start_transfer(buffer);
            } else {
                // No pending data; wait for next `log()` call
                logger.buffer = Some(buffer);
            }
        }
    });
}

/// Initialize the DMA-based logger with a UART transfer half and a DMA channel
///
/// `tx` should be an `imxrt_hal::uart::Tx` half, obtained by calling `split()` on a
/// configured `UART` peripheral. Returns an error if you've already called `init()`, or if
/// you've already specified a logger through another interface.
///
/// See the [module-level documentation](index.html#example) for an example.
pub fn init<T>(tx: T, channel: Channel, config: LoggingConfig) -> Result<(), SetLoggerError>
where
    T: IntoSink,
{
    interrupt::free(|cs| {
        let logger = LOGGER.borrow(cs);
        let mut logger = logger.borrow_mut();
        if logger.is_none() {
            *logger = Some(Logger {
                inner: Mutex::new(RefCell::new(Inner {
                    sink: tx.into_sink(channel),
                    buffer: Some(Circular::new(&buffer::BUFFER.0).unwrap()),
                })),
                filters: Filters(config.filters),
            })
        }

        // Safety: lifetime is static, and we're transmuting lifetimes
        let logger: &'static Logger = unsafe { core::mem::transmute(logger.as_ref().unwrap()) };
        ::log::set_logger(logger)
            .map(|_| ::log::set_max_level(config.max_level))
            .map_err(From::from)
    })
}

mod buffer {
    use imxrt_hal::dma::Buffer;

    #[repr(align(2048))]
    pub struct Alignment(pub Buffer<[u8; 2048]>);

    pub static BUFFER: Alignment = Alignment(Buffer::new([0; 2048]));
}
