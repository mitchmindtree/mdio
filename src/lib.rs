//! An API to simplify Management Data Input/Output (MDIO) interface access, including an
//! implementation of the standard Media Independent Interface Management (MIIM) protocol.
//!
//! ## Terminology
//!
//! In the general discourse, MDIO and MIIM are often used to refer to the same thing. In this
//! crate, MDIO is used to refer to the two-pin (MDIO, MDC) I/O interface, while MIIM is used to
//! refer to the standard MII Management protocol that is communicated via the MDIO pins.
//!
//! By distinguishing between the two in this manner, we create room in the API for both the
//! standard MIIM protocol while also allowing to support non-standard protocols on the MDIO
//! interface, a practise that is not uncommon on some Ethernet switches.
//!
//! ## The `mdio::Read` and `mdio::Write` traits.
//!
//! These should be implemented for interfaces where, given 16 control bits, an MDIO operation can
//! be performed.
//!
//! These traits are designed to allow MDIO interfaces to support both standard MIIM as well as
//! custom variations on the protocol. For example, it is not uncommon for some Ethernet switches
//! to support extended register access by modifying the standard MIIM protocol slightly, i.e.
//! setting the Op code bits to `0b00`, or using some of the PHY address bits to increase the
//! number of register addresses.
//!
//! Note that it may not be possible to implement these traits directly for some MCUs in a manner
//! that allows for both using the integrated MAC while *also* supporting customised protocols.
//! This is because some MCUs provide limited MAC interfaces that only support the standard MIIM
//! protocol. In these cases, while not ideal, the traits can be implemented for "bit-banging"
//! interfaces instead.

#[cfg(feature = "bitbang")]
pub mod bb;
pub mod miim;

/// Performing read operations via an MDIO interface.
pub trait Read {
    /// Errors that might occur during MDIO operation.
    type Error;
    /// Given the 16 control bits, perform an MDIO read operation.
    ///
    /// ## MIIM
    ///
    /// In the standard MIIM protocol, the `ctrl_bits` should contain the following in order:
    ///
    /// - Start of frame (2 bits) `01`.
    /// - Op code (2 bits) `10`.
    /// - PHY address (5 bits)
    /// - Register address (5 bits)
    /// - Turn around (2 bits, MDIO line is released).
    ///
    /// See the `miim::Read` trait.
    fn read(&mut self, ctrl_bits: u16) -> Result<u16, Self::Error>;
}

/// Performing write operations via an MDIO interface.
pub trait Write {
    /// Errors that might occur during MDIO operation.
    type Error;
    /// Given the 16 control bits and 16 bits of data, perform an MDIO write operation.
    ///
    /// ## MIIM
    ///
    /// In the standard MIIM protocol, the `ctrl_bits` should contain the following in order:
    ///
    /// - Start of frame (2 bits) `01`.
    /// - Op code (2 bits) `01`.
    /// - PHY address (5 bits)
    /// - Register address (5 bits)
    /// - Turn around (2 bits) `10`.
    ///
    /// See the `miim::Write` trait.
    fn write(&mut self, ctrl_bits: u16, data_bits: u16) -> Result<(), Self::Error>;
}
