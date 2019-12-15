#![no_std]

#[macro_use]
extern crate bitflags;

use nrf52840_hal as hal;

pub use radio::{AsyncResult, Error, Radio, RadioExt, Result};

mod values_as_enum;
pub mod tx_power;
pub mod mode;
pub mod packet_config;
pub mod base_address;
pub mod frequency;
pub mod logical_address;
pub mod rx_addresses;
pub mod shortcuts;
pub mod states;
pub mod radio;

