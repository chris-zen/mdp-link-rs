/*!

nrf52840 Radio

See [Product Specification](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.0.pdf): 6.20 RADIO — 2.4 GHz radio

*/

use crate::hal::target::RADIO;
use crate::states::*;
use crate::tx_power::TxPower;
use crate::mode::Mode;
use crate::packet_config::{PacketPreamble, PacketEndianess};

pub trait RadioExt {
  fn constrain(self) -> Radio<Disabled>;
}

impl RadioExt for RADIO {
  fn constrain(self) -> Radio<Disabled> {
    Radio {
      _state: Disabled,
      radio: self
    }
  }
}

pub struct Radio<S> {
  _state: S,
  radio: RADIO,
}

impl<S> Radio<S> {

}

impl Radio<Disabled> {
  pub fn set_tx_power(self, tx_power: TxPower) -> Self {
    self.radio.txpower.write(|w| unsafe { w.bits(tx_power.value()) });
    self
  }

  pub fn set_mode(self, mode: Mode) -> Self {
    self.radio.mode.write(|w| unsafe { w.bits(mode.value()) });
    self
  }

  pub fn set_packet_length_bits(self, bits: u8) -> Self {
    self.radio.pcnf0.write(|w| unsafe { w.lflen().bits(bits) });
    self
  }

  pub fn set_packet_s0_exclude(self) -> Self {
    self.radio.pcnf0.write(|w| w.s0len().clear_bit());
    self
  }

  pub fn set_packet_s0_include(self) -> Self {
    self.radio.pcnf0.write(|w| w.s0len().set_bit());
    self
  }

  pub fn set_packet_s1_exclude(self) -> Self {
    self.radio.pcnf0.write(|w| unsafe { w.s1len().bits(0) });
    self.radio.pcnf0.write(|w| w.s1incl().automatic());
    self
  }

  pub fn set_packet_s1_include(self, bits: u8) -> Self {
    self.radio.pcnf0.write(|w| unsafe { w.s1len().bits(bits) });
    self.radio.pcnf0.write(|w| w.s1incl().include());
    self
  }

  pub fn set_packet_code_indicator_length(self, len: u8) -> Self {
    self.radio.pcnf0.write(|w| unsafe { w.cilen().bits(len) });
    self
  }

  pub fn set_packet_preamble(self, preamble: PacketPreamble) -> Self {
    let value = match preamble {
      PacketPreamble::Length8Bits => 0,
      PacketPreamble::Length16Bits => 1,
      PacketPreamble::Length32Bits => 2,
      PacketPreamble::LongRange => 3,
    };
    self.radio.pcnf0.write(|w| w.plen().bits(value));
    self
  }

  pub fn set_packet_length_include_crc(self, include: bool) -> Self {
    if include {
      self.radio.pcnf0.write(|w| w.crcinc().include());
    }
    else {
      self.radio.pcnf0.write(|w| w.crcinc().exclude());
    }
    self
  }

  pub fn set_packet_term_length(self, len: u8) -> Self {
    self.radio.pcnf0.write(|w| unsafe { w.termlen().bits(len) });
    self
  }

  pub fn get_packet_payload_max_length(&self) -> u8 {
    self.radio.pcnf1.read().maxlen().bits()
  }

  pub fn set_packet_payload_max_length(self, max_len: u8) -> Self {
    self.radio.pcnf1.write(|w| unsafe { w.maxlen().bits(max_len) });
    self
  }

  pub fn set_packet_static_length(self, bytes: u8) -> Self {
    self.radio.pcnf1.write(|w| unsafe { w.statlen().bits(bytes) });
    self
  }

//  pub fn set_packet_base_address_length(self, bytes: u8) -> Self {
//    self.radio.pcnf1.write(|w| unsafe { w.balen().bits(bytes) });
//    self
//  }

  pub fn set_packet_endianess(self, endianess: PacketEndianess) -> Self {
    match endianess {
      PacketEndianess::LittleEndian =>
        self.radio.pcnf1.write(|w| w.endian().little()),
      PacketEndianess::BigEndian =>
        self.radio.pcnf1.write(|w| w.endian().big()),
    }
    self
  }

  pub fn set_packet_whiteen_enabled(self, enabled: bool) -> Self {
    if enabled {
      self.radio.pcnf1.write(|w| w.whiteen().enabled());
    }
    else {
      self.radio.pcnf1.write(|w| w.whiteen().disabled());
    }
    self
  }

  pub fn set_crc_disabled(self) -> Self {
    self.radio.crccnf.write(|w| w.len().disabled());
    self
  }

  pub fn set_crc_8bits(self, initial: u8, polynomial: u32) -> Self {
    self.radio.crccnf.write(|w| w.len().one());
    self.radio.crcinit.write(|w| unsafe { w.bits(initial.into()) });
    self.radio.crcpoly.write(|w| unsafe { w.bits(polynomial) });
    self
  }

  pub fn set_crc_16bits(self, initial: u16, polynomial: u32) -> Self {
    self.radio.crccnf.write(|w| w.len().two());
    self.radio.crcinit.write(|w| unsafe { w.bits(initial.into()) });
    self.radio.crcpoly.write(|w| unsafe { w.bits(polynomial) });
    self
  }

  pub fn set_crc_24bits(self, initial: u32, polynomial: u32) -> Self {
    self.radio.crccnf.write(|w| w.len().three());
    self.radio.crcinit.write(|w| unsafe { w.crcinit().bits(initial) });
    self.radio.crcpoly.write(|w| unsafe { w.bits(polynomial) });
    self
  }

  pub fn set_crc_include_address(self) -> Self {
    self.radio.crccnf.write(|w| w.skipaddr().include());
    self
  }

  pub fn set_crc_skip_address(self) -> Self {
    self.radio.crccnf.write(|w| w.skipaddr().skip());
    self
  }

  pub fn set_crc_ieee802154(self) -> Self {
    self.radio.crccnf.write(|w| w.skipaddr().include());
    self
  }

  pub fn set_base_address(self, length: u8, base0: u32, base1: u32) -> Self {
    assert!(length >= 2 && length <= 4);
    self.radio.pcnf1.write(|w| unsafe { w.balen().bits(length) });
    self.radio.base0.write(|w| unsafe { w.bits(base0.reverse_bits()) } );
    self.radio.base1.write(|w| unsafe { w.bits(base1.reverse_bits()) } );
    self
  }

  pub fn set_prefixes(self, prefixes: [u8; 8]) -> Self {
    let prefix0 = u32::from(prefixes[0]) << 24 |
                        u32::from(prefixes[1]) << 16 |
                        u32::from(prefixes[2]) << 8 |
                        u32::from(prefixes[3]);
    let prefix1 = u32::from(prefixes[4]) << 24 |
                        u32::from(prefixes[5]) << 16 |
                        u32::from(prefixes[6]) << 8 |
                        u32::from(prefixes[7]);
    self.radio.prefix0.write(|w| unsafe { w.bits(prefix0.reverse_bits()) });
    self.radio.prefix1.write(|w| unsafe { w.bits(prefix1.reverse_bits()) });
    self
  }

  pub fn set_channel(self, channel: u8) -> Self {
    self.radio.frequency.write(|w| unsafe { w.frequency().bits(channel) });
    self
  }

  pub fn enable_rx(self) -> Radio<RxRumpUp> {
    Radio {
      _state: RxRumpUp,
      radio: self.radio
    }
  }
}
