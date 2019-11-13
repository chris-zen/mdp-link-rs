#![no_std]

use nrf52840_hal as hal;
use nb;

mod values_as_enum;
pub mod tx_power;
pub mod mode;
pub mod packet_config;
pub mod base_address;
pub mod frequency;
pub mod rx_addresses;
pub mod states;
pub mod radio;

pub use radio::{Radio, RadioExt};

pub type NbResult<T> = nb::Result<T, nb::Error<()>>;
