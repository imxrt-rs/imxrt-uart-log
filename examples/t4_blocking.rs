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
#[cfg(target_arch = "arm")]
extern crate teensy4_fcb;

mod demo;

use core::time::Duration;
use imxrt_hal::gpt;
use imxrt_hal::ral::interrupt;
use teensy4_rt::entry;
use teensy4_rt::interrupt;

const BAUD: u32 = 115_200;
const TX_FIFO_SIZE: u8 = 4;

static mut TIMER: Option<gpt::GPT> = None;

/// GPT output compare register selection
const INTERRUPT_OCR: gpt::OutputCompareRegister = gpt::OutputCompareRegister::Three;
const INTERRUPT_PERIOD: Duration = Duration::from_millis(850);

#[interrupt]
unsafe fn GPT1() {
    let gpt1 = TIMER.as_mut().unwrap();
    gpt1.output_compare_status(INTERRUPT_OCR).clear();
    gpt1.set_enable(false);
    log::warn!("Called from interrupt!");
    gpt1.set_output_compare_duration(INTERRUPT_OCR, INTERRUPT_PERIOD);
    gpt1.set_enable(true);
}

#[entry]
fn main() -> ! {
    let mut peripherals = imxrt_hal::Peripherals::take().unwrap();

    let (_, ipg_hz) = peripherals.ccm.pll1.set_arm_clock(
        imxrt_hal::ccm::PLL1::ARM_HZ,
        &mut peripherals.ccm.handle,
        &mut peripherals.dcdc,
    );

    //
    // UART initialization
    //
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
            BAUD,
        )
        .unwrap();
    uart.set_tx_fifo(core::num::NonZeroU8::new(TX_FIFO_SIZE));

    let (tx, _) = uart.split();
    imxrt_uart_log::blocking::init(tx, Default::default()).unwrap();

    //
    // GPT2 initialization (for timing how long logging takes)
    //
    let mut cfg = peripherals.ccm.perclk.configure(
        &mut peripherals.ccm.handle,
        imxrt_hal::ccm::perclk::PODF::DIVIDE_3,
        imxrt_hal::ccm::perclk::CLKSEL::IPG(ipg_hz),
    );

    let mut gpt2 = peripherals.gpt2.clock(&mut cfg);
    gpt2.set_mode(imxrt_hal::gpt::Mode::FreeRunning);
    gpt2.set_enable(true);

    //
    // GPT1 initialization (for demonstrating logging in an interrupt)
    //
    let mut gpt1 = peripherals.gpt1.clock(&mut cfg);
    gpt1.set_output_interrupt_on_compare(INTERRUPT_OCR, true);
    gpt1.set_wait_mode_enable(true);
    gpt1.set_mode(imxrt_hal::gpt::Mode::FreeRunning);

    gpt1.set_enable(false);
    gpt1.set_output_compare_duration(INTERRUPT_OCR, INTERRUPT_PERIOD);
    gpt1.set_enable(true);

    unsafe {
        TIMER = Some(gpt1);
        cortex_m::peripheral::NVIC::unmask(interrupt::GPT1);
    }

    demo::log_loop(gpt2);
}
