//! DMA buffer writer

use core::fmt;
use imxrt_hal::dma::{Circular, WriteHalf};

pub enum Writer<'a> {
    Circular(&'a mut Circular<u8>),
    WriteHalf(&'a mut WriteHalf<'a, u8>),
}

impl<'a> fmt::Write for Writer<'a> {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        match self {
            Writer::Circular(circular) => circular.insert(string.as_bytes().iter().copied()),
            Writer::WriteHalf(write_half) => write_half.insert(string.as_bytes().iter().copied()),
        };
        Ok(())
    }
}
