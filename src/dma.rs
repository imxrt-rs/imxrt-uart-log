//! Asynchronous, DMA-based logging

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
/// Users *must* call this from a DMA interrupt! Otherwise, serial logging will do nothing.
///
/// # Panics
///
/// If a user calls `on_interrupt()` from anywhere that's *not* an interrupt, `on_interrupt()` panics.
#[inline]
pub fn on_interrupt() {
    // TODO panic checks
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

pub fn init_async<T>(tx: T, channel: Channel, config: LoggingConfig) -> Result<(), SetLoggerError>
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
