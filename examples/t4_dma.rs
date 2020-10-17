//! DMA-based serial logging - Teensy 4 example
//!
//! This use the same setup as the `t4_uart.rs` example. Connect
//! a serial receive to pin 14, and you should receive log messages
//! and timing measurements.

#![no_std]
#![no_main]

extern crate panic_halt;

mod demo;

use cortex_m_rt::entry;
use cortex_m_rt::interrupt;
use imxrt_hal::ral::interrupt;

const BAUD: u32 = 115_200;

#[interrupt]
fn DMA7_DMA23() {
    imxrt_uart_log::dma::poll();
}

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

    //
    // DMA initialization
    //
    let mut dma_channels = dma.clock(&mut ccm.handle);
    let mut channel = dma_channels[7].take().unwrap();
    channel.set_interrupt_on_completion(true);
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::DMA7_DMA23);
    }

    //
    // UART initialization
    //
    let uarts = uart.clock(
        &mut ccm.handle,
        imxrt_hal::ccm::uart::ClockSelect::OSC,
        imxrt_hal::ccm::uart::PrescalarSelect::DIVIDE_1,
    );
    let uart = uarts
        .uart2
        .init(iomuxc.ad_b1.p02, iomuxc.ad_b1.p03, BAUD)
        .unwrap();

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
        },
        |_| {},
    );
}
