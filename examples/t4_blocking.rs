//! Serial logging - Teensy 4 example
//!
//! Demonstrates how to use the blocking serial interface.
//! Connect a serial receiver to pin 14 of a Teensy 4, and you
//! should see messages and timing measurements. The example
//! uses a GPT timer to demonstrate logging from an interrupt
//! handler.

#![no_std]
#![no_main]

extern crate panic_halt;

mod demo;

use cortex_m_rt::entry;

const BAUD: u32 = 115_200;
const TX_FIFO_SIZE: u8 = 4;

#[entry]
fn main() -> ! {
    let teensy4_bsp::Peripherals {
        uart,
        mut ccm,
        dcdc,
        gpt1,
        gpt2,
        iomuxc,
        ..
    } = teensy4_bsp::Peripherals::take().unwrap();

    let pins = teensy4_bsp::t40::into_pins(iomuxc);
    //
    // UART initialization
    //
    let uarts = uart.clock(
        &mut ccm.handle,
        imxrt_hal::ccm::uart::ClockSelect::OSC,
        imxrt_hal::ccm::uart::PrescalarSelect::DIVIDE_1,
    );
    let mut uart = uarts.uart2.init(pins.p14, pins.p15, BAUD).unwrap();
    uart.set_tx_fifo(core::num::NonZeroU8::new(TX_FIFO_SIZE));

    let (tx, _) = uart.split();
    imxrt_uart_log::blocking::init(tx, Default::default()).unwrap();

    demo::log_loop(
        demo::Setup {
            ccm,
            dcdc,
            gpt1,
            gpt2,
            dwt: cortex_m::Peripherals::take().unwrap().DWT,
            led: teensy4_bsp::configure_led(pins.p13),
        },
        |_| {},
    );
}
