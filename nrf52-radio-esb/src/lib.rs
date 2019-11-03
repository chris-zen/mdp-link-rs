#![no_std]

pub mod protocol;

use nrf52_radio::Radio;
use nrf52_radio::states::*;
use nrf52_radio::packet_config::PacketEndianess;
use nrf52_radio::NbResult;

use crate::protocol::Protocol;


pub struct Esb<S> {
  protocol: Protocol,
  radio: Radio<S>,
}

impl<S> Esb<S> {
  pub fn new(radio: Radio<Disabled>) -> Esb<Disabled> {
    Esb {
      protocol: Protocol::FixedPayloadLength(32),
      radio
    }
  }

  pub fn with_radio<F>(self, transform: F) -> Self
    where F: Fn(Radio<S>) -> Radio<S>
  {
    Esb {
      radio: transform(self.radio),
      .. self
    }
  }
}

impl Esb<Disabled> {

  pub fn set_protocol(mut self, protocol: Protocol) -> Self {
    self.protocol = protocol;
    self.with_radio(|radio| {
      match protocol {
        Protocol::DynamicPayloadLength(max_length) => {
          let length_bits = if max_length <= 32 { 6 } else { 8 };
          radio
              .set_packet_length_bits(length_bits)
              .set_packet_s0_exclude()
              .set_packet_s1_include(3)
              .set_packet_payload_max_length(max_length)
              .set_packet_static_length(0)
              .set_packet_endianess(PacketEndianess::BigEndian)
              .set_packet_whiteen_enabled(false)
        },
        Protocol::FixedPayloadLength(length) => {
          radio
              .set_packet_length_bits(0)
              .set_packet_s0_include()
              .set_packet_s1_include(1)
              .set_packet_payload_max_length(length)
              .set_packet_static_length(length)
              .set_packet_endianess(PacketEndianess::BigEndian)
              .set_packet_whiteen_enabled(false)
        },
      }
    })
  }

  pub fn set_crc_disabled(self) -> Self {
    self.with_radio(|radio| radio.set_crc_disabled())
  }

  pub fn set_crc_8bits(self) -> Self {
    self.with_radio(|radio| radio.set_crc_8bits(0xff, 0x107))
  }

  pub fn set_crc_16bits(self) -> Self {
    self.with_radio(|radio| radio.set_crc_16bits(0xffff, 0x11021))
  }

  pub fn start_rx(self, buffer: &mut [u8]) -> NbResult<Esb<Rx>> {
    let protocol = self.protocol;
    self.radio.enable_rx(buffer)
        .into_idle()
        .map(|radio_idle| radio_idle.start_rx())
        .map(|radio_rx| Esb {
          protocol,
          radio: radio_rx,
        })
  }
}

impl<'a> Esb<Rx<'a>> {
  pub fn read_packet(&self) -> NbResult<&[u8]> {
    self.radio.read_packet()
  }
}
