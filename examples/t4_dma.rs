//! DMA-based serial logging - Teensy 4 example
//!
//! This use the same setup as the `t4_uart.rs` example. Connect
//! a serial receive to pin 14, and you should receive log messages
//! and timing measurements.
//!
//! Unlike the `t4_uart.rs` example, this example does not demonstrate
//! logging from an interrupt. Logging from an interrupt is not supported
//! with the DMA-based logger.

#![no_std]
#![no_main]

extern crate panic_halt;
#[cfg(target_arch = "arm")]
extern crate teensy4_fcb;

use core::time::Duration;
use imxrt_hal::gpt;
use imxrt_hal::ral::interrupt;
use teensy4_rt::entry;
use teensy4_rt::interrupt;

const BAUD: u32 = 115_200;

/// Output compare register that we'll use for delays
const DELAY_OCR: gpt::OutputCompareRegister = gpt::OutputCompareRegister::Two;

#[interrupt]
fn DMA7_DMA23() {
    imxrt_uart_log::dma::poll()
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
    // DMA initialization
    //
    let mut dma_channels = peripherals.dma.clock(&mut peripherals.ccm.handle);
    let channel = dma_channels[7].take().unwrap();
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::DMA7_DMA23);
    }

    //
    // UART initialization
    //
    let uarts = peripherals.uart.clock(
        &mut peripherals.ccm.handle,
        imxrt_hal::ccm::uart::ClockSelect::OSC,
        imxrt_hal::ccm::uart::PrescalarSelect::DIVIDE_1,
    );
    let uart = uarts
        .uart2
        .init(
            peripherals.iomuxc.gpio_ad_b1_02.alt2(),
            peripherals.iomuxc.gpio_ad_b1_03.alt2(),
            BAUD,
        )
        .unwrap();

    let (tx, _) = uart.split();
    imxrt_uart_log::dma::init(tx, channel, Default::default()).unwrap();

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

    let delay = |gpt: &mut gpt::GPT| {
        use embedded_hal::timer::CountDown;
        let mut cd = gpt.count_down(DELAY_OCR);
        cd.start(Duration::from_millis(1_000));
        while cd.wait().is_err() {
            core::sync::atomic::spin_loop_hint();
        }
    };

    loop {
        let (_, duration) = gpt2.time(|| {
            log::info!("Hello world! 3 + 2 = {}", 3 + 2);
        });
        log::info!("Logging that took {:?}", duration);
        delay(&mut gpt2);

        let (_, duration) = gpt2.time(|| {
            log::info!("Hello world! 3 + 2 = 5");
        });
        log::info!("Logging that took {:?}", duration);
        delay(&mut gpt2);

        let (_, duration) = gpt2.time(|| {
            log::info!("");
        });
        log::info!("Logging that took {:?}", duration);
        delay(&mut gpt2);

        let (_, duration) = gpt2.time(|| {
            // 100 characters
            log::info!("1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890");
        });
        log::info!("Logging that took {:?}", duration);
        delay(&mut gpt2);
    }
}
