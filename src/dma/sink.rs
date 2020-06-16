//! DMA sink

use imxrt_hal::{
    dma::{Channel, Circular, Config, ConfigBuilder, Peripheral, WriteHalf},
    uart::{module, Tx},
};

/// DMA output
type Output<M> = Peripheral<Tx<M>, u8, Circular<u8>>;

pub enum Sink {
    _1(Output<module::_1>),
    _2(Output<module::_2>),
    _3(Output<module::_3>),
    _4(Output<module::_4>),
    _5(Output<module::_5>),
    _6(Output<module::_6>),
    _7(Output<module::_7>),
    _8(Output<module::_8>),
}

impl Sink {
    pub fn is_transfer_interrupt(&self) -> bool {
        match self {
            Sink::_1(periph) => periph.is_transfer_interrupt(),
            Sink::_2(periph) => periph.is_transfer_interrupt(),
            Sink::_3(periph) => periph.is_transfer_interrupt(),
            Sink::_4(periph) => periph.is_transfer_interrupt(),
            Sink::_5(periph) => periph.is_transfer_interrupt(),
            Sink::_6(periph) => periph.is_transfer_interrupt(),
            Sink::_7(periph) => periph.is_transfer_interrupt(),
            Sink::_8(periph) => periph.is_transfer_interrupt(),
        }
    }

    pub fn transfer_clear_interrupt(&mut self) {
        match self {
            Sink::_1(periph) => periph.transfer_clear_interrupt(),
            Sink::_2(periph) => periph.transfer_clear_interrupt(),
            Sink::_3(periph) => periph.transfer_clear_interrupt(),
            Sink::_4(periph) => periph.transfer_clear_interrupt(),
            Sink::_5(periph) => periph.transfer_clear_interrupt(),
            Sink::_6(periph) => periph.transfer_clear_interrupt(),
            Sink::_7(periph) => periph.transfer_clear_interrupt(),
            Sink::_8(periph) => periph.transfer_clear_interrupt(),
        }
    }

    pub fn transfer_complete(&mut self) -> Option<Circular<u8>> {
        match self {
            Sink::_1(periph) => periph.transfer_complete(),
            Sink::_2(periph) => periph.transfer_complete(),
            Sink::_3(periph) => periph.transfer_complete(),
            Sink::_4(periph) => periph.transfer_complete(),
            Sink::_5(periph) => periph.transfer_complete(),
            Sink::_6(periph) => periph.transfer_complete(),
            Sink::_7(periph) => periph.transfer_complete(),
            Sink::_8(periph) => periph.transfer_complete(),
        }
    }

    pub fn start_transfer(&mut self, buffer: Circular<u8>) {
        match self {
            Sink::_1(periph) => periph
                .start_transfer(buffer)
                .expect("Start transfer UART1 failed"),
            Sink::_2(periph) => periph
                .start_transfer(buffer)
                .expect("Start transfer UART2 failed"),
            Sink::_3(periph) => periph
                .start_transfer(buffer)
                .expect("Start transfer UART3 failed"),
            Sink::_4(periph) => periph
                .start_transfer(buffer)
                .expect("Start transfer UART4 failed"),
            Sink::_5(periph) => periph
                .start_transfer(buffer)
                .expect("Start transfer UART5 failed"),
            Sink::_6(periph) => periph
                .start_transfer(buffer)
                .expect("Start transfer UART6 failed"),
            Sink::_7(periph) => periph
                .start_transfer(buffer)
                .expect("Start transfer UART7 failed"),
            Sink::_8(periph) => periph
                .start_transfer(buffer)
                .expect("Start transfer UART8 failed"),
        }
    }

    pub fn write_half(&mut self) -> Option<WriteHalf<u8>> {
        match self {
            Sink::_1(periph) => periph.write_half(),
            Sink::_2(periph) => periph.write_half(),
            Sink::_3(periph) => periph.write_half(),
            Sink::_4(periph) => periph.write_half(),
            Sink::_5(periph) => periph.write_half(),
            Sink::_6(periph) => periph.write_half(),
            Sink::_7(periph) => periph.write_half(),
            Sink::_8(periph) => periph.write_half(),
        }
    }
}

pub trait IntoSink {
    fn into_sink(self, channel: Channel) -> Sink;
}

fn config() -> Config {
    ConfigBuilder::new().interrupt_on_completion(true).build()
}

impl IntoSink for Tx<module::_1> {
    fn into_sink(self, channel: Channel) -> Sink {
        Sink::_1(Peripheral::new_transfer(self, channel, config()))
    }
}

impl IntoSink for Tx<module::_2> {
    fn into_sink(self, channel: Channel) -> Sink {
        Sink::_2(Peripheral::new_transfer(self, channel, config()))
    }
}

impl IntoSink for Tx<module::_3> {
    fn into_sink(self, channel: Channel) -> Sink {
        Sink::_3(Peripheral::new_transfer(self, channel, config()))
    }
}

impl IntoSink for Tx<module::_4> {
    fn into_sink(self, channel: Channel) -> Sink {
        Sink::_4(Peripheral::new_transfer(self, channel, config()))
    }
}

impl IntoSink for Tx<module::_5> {
    fn into_sink(self, channel: Channel) -> Sink {
        Sink::_5(Peripheral::new_transfer(self, channel, config()))
    }
}

impl IntoSink for Tx<module::_6> {
    fn into_sink(self, channel: Channel) -> Sink {
        Sink::_6(Peripheral::new_transfer(self, channel, config()))
    }
}

impl IntoSink for Tx<module::_7> {
    fn into_sink(self, channel: Channel) -> Sink {
        Sink::_7(Peripheral::new_transfer(self, channel, config()))
    }
}

impl IntoSink for Tx<module::_8> {
    fn into_sink(self, channel: Channel) -> Sink {
        Sink::_8(Peripheral::new_transfer(self, channel, config()))
    }
}
