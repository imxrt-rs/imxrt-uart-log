//! Demo code that's shared in each example

use core::time::Duration;
use imxrt_hal::gpt::{OutputCompareRegister, GPT};

/// Output compare register that we'll use for delays
const DELAY_OCR: OutputCompareRegister = OutputCompareRegister::Two;

/// Blocking delay implemented on the GPT timer
pub fn delay(gpt: &mut GPT) {
    use embedded_hal::timer::CountDown;
    let mut cd = gpt.count_down(DELAY_OCR);
    cd.start(Duration::from_millis(1_000));
    while cd.wait().is_err() {
        core::sync::atomic::spin_loop_hint();
    }
}

/// Drop into the common loop that logs data
///
/// `func()` is an operation that will run at the beginning of each
/// loop.
pub fn log_loop<F: Fn(&mut GPT)>(mut gpt: GPT, func: F) -> ! {
    loop {
        func(&mut gpt);
        let (_, duration) = gpt.time(|| {
            log::info!("Hello world! 3 + 2 = {}", 3 + 2);
        });
        log::info!("Logging that took {:?}", duration);
        delay(&mut gpt);

        let (_, duration) = gpt.time(|| {
            log::info!("Hello world! 3 + 2 = 5");
        });
        log::info!("Logging that took {:?}", duration);
        delay(&mut gpt);

        let (_, duration) = gpt.time(|| {
            log::info!("");
        });
        log::info!("Logging that took {:?}", duration);
        delay(&mut gpt);

        let (_, duration) = gpt.time(|| {
            // 100 characters
            log::info!("1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890");
        });
        log::info!("Logging that took {:?}", duration);
        delay(&mut gpt);
    }
}
