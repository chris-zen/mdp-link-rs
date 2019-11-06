/*!

nrf52840 Radio

See [Product Specification](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.0.pdf): 6.20 RADIO — 2.4 GHz radio

*/

use cortex_m_semihosting::{dbg, hprintln, heprintln};

use nb;

use crate::hal::target::RADIO;
use crate::tx_power::TxPower;
use crate::mode::Mode;
use crate::packet_config::{PacketPreamble, PacketEndianess};
use crate::frequency::Frequency;
use crate::base_address::BaseAddresses;
use crate::NbResult;
use crate::states::State;


pub trait RadioExt {
  fn constrain(self) -> Radio;
}

impl RadioExt for RADIO {
  fn constrain(self) -> Radio {
    Radio {
      radio: self
    }
  }
}

pub struct Radio {
  pub radio: RADIO,
}

impl Radio {

  pub fn enable_interrupts(self, bits: u32) -> Self {
    self.radio.intenset.write(|w| unsafe { w.bits(bits) });
    self
  }

  pub fn disable_interrupts(self, bits: u32) -> Self {
    self.radio.intenclr.write(|w| unsafe { w.bits(bits) });
    self
  }

  pub fn disable_all_interrupts(self) -> Self {
    self.radio.intenclr.write(|w| unsafe { w.bits(0xffffffff) });
    self
  }

  pub fn enable_power(self) -> Self {
    self.radio.power.write(|w| w.power().enabled());
    self
  }

  pub fn disable_power(self) -> Self {
    self.radio.power.write(|w| w.power().enabled());
    self
  }

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

  pub fn get_state(&self) -> State {
    State::from_value(self.radio.state.read().state().bits())
  }

  pub fn set_base_addresses(self, addr: BaseAddresses) -> Self {
    let (length, base0, base1) = match addr {
      BaseAddresses::TwoBytes(addr0, addr1) => (2, u32::from(addr0), u32::from(addr1)),
      BaseAddresses::ThreeBytes(addr0, addr1) => (2, addr0 & 0xffffff, addr1 & 0xffffff),
      BaseAddresses::FourBytes(addr0, addr1) => (2, addr0, addr1),
    };
    self.radio.pcnf1.write(|w| unsafe { w.balen().bits(length) });
    self.radio.base0.write(|w| unsafe { w.bits(base0/*.reverse_bits()*/) } );
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
    self.radio.prefix0.write(|w| unsafe { w.bits(prefix0/*.reverse_bits()*/) });
    self.radio.prefix1.write(|w| unsafe { w.bits(prefix1.reverse_bits()) });
    self
  }

  /// 6.20.14.9 PACKETPTR
  /// Not sure to expose this, we need to play well with ownership and borrowing
  fn set_packet_ptr(&self, buffer: &mut [u8]) {
    let ptr = buffer.as_ptr() as u32;
    self.radio.packetptr.write(|w| unsafe { w.bits(ptr) });
  }

  /// 6.20.14.10 FREQUENCY
  pub fn set_frequency(self, freq: Frequency) -> Self {
    self.radio.frequency.write(|w| unsafe {
      let channel = match freq {
        Frequency::Default2400MHz(channel) => {
          w.map().default();
          channel
        },
        Frequency::Low2360MHz(channel) => {
          w.map().low();
          channel
        },
      };
      assert!(channel <= 100);
      w.frequency().bits(channel)
    });
    self
  }

  /// Receive address select. 6.20.14.20 RXADDRESSES
  pub fn set_rx_addresses(self, mask: u8) -> Self {
    self.radio.rxaddresses.write(|w| unsafe { w.bits(mask.into()) });
    self
  }

  pub fn enable_rx(&self, buffer: &mut [u8]) {
    // TODO check current state
    self.set_packet_ptr(buffer);
    self.radio.events_ready.reset();
    self.radio.events_disabled.reset();
    self.radio.events_end.reset();
    self.radio.events_address.reset();
    self.radio.events_payload.reset();
    self.radio.tasks_rxen.write(|w| w.tasks_rxen().set_bit());
    dbg!(self.radio.state.read().bits());
  }

  pub fn is_ready(&self) -> bool {
    self.radio.events_ready.read().events_ready().bit_is_set()
  }

  pub fn wait_idle(&self) -> NbResult<()> {
    if self.is_ready() {
      self.radio.events_ready.reset();
      Ok(())
    }
    else {
      Err(nb::Error::WouldBlock)
    }
  }

  pub fn start_rx(&self) {
    // TODO check current state
//    heprintln!("{:x}", self.radio.base0.read().bits().reverse_bits()).unwrap();
//    heprintln!("{:x}", self.radio.base1.read().bits().reverse_bits()).unwrap();
//    heprintln!("{:x}", self.radio.prefix0.read().bits().reverse_bits()).unwrap();
//    heprintln!("{:x}", self.radio.prefix1.read().bits().reverse_bits()).unwrap();
    self.radio.events_end.reset();
    self.radio.events_address.reset();
    self.radio.events_payload.reset();
    self.radio.events_disabled.reset();
    self.radio.tasks_start.write(|w| w.tasks_start().set_bit());
  }

  pub fn is_address_received(&self) -> bool {
    self.radio.events_address.read().events_address().bit_is_set()
  }

  pub fn is_payload_received(&self) -> bool {
    self.radio.events_payload.read().events_payload().bit_is_set()
  }

  pub fn is_packet_received(&self) -> bool {
    self.radio.events_end.read().events_end().bit_is_set()
  }

  pub fn is_crc_ok(&self) -> bool {
    self.radio.crcstatus.read().crcstatus().is_crcok()
  }

  pub fn wait_packet_received(&self) -> NbResult<()> {
    if self.is_packet_received() {
      self.radio.events_end.reset();
      self.radio.events_address.reset();
      self.radio.events_payload.reset();
//      dbg!(self.radio.crcstatus.read().bits());
//      dbg!(self.radio.rxmatch.read().bits());
//      dbg!(self.radio.rxcrc.read().bits());
//      dbg!(self.radio.dai.read().bits());
      Ok(())
    }
    else {
      Err(nb::Error::WouldBlock)
    }
  }

  pub fn stop(&self) {
    // TODO clear events ???
    self.radio.tasks_stop.write(|w| w.tasks_stop().set_bit());
  }

  pub fn disable(&self) {
    self.radio.events_disabled.reset();
    self.radio.tasks_disable.write(|w| w.tasks_disable().set_bit());
  }

  pub fn is_disabled(&self) -> bool {
    self.radio.events_disabled.read().events_disabled().bit_is_set()
  }

  pub fn wait_disabled(&self) -> NbResult<()> {
    if self.is_disabled() {
      self.radio.events_disabled.reset();
      Ok(())
    }
    else {
      Err(nb::Error::WouldBlock)
    }
  }

  pub fn free(self) -> RADIO {
    self.radio
  }
}
