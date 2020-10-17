//! Demo code that's shared in each example

use core::time::Duration;
use cortex_m::peripheral::DWT;
use cortex_m_rt::interrupt;
use imxrt_hal::gpt;
use imxrt_hal::gpt::{OutputCompareRegister, GPT};
use imxrt_hal::ral::interrupt;

/// Output compare register that we'll use for delays
const DELAY_OCR: OutputCompareRegister = OutputCompareRegister::Two;

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

/// Blocking delay implemented on the GPT timer
pub fn delay(gpt: &mut GPT) {
    use embedded_hal::timer::CountDown;
    let mut cd = gpt.count_down(DELAY_OCR);
    cd.start(Duration::from_millis(1_000));
    while cd.wait().is_err() {
        core::sync::atomic::spin_loop_hint();
    }
}

pub struct Setup {
    pub ccm: imxrt_hal::ccm::CCM,
    pub led: teensy4_bsp::LED,
    pub dcdc: imxrt_hal::dcdc::DCDC,
    pub gpt1: imxrt_hal::gpt::Unclocked,
    pub gpt2: imxrt_hal::gpt::Unclocked,
    pub dwt: DWT,
}

/// Count the clock cycles required to execute `f`
#[inline(always)]
pub fn cycles<F: FnOnce()>(f: F) -> u32 {
    let start = DWT::get_cycle_count();
    f();
    let end = DWT::get_cycle_count();
    end - start
}

/// Drop into the common loop that logs data
///
/// `func()` is an operation that will run at the beginning of each
/// loop.
pub fn log_loop<F: Fn(&mut GPT)>(mut setup: Setup, func: F) -> ! {
    DWT::unlock();
    setup.dwt.enable_cycle_counter();

    let (_, ipg_hz) = setup.ccm.pll1.set_arm_clock(
        imxrt_hal::ccm::PLL1::ARM_HZ,
        &mut setup.ccm.handle,
        &mut setup.dcdc,
    );

    //
    // GPT2 initialization (for timing how long logging takes)
    //
    let mut cfg = setup.ccm.perclk.configure(
        &mut setup.ccm.handle,
        imxrt_hal::ccm::perclk::PODF::DIVIDE_3,
        imxrt_hal::ccm::perclk::CLKSEL::IPG(ipg_hz),
    );

    let mut gpt2 = setup.gpt2.clock(&mut cfg);
    gpt2.set_mode(imxrt_hal::gpt::Mode::FreeRunning);
    gpt2.set_enable(true);

    //
    // GPT1 initialization (for demonstrating logging in an interrupt)
    //
    let mut gpt1 = setup.gpt1.clock(&mut cfg);
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

    loop {
        func(&mut gpt2);
        let count = cycles(|| {
            log::info!("Hello world! 3 + 2 = {}", 3 + 2);
        });
        log::info!("Logging that took {} cycles", count);
        delay(&mut gpt2);
        setup.led.toggle();

        let count = cycles(|| {
            log::info!("Hello world! 3 + 2 = 5");
        });
        log::info!("Logging that took {} cycles", count);
        delay(&mut gpt2);
        setup.led.toggle();

        let count = cycles(|| {
            log::info!("");
        });
        log::info!("Logging that took {} cycles", count);
        delay(&mut gpt2);
        setup.led.toggle();

        let count = cycles(|| {
            // 100 characters
            log::info!("1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890");
        });
        log::info!("Logging that took {} cycles", count);
        delay(&mut gpt2);
        setup.led.toggle();
    }
}
