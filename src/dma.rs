//! Asynchronous logging using DMA
//!
//! This logger is slightly more complicated to set up. To log data,
//!
//! 1. Configure a UART peripheral with baud rates, parities, inversions, etc.
//! 2. Select a DMA channel. Take note of the DMA channel number.
//! 3. Call [`poll()`](fn.poll.html) in either your DMA channel's interrupt handler, or throughout
//!    your event loop.
//!   - If you're calling `poll()` in a DMA channel's interrupt handler, unmask the interrupt
//!    via an `unsafe` call to `cortex_m::interrupt::unmask()`, and also enable interrupt on
//!    completion.
//! 4. Call [`init`](fn.init.html) with all of
//!   - a UART transfer half,
//!   - a DMA channel
//!   - a logging configuration
//! 5. Use the macros from the [`log`](https://crates.io/crates/log) crate to write data.
//!
//! Optionally, you may specify your own DMA buffer. See the [BYOB](#byob) feature to learn about
//! user-supplied DMA buffers.
//!
//! # Use-cases
//!
//! - Infrequently logging smaller messages with minimal delay
//! - Logging where the responsiveness of the logging implementation may not be critical
//!
//! # Implementation
//!
//! The implementation minimizes the time it takes to call and return from a `log` macro.
//! The caller incurs the time it takes to perform any string interpolation and a copy into
//! a circular buffer. Both string interpolation and copying happen in a critical section. Then,
//!
//! 1. if there is no active DMA transfer, the logger schedules a DMA transfer, and returns.
//! 2. if there is a completed DMA transfer, the logger finalizes the transfer. The logger appends the new log message
//!    to the enqueued contents in the circular buffer. The logger schedules a DMA transfer, and returns.
//! 3. if there is an active DMA transfer, the logger enqueues the new message in the circular buffer, and returns immediately.
//!
//! The implementation schedules new transfers when the current transfer is complete, there are log messages in the circular
//! buff, and you either
//!
//! - call [`poll()`](fn.poll.html), or
//! - log another message (see 2. above)
//!
//! By default, the implementation relies on a 2KiB statically-allocated circular buffer. If you saturate
//! the buffer before the next transfer is scheduled, the data that cannot be copied into the buffer
//! **will be dropped.** Either keep messages small, or keep messages infrequent, to avoid circular buffer saturation.
//!
//! # Tips
//!
//! ## Interrupt priorities
//!
//! To improve logging responsiveness, consider changing your DMA channel's interrupt priority. This may be helpful
//! when frequently logging from interrupts. If your DMA channel's interrupt priority is greater than your other interrupt
//! priorities, `poll()` is more likely to be called, which will mean more data sent over serial.
//!
//! ## Flush the async logger
//!
//! To guarantee that a transfer completes, use [`poll()`](fn.poll.html) while waiting for an [`Idle`](struct.Poll.html) return:
//!
//! ```no_run
//! use imxrt_uart_log::dma::{poll, Poll};
//!
//! log::error!("Send message and wait for the transfer to finish");
//! while Poll::Idle != poll() {}
//! ```
//!
//! Note that this will flush *all* contents from the async logger, so you will also wait for any previously-scheduled
//! transfers to complete.
//!
//! ## Responsiveness
//!
//! If you do not care about logging responsiveness, or if you're OK with "bursty" logging, you do not need to call [`poll()`](fn.poll.html). The
//! logging implementation will finalize and schedule transfers when it detects that a previous transfer is complete. For example,
//!
//! ```no_run
//! log::info!("what's");   // Immediately schedules DMA transfer
//! log::info!("the");      // Enqueues message, but not transferred
//! // Some time later, when "what's" transfer completes...
//! log::info!("weather?"); // Sends both "the" and "weather?"
//! ```
//!
//! will immediately log `"what's"`. The second message `"the"` is written to the circular buffer, and it will be scheduled to transfer
//! when you write `"weather?"`.
//!
//! If the time between `"the"` and `"weather?"` is large, and you'd like to receive `"the"` before `"weather"` is written, add one or
//! or more `poll()` calls in between the two calls. The most responsive async logger will call `poll()` in the DMA channel's interrupt
//! handler, which will run as soon as a transfer completes.
//!
//! # Example
//!
//! In this example, we select DMA channel 7 to use for logging transfers. We implement the `DMA7_DMA23` interrupt to
//! service DMA transfers. We need to `unmask` the `DMA7_DMA23` interrupt for proper operation. See the comments for
//! more information.
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
//! let mut channel = dma_channels[7].take().unwrap();
//! // Enable interrupt generation when the DMA transfer completes
//! channel.set_interrupt_on_completion(true);
//! // Don't forget to unmask the interrupt!
//! unsafe {
//!     cortex_m::peripheral::NVIC::unmask(interrupt::DMA7_DMA23);
//! }
//!
//! let uarts = peripherals.uart.clock(
//!     &mut peripherals.ccm.handle,
//!     imxrt_hal::ccm::uart::ClockSelect::OSC,
//!     imxrt_hal::ccm::uart::PrescalarSelect::DIVIDE_1,
//! );
//! let uart = uarts
//!     .uart2
//!     .init(
//!         peripherals.iomuxc.ad_b1.p02,
//!         peripherals.iomuxc.ad_b1.p03,
//!         115_200,
//!     )
//!     .unwrap();
//!
//! let (tx, _) = uart.split();
//! imxrt_uart_log::dma::init(tx, channel, Default::default()).unwrap();
//!
//! // At this point, you may use log macros to write data.
//! log::info!("Hello world!");
//! ```
//!
//! # BYOB
//!
//! "Bring Your Own Buffer" (BYOB) is an optional, compile-time feature that affects the DMA logging API. If you enable
//! the `"byob"` feature, you indicate that you will statically allocate a circular DMA buffer, rather than relying on
//! the default DMA buffer. You may supply the buffer to [`init()`](fn.init.html).
//!
//! BYOB is useful if you want to control either the size or placement of the DMA buffer. You're responsible for following the
//! alignment requirements. See the i.MX RT HAL's DMA documentation for more details on DMA buffers.

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
    /// The buffer transitions into the DMA peripheral when there is an active
    /// transfer. If this is `Some(..)`, we're idle.
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
                } else if logger.sink.is_transfer_complete() {
                    // Transfer is complete. We need to finalize the transfer,
                    // and re-schedule it here.
                    let mut buffer = logger.sink.transfer_complete().unwrap();
                    write!(
                        Writer::Circular(&mut buffer),
                        "[{} {}]: {}\r\n",
                        record.level(),
                        record.target(),
                        record.args()
                    )
                    .expect("never fails");
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

/// A [`poll()`](fn.poll.html)ing result
///
/// `Poll` provides insight into the DMA logger's state
#[derive(Debug, PartialEq, Eq)]
pub enum Poll {
    /// There is an active transfer
    ///
    /// The next log message will be scheduled after the active transfer
    /// completes. The transfer will start on either the next call to
    /// [`poll()`](fn.poll.html), or along with the next written log message.
    ///
    /// An `Active` return could mean that
    ///
    /// - `poll()` was called while there was an active transfer, and nothing
    ///   happened.
    /// - `poll()` was called, and an active transfer is now complete. `poll()` scheduled
    ///   another transfer after detecting data in the circular buffer.
    Active,
    /// There is no active transfer, and the logger is idle
    ///
    /// The next log message will be scheduled immediately. An `Idle` result could
    /// mean that
    ///
    /// - `poll()` was called when there was no active transfer.
    /// - `poll()` was called, and an active transfer is now complete. There was no other
    ///   log message in the circular buffer, so there's nothing to do.
    Idle,
}

/// Drives DMA-based logging
///
/// You may call this repeatedly to drive the DMA-based logging. Calling `poll()`
/// can happen in the DMA channel's interrupt handler, or throughout an event loop. `poll()`
/// runs in a critical section.
///
/// If the transfer is not complete, `poll()` does nothing.
///
/// # Panics
///
/// If you failed to register a logger using [`init()`](fn.init.html), `poll()` panics.
#[inline]
pub fn poll() -> Poll {
    interrupt::free(|cs| {
        let logger = LOGGER.borrow(cs);
        let mut logger = logger.borrow_mut();
        let logger = logger.as_mut().expect("User has registered a logger");

        let logger = logger.inner.borrow(cs);
        let mut logger = logger.borrow_mut();

        if logger.sink.is_transfer_interrupt() {
            logger.sink.transfer_clear_interrupt();
        }

        if logger.sink.is_transfer_complete() {
            let buffer = logger.sink.transfer_complete().unwrap();
            if !buffer.is_empty() {
                // There's pending data to send
                logger.sink.start_transfer(buffer);
            } else {
                // No pending data; wait for next `log()` call
                logger.buffer = Some(buffer);
            }
        }

        match &logger.buffer {
            Some(_) => Poll::Idle,
            None => Poll::Active,
        }
    })
}

/// Initialize the DMA-based logger with a UART transfer half and a DMA channel
///
/// `tx` should be an `imxrt_hal::uart::Tx` half, obtained by calling `split()` on a
/// configured `UART` peripheral. Returns an error if you've already called `init()`, or if
/// you've already specified a logger through another interface.
///
/// See the [module-level documentation](index.html#example) for an example.
///
/// # 'BYOB' Feature
///
/// "Bring Your Own Buffer" (BYOB) is an optional, compile-time feature. See the [module-level documentation](index.html#byob)
/// for more information.
///
/// If `"byob"` is enabled, the `init()` function signature accepts a fourth argument bound to `buffer`, type `Circular<u8>`:
///
/// ```ignore
/// pub fn init<T>(
///     tx: T,
///     channel: Channel,
///     config: LoggingConfig,
///     buffer: Circular<u8>, // <---- New!
/// ) -> Result<(), SetLoggerError>
/// ```
///
/// The implementation will use this buffer for transferring log messages.
pub fn init<T>(
    tx: T,
    channel: Channel,
    config: LoggingConfig,
    #[cfg(feature = "byob")] buffer: Circular<u8>,
) -> Result<(), SetLoggerError>
where
    T: IntoSink,
{
    let buffer = {
        #[cfg(feature = "byob")]
        {
            buffer
        }
        #[cfg(not(feature = "byob"))]
        {
            Circular::new(&buffer::BUFFER.0).unwrap()
        }
    };

    interrupt::free(move |cs| {
        let logger = LOGGER.borrow(cs);
        let mut logger = logger.borrow_mut();
        if logger.is_none() {
            *logger = Some(Logger {
                inner: Mutex::new(RefCell::new(Inner {
                    sink: tx.into_sink(channel),
                    buffer: Some(buffer),
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

#[cfg(not(feature = "byob"))]
mod buffer {
    use imxrt_hal::dma::Buffer;

    #[repr(align(2048))]
    pub struct Alignment(pub Buffer<[u8; 2048]>);

    pub static BUFFER: Alignment = Alignment(Buffer::new([0; 2048]));
}
