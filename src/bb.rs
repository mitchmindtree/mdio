//! A bit-banged implementation of the MDIO traits.

use crate::{Read, Write};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::timer::{CountDown, Periodic};
use nb::block;

/// A type providing a "bit-banged" MDIO implementation around two given GPIO pins.
///
/// ### Read
///
/// The `mdio::Read` implementation works as follows:
///
/// - Writes the 32-bit preamble.
/// - Writes the 14 most significant bits of the given `ctrl_bits` in MSB order.
/// - Waits for 2 bit times for the turn around.
/// - Reads the 16-bit data using `u16::from_be_bytes`.
///
/// ### Write
///
/// The `mdio::Write` implementation works as follows:
///
/// - Writes the 32-bit preamble.
/// - Writes the 16-bit ctrl value in MSB order.
/// - Writes the 16-bit data in MSB order.
///
/// ## Example
///
/// Here's a rough example of what creating a bit-banged MDIO interface looks like. This code was
/// taken from an application using an STM32F107 MCU to bit-bang a KSZ8863RLL switch.
///
/// ```ignore
/// let mut rcc = device.RCC.constrain();
/// let clocks = rcc.cfgr.sysclk(CYCLE_HZ.hz()).freeze();
/// let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
/// let mut gpioc = device.GPIOC.split(&mut rcc.apb2);
/// let mdio = gpioa.pa2.into_open_drain_output(&mut gpioa.crl);
/// let mdc = gpioc.pc1.into_push_pull_output(&mut gpioc.crl);
/// let timer = hal::timer::Timer::tim3(device.TIM3, &clocks, &mut rcc.apb1).start_count_down(2_500.khz());
/// let mut mdio = mdio::bb::Mdio::new(mdio, mdc, timer);
/// ```
///
/// *Note: The timer used here is 2.5 Mhz. This value was specified in the datasheet for the
/// Ethernet switch. Be sure to change this to suit your application.*
///
/// Now we can read or write using either the `mdio::{Read, Write}` or `mdio::miim::{Read, Write}`
/// traits.
///
/// ```ignore
/// use mdio::miim::{Read, Write};
///
/// let data = mdio.read(phy_addr, reg_addr);
/// mdio.write(phy_addr, reg_addr, data);
/// ```
pub struct Mdio<MdioPin, MdcPin, Clk> {
    /// The data pin.
    mdio: MdioPin,
    /// The clock pin.
    mdc: MdcPin,
    /// The clock used to time bangs.
    clk: Clk,
}

impl<MdioPin, MdcPin, Clk, E> Mdio<MdioPin, MdcPin, Clk>
where
    MdcPin: OutputPin<Error = E>,
    MdioPin: InputPin<Error = E> + OutputPin<Error = E>,
    Clk: CountDown + Periodic,
{
    /// The duration of the preamble in bits.
    const PREAMBLE_BITS: usize = 32;

    /// Create the bit-banged MDIO instance.
    pub fn new(mdio: MdioPin, mdc: MdcPin, clk: Clk) -> Self {
        Self { mdio, mdc, clk }
    }

    /// Split the MDIO bit-banged implementation into its parts.
    pub fn into_parts(self) -> (MdioPin, MdcPin, Clk) {
        let Self { mdio, mdc, clk } = self;
        (mdio, mdc, clk)
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

impl<MdioPin, MdcPin, Clk, E> Read for Mdio<MdioPin, MdcPin, Clk>
where
    MdcPin: OutputPin<Error = E>,
    MdioPin: InputPin<Error = E> + OutputPin<Error = E>,
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

impl<MdioPin, MdcPin, Clk, E> Write for Mdio<MdioPin, MdcPin, Clk>
where
    MdcPin: OutputPin<Error = E>,
    MdioPin: InputPin<Error = E> + OutputPin<Error = E>,
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
