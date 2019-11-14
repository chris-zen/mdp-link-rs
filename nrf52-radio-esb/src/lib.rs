#![no_std]

pub mod protocol;

use nrf52_radio::Radio;
use nrf52_radio::packet_config::{S1Length, S1IncludeInRam, Endianess, PacketConfig};
//use nrf52_radio::NbResult;

use crate::protocol::Protocol;


pub struct Esb<'a, A, B> {
  protocol: Protocol,
  pub radio: Radio<'a, A, B>,
}

impl<'a, A, B> Esb<'a, A, B> {
  pub fn new(radio: Radio<'a, A, B>, protocol: Protocol) -> Esb<'a, A, B> {
    // TODO check Radio state
    Self::setup_protocol(&radio, &protocol);
    Esb {
      protocol,
      radio,
    }
  }

  fn setup_protocol(radio: &'a Radio<A, B>, protocol: &Protocol) {
    let pcfn = match protocol {
      Protocol::FixedPayloadLength(length) =>
        PacketConfig::default()
            .with_length_bits(0)
            .with_s0_byte_included(true)
            .with_s1_len(S1Length::Of1Bits)
            .with_s1_include_in_ram(S1IncludeInRam::Automatic)
            .with_max_bytes(*length)
            .with_static_bytes(*length)
            .with_endianess(Endianess::BigEndian)
            .with_whitening_enabled(false),
      Protocol::DynamicPayloadLength(max_length) => {
        let length_bits = if *max_length <= 32 { 6 } else { 8 };
        PacketConfig::default()
            .with_length_bits(length_bits)
            .with_s0_byte_included(false)
            .with_s1_len(S1Length::Of3Bits)
            .with_s1_include_in_ram(S1IncludeInRam::Automatic)
            .with_max_bytes(*max_length)
            .with_static_bytes(0)
            .with_endianess(Endianess::BigEndian)
            .with_whitening_enabled(false)
      }
    };
    radio.set_packet_config(pcfn);
  }

  pub fn set_crc_disabled(&self) -> &Self {
    self.radio.set_crc_disabled();
    self
  }

  pub fn set_crc_8bits(&self) -> &Self {
    self.radio.set_crc_8bits(0xff, 0x107);
    self
  }

  pub fn set_crc_16bits(&self) -> &Self {
    self.radio.set_crc_16bits(0xffff, 0x11021);
    self
  }

//  pub fn buffer()
//  pub fn transmit()
//  pub fn receive()
}
