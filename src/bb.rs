use crate::{Read, Write};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::timer::{CountDown, Periodic};
use nb::block;

/// A type providing a "bit-banged" MDIO implementation around two given GPIO pins.
pub struct MdioBb<Mdio, Mdc, Clk> {
    /// The data pin.
    mdio: Mdio,
    /// The clock pin.
    mdc: Mdc,
    /// The clock used to time bangs.
    clk: Clk,
}

impl<Mdio, Mdc, Clk, E> MdioBb<Mdio, Mdc, Clk>
where
    Mdc: OutputPin<Error = E>,
    Mdio: InputPin<Error = E> + OutputPin<Error = E>,
    Clk: CountDown + Periodic,
{
    /// The duration of the preamble in bits.
    const PREAMBLE_BITS: usize = 32;

    /// Create the bit-banged MDIO instance.
    pub fn new(mdio: Mdio, mdc: Mdc, clk: Clk) -> Self {
        Self { mdio, mdc, clk }
    }

    fn wait_for_clk(&mut self) {
        block!(self.clk.wait()).ok();
    }

    fn pulse_clock(&mut self) -> Result<(), E> {
        self.wait_for_clk();
        self.mdc.set_high()?;
        self.wait_for_clk();
        self.mdc.set_low()
    }

    fn preamble(&mut self) -> Result<(), E> {
        self.mdio.set_high()?;
        for _bit in 0..Self::PREAMBLE_BITS {
            self.pulse_clock()?;
        }
        Ok(())
    }

    fn write_bit(&mut self, bit: bool) -> Result<(), E> {
        if bit {
            self.mdio.set_high()?;
        } else {
            self.mdio.set_low()?;
        }
        self.pulse_clock()
    }

    /// Write `count` number of most significant bits in the given byte.
    fn write_bits(&mut self, byte: u8, count: usize) -> Result<(), E> {
        for bit_offset in 0..core::cmp::min(count, 8) {
            self.write_bit(bit_is_set(byte, bit_offset))?;
        }
        Ok(())
    }

    /// Write each bit in the byte, starting with the most significant.
    fn write_u8(&mut self, byte: u8) -> Result<(), E> {
        for bit_offset in 0..8 {
            self.write_bit(bit_is_set(byte, bit_offset))?;
        }
        Ok(())
    }

    /// Write both bytes in the given `u16`, starting with the most significant.
    fn write_u16(&mut self, bytes: u16) -> Result<(), E> {
        for &byte in &bytes.to_be_bytes() {
            self.write_u8(byte)?;
        }
        Ok(())
    }

    /// Wait for the turnaround before reading.
    fn turnaround(&mut self) -> Result<(), E> {
        // TODO: Is anything needed to release Mdio pin here?
        self.pulse_clock()?;
        self.pulse_clock()
    }

    fn read_bit(&mut self) -> Result<bool, E> {
        let b = self.mdio.is_high()?;
        self.pulse_clock()?;
        Ok(b)
    }

    fn read_byte(&mut self) -> Result<u8, E> {
        let mut byte = 0u8;
        for bit_offset in 0..8 {
            if self.read_bit()? {
                set_bit(&mut byte, bit_offset);
            }
        }
        Ok(byte)
    }

    fn read_u16(&mut self) -> Result<u16, E> {
        let a = self.read_byte()?;
        let b = self.read_byte()?;
        let u = u16::from_be_bytes([a, b]);
        Ok(u)
    }
}

impl<Mdio, Mdc, Clk, E> Read for MdioBb<Mdio, Mdc, Clk>
where
    Mdc: OutputPin<Error = E>,
    Mdio: InputPin<Error = E> + OutputPin<Error = E>,
    Clk: CountDown + Periodic,
{
    type Error = E;
    fn read(&mut self, ctrl_bits: u16) -> Result<u16, Self::Error> {
        self.preamble()?;
        let [ctrl_a, ctrl_b] = ctrl_bits.to_be_bytes();
        self.write_u8(ctrl_a)?;
        self.write_bits(ctrl_b, 6)?;
        self.turnaround()?;
        self.read_u16()
    }
}

impl<Mdio, Mdc, Clk, E> Write for MdioBb<Mdio, Mdc, Clk>
where
    Mdc: OutputPin<Error = E>,
    Mdio: InputPin<Error = E> + OutputPin<Error = E>,
    Clk: CountDown + Periodic,
{
    type Error = E;
    fn write(&mut self, ctrl_bits: u16, data_bits: u16) -> Result<(), Self::Error> {
        self.preamble()?;
        self.write_u16(ctrl_bits)?;
        self.write_u16(data_bits)
    }
}

/// Whether or not the bit at the given index is set.
///
/// Assumes a `bit_index` in the range `0..8`, where `0` is the most significant bit in the byte,
/// `7` is the least significant.
fn bit_is_set(byte: u8, bit_index: usize) -> bool {
    let out_bit = (byte >> (7 - bit_index)) & 0b1;
    out_bit == 1
}

/// Set the bit at the given index.
///
/// Assumes a `bit_index` in the range `0..8`, where `0` is the most significant bit in the byte,
/// `7` is the least significant.
fn set_bit(byte: &mut u8, bit_index: usize) {
    *byte |= 1 << (7 - bit_index);
}
