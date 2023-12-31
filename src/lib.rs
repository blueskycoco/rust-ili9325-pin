#![no_std]

//! Generic parallel GPIO interface for display drivers

use embedded_hal::digital::v2::OutputPin;
use stm32f4xx_hal::pac::{GPIOB};
pub use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use cortex_m_semihosting::hprintln;

type Result<T = ()> = core::result::Result<T, DisplayError>;

/// Parallel 16 Bit communication interface
///
/// This interface implements a 16-Bit "8080" style write-only display interface using any
/// 16-bit [OutputBus] implementation as well as one
/// `OutputPin` for the data/command selection and one `OutputPin` for the write-enable flag.
///
/// All pins are supposed to be high-active, high for the D/C pin meaning "data" and the
/// write-enable being pulled low before the setting of the bits and supposed to be sampled at a
/// low to high edge.
pub struct PGPIO16BitInterface<DC, WR, CS, RD> {
    gpio: GPIOB,
    dc: DC,
    wr: WR,
    cs: CS,
    rd: RD,
}

impl<DC, WR, CS, RD> PGPIO16BitInterface<DC, WR, CS, RD>
where
    DC: OutputPin,
    WR: OutputPin,
    CS: OutputPin,
    RD: OutputPin,
{
    /// Create new parallel GPIO interface for communication with a display driver
    pub fn new(gpio: GPIOB, mut dc: DC, mut wr: WR, mut cs: CS, mut rd: RD) -> Self {
        //read id first
        gpio.moder.write(|w| unsafe { w.bits(0x55555555) });
        gpio.pupdr.write(|w| unsafe { w.bits(0x55555555) });
        gpio.ospeedr.write(|w| unsafe { w.bits(0xffffffff) });
        hprintln!("moder: {:#x}", gpio.moder.read().bits());
        cs.set_low().map_err(|_| DisplayError::DCError);
        dc.set_low().map_err(|_| DisplayError::DCError);
        rd.set_high().map_err(|_| DisplayError::DCError);
        wr.set_low().map_err(|_| DisplayError::BusWriteError);
        gpio.odr.write(|w| unsafe { w.bits(0x00 as u32) } );
        wr.set_high().map_err(|_| DisplayError::BusWriteError);
        cs.set_high().map_err(|_| DisplayError::DCError);
        
        gpio.moder.write(|w| unsafe { w.bits(0x00 as u32) });
        hprintln!("moder: {:#x}", gpio.moder.read().bits());
        cs.set_low().map_err(|_| DisplayError::DCError);
        dc.set_high().map_err(|_| DisplayError::DCError);
        wr.set_high().map_err(|_| DisplayError::BusWriteError);
        rd.set_low().map_err(|_| DisplayError::DCError);
        hprintln!("ili9325 id: {:#x}", gpio.idr.read().bits());
        gpio.moder.write(|w| unsafe { w.bits(0x55555555) });
        hprintln!("moder: {:#x}", gpio.moder.read().bits());
        rd.set_high().map_err(|_| DisplayError::DCError);
        cs.set_high().map_err(|_| DisplayError::DCError);
        Self { gpio, dc, wr, cs, rd }
    }

    /// Consume the display interface and return
    /// the bus and GPIO pins used by it
    pub fn release(self) -> (DC, WR, CS, RD) {
        (self.dc, self.wr, self.cs, self.rd)
    }

    fn write_iter(&mut self, iter: impl Iterator<Item = u16>) -> Result {
        for value in iter {
            self.cs.set_low().map_err(|_| DisplayError::DCError);
            self.wr.set_low().map_err(|_| DisplayError::BusWriteError)?;
//            hprintln!("value w is {:#x}", value);
            self.gpio.odr.write(|w| unsafe { w.bits(value as u32) } );
//            hprintln!("value r is {:#x}", self.gpio.odr.read().bits());
            self.wr.set_high().map_err(|_| DisplayError::BusWriteError)?;
            self.cs.set_high().map_err(|_| DisplayError::DCError);
        }

        Ok(())
    }

    fn write_data(&mut self, data: DataFormat<'_>) -> Result {
        match data {
            DataFormat::U8(slice) => self.write_iter(slice.iter().copied().map(u16::from)),
            DataFormat::U8Iter(iter) => self.write_iter(iter.map(u16::from)),
            DataFormat::U16(slice) => self.write_iter(slice.iter().copied()),
            DataFormat::U16BE(slice) => self.write_iter(slice.iter().copied()),
            DataFormat::U16LE(slice) => self.write_iter(slice.iter().copied()),
            DataFormat::U16BEIter(iter) => self.write_iter(iter),
            DataFormat::U16LEIter(iter) => self.write_iter(iter),
            _ => Err(DisplayError::DataFormatNotImplemented),
        }
    }
}

impl<DC, WR, CS, RD> WriteOnlyDataCommand for PGPIO16BitInterface<DC, WR, CS, RD>
where
    DC: OutputPin,
    WR: OutputPin,
    CS: OutputPin,
    RD: OutputPin,
{
    fn send_commands(&mut self, cmds: DataFormat<'_>) -> Result {
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;
        self.write_data(cmds)
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result {
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;
        self.write_data(buf)
    }
}
