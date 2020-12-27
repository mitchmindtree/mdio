//! MII Management (MIIM) Interface.
//!
//! The IEEE 802.3 MII Management Interface. Allows for upper-layer devices to monitor and control
//! the state of one or more PHYs.
//!
//! Each of the 8 16-bit registers are indexed via a 5-bit address, preceded by a 5-bit PHY address.
//!
//! A blanket implementation of the `mdio::miim::{Read, Write}` traits is provided for types
//! implementing the `mdio::{Read, Write}` traits.

/// A trait for reading the standard MIIM protocol.
///
/// A blanket implementation is provided for types implementing the lower-level `mdio::Read` trait.
pub trait Read {
    /// Errors that might occur on the MIIM interface.
    type Error;
    /// Read the data from the given register address associated with the specified PHY.
    fn read(&mut self, phy_addr: u8, reg_addr: u8) -> Result<u16, Self::Error>;
}

/// A trait for writing the standard MIIM protocol.
///
/// A blanket implementation is provided for types implementing the lower-level `mdio::Write` trait.
pub trait Write {
    /// Errors that might occur on the MIIM interface.
    type Error;
    /// Write to the register at the given address associated with the specified PHY.
    fn write(&mut self, phy_addr: u8, reg_addr: u8, data: u16) -> Result<(), Self::Error>;
}

impl<T> Read for T
where
    T: crate::Read,
{
    type Error = T::Error;
    fn read(&mut self, phy_addr: u8, reg_addr: u8) -> Result<u16, Self::Error> {
        crate::Read::read(self, read_ctrl_bits(phy_addr, reg_addr))
    }
}

impl<T> Write for T
where
    T: crate::Write,
{
    type Error = T::Error;
    fn write(&mut self, phy_addr: u8, reg_addr: u8, data: u16) -> Result<(), Self::Error> {
        crate::Write::write(self, write_ctrl_bits(phy_addr, reg_addr), data)
    }
}

fn phy_addr_ctrl_bits(phy_addr: u8) -> u16 {
    const PHY_ADDR_OFFSET: u16 = 7;
    ((phy_addr & 0b00011111) as u16) << PHY_ADDR_OFFSET
}

fn reg_addr_ctrl_bits(reg_addr: u8) -> u16 {
    const REG_ADDR_OFFSET: u16 = 2;
    ((reg_addr & 0b00011111) as u16) << REG_ADDR_OFFSET
}

/// Given the PHY and register addresses, produce the control bits for an MDIO read operation.
pub fn read_ctrl_bits(phy_addr: u8, reg_addr: u8) -> u16 {
    const READ_CTRL_BITS: u16 = 0b0110_00000_00000_00;
    READ_CTRL_BITS | phy_addr_ctrl_bits(phy_addr) | reg_addr_ctrl_bits(reg_addr)
}

/// Given the PHY and register addresses, produce the control bits for an MDIO write operation.
pub fn write_ctrl_bits(phy_addr: u8, reg_addr: u8) -> u16 {
    const WRITE_CTRL_BITS: u16 = 0b0101_00000_00000_10;
    WRITE_CTRL_BITS | phy_addr_ctrl_bits(phy_addr) | reg_addr_ctrl_bits(reg_addr)
}
