//! Logging sink

use core::fmt;
use imxrt_hal::uart;

use embedded_hal::blocking::serial::Write;

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

/// A logging sink which dispatches to any of the eight possible UART peripherals
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
    pub(super) fn flush(&mut self) {
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
