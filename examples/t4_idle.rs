//! DMA-based serial logging - Teensy 4 example
//!
//! Unlike the t4_dma example, this example shows that you can manually
//! add `poll()` points into your code, rather than putting `poll()` in
//! DMA interrupt, to drive serial logging.

#![no_std]
#![no_main]

extern crate panic_halt;

mod demo;

use cortex_m_rt::entry;

const BAUD: u32 = 115_200;

/// See the "BYOB" documentation for more details
#[cfg(feature = "byob")]
mod buffer {
    use imxrt_hal::dma::Buffer;
    pub use imxrt_hal::dma::Circular;

    // Using a 512-byte buffer, rather than the 2KiB default buffer
    #[repr(align(512))]
    pub struct Alignment(pub Buffer<[u8; 512]>);
    pub static BUFFER: Alignment = Alignment(Buffer::new([0; 512]));
}

#[entry]
fn main() -> ! {
    let teensy4_bsp::Peripherals {
        uart,
        mut ccm,
        dcdc,
        gpt1,
        gpt2,
        iomuxc,
        dma,
        ..
    } = teensy4_bsp::Peripherals::take().unwrap();
    let pins = teensy4_bsp::t40::into_pins(iomuxc);

    //
    // DMA initialization
    //
    let mut dma_channels = dma.clock(&mut ccm.handle);
    let channel = dma_channels[7].take().unwrap();

    //
    // UART initialization
    //
    let uarts = uart.clock(
        &mut ccm.handle,
        imxrt_hal::ccm::uart::ClockSelect::OSC,
        imxrt_hal::ccm::uart::PrescalarSelect::DIVIDE_1,
    );
    let uart = uarts.uart2.init(pins.p14, pins.p15, BAUD).unwrap();

    let (tx, _) = uart.split();
    imxrt_uart_log::dma::init(
        tx,
        channel,
        Default::default(),
        #[cfg(feature = "byob")]
        {
            buffer::Circular::new(&buffer::BUFFER.0).unwrap()
        },
    )
    .unwrap();

    demo::log_loop(
        demo::Setup {
            ccm,
            dcdc,
            gpt1,
            gpt2,
            dwt: cortex_m::Peripherals::take().unwrap().DWT,
            led: teensy4_bsp::configure_led(pins.p13),
        },
        |mut gpt| {
            imxrt_uart_log::dma::poll();
            let cycle_count = cortex_m::interrupt::free(|_| {
                demo::cycles(|| {
                    log::error!("I want to guarantee that this is transferred!");
                    while imxrt_uart_log::dma::Poll::Idle != imxrt_uart_log::dma::poll() {}
                })
            });
            log::error!("Flushing the async logger took {} cycles", cycle_count);
            demo::delay(&mut gpt);
        },
    );
}
