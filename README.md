# mdio [![Actions Status](https://github.com/mitchmindtree/mdio/workflows/mdio/badge.svg)](https://github.com/mitchmindtree/mdio/actions) [![Crates.io](https://img.shields.io/crates/v/mdio.svg)](https://crates.io/crates/mdio) [![Crates.io](https://img.shields.io/crates/l/mdio.svg)](https://github.com/mitchmindtree/mdio/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/mdio/badge.svg)](https://docs.rs/mdio/)

An API to simplify Management Data Input/Output (MDIO) interface access,
including an implementation of the standard Media Independent Interface
Management (MIIM) protocol.

## Terminology

In the general discourse, MDIO and MIIM are often used to refer to the same
thing. In this crate, MDIO is used to refer to the two-pin (MDIO, MDC) I/O
interface, while MIIM is used to refer to the standard MII Management protocol
that is communicated via the MDIO pins.

By distinguishing between the two in this manner, we create room in the API for
both the standard MIIM protocol while also allowing to support non-standard
protocols on the MDIO interface, a practise that is not uncommon on some
Ethernet switches.

## The `mdio::Read` and `mdio::Write` traits.

These should be implemented for interfaces where, given 16 control bits, an MDIO
operation can be performed.

These traits are designed to allow MDIO interfaces to support both standard MIIM
as well as custom variations on the protocol. For example, it is not uncommon
for some Ethernet switches to support extended register access by modifying the
standard MIIM protocol slightly, i.e.  setting the Op code bits to `0b00`, or
using some of the PHY address bits to increase the number of register addresses.

Note that it may not be possible to implement these traits directly for some
MCUs in a manner that allows for both using the integrated MAC while *also*
supporting customised protocols. This is because some MCUs provide limited MAC
interfaces that only support the standard MIIM protocol. In these cases, while
not ideal, the traits can be implemented for "bit-banging" interfaces instead.
This crate provides one such interface via the `bitbang` feature.

## Features

- `bitbang`: Enables the `bb` module along with an `bb::Mdio` type providing a
  bit-banged implementation of the `mdio::Read` and `mdio::Write` traits.

All of these features are **opt-in** and disabled by default.
