///!
///!
///! nrf52840 Radio
///!
///! See [Product Specification](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.0.pdf): 6.20 RADIO — 2.4 GHz radio
///!

use core::convert::TryFrom;
use core::sync::atomic::{compiler_fence, Ordering};
//use core::ops::Deref;
//use cortex_m_semihosting::{dbg, hprintln, heprintln};
use nb;

use crate::hal::target::RADIO;
use crate::hal::clocks::ExternalOscillator;
use crate::tx_power::TxPower;
use crate::mode::Mode;
use crate::packet_config::{S1IncludeInRam, Endianess, PacketConfig};
use crate::frequency::Frequency;
use crate::base_address::BaseAddresses;
use crate::states::State;
use nrf52840_hal::Clocks;


macro_rules! map_or {
  ( $option:expr, $default:expr, |$value:ident| $transform:expr ) => {
    match $option {
      Some($value) => $transform,
      None => $default,
    }
  }
}

pub type Result<A> = core::result::Result<A, Error>;
pub type AsyncResult<T> = nb::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
  BufferNotDefined,
  WrongState,
}

pub trait RadioExt {
  fn constrain<LFOSC, LFSTAT>(self,
                              clocks: &Clocks<ExternalOscillator, LFOSC, LFSTAT>
                             ) -> Radio<LFOSC, LFSTAT>;
}

impl RadioExt for RADIO {
  fn constrain<'a, LFOSC, LFSTAT>(self,
                                  clocks: &'a Clocks<ExternalOscillator, LFOSC, LFSTAT>,
                                 ) -> Radio<'a, LFOSC, LFSTAT> {

    Radio::<'a, LFOSC, LFSTAT>::new(self, clocks)
  }
}

//struct RadioX<T> {
//  regs: T
//}
//
//impl<T> RadioX<T> where T: Deref<Target=crate::hal::target::radio::RegisterBlock> {
//  pub fn foo(&self) {
//    self.regs.power.write(|w| w.power().enabled());
//  }
//}

pub struct Packet;

pub struct Radio<'a, LFOSC, LFSTAT> {
  clocks: &'a Clocks<ExternalOscillator, LFOSC, LFSTAT>,
  pub radio: RADIO,
  buffer: Option<&'a mut [u8]>,
}

impl<'a, LFOSC, LFSTAT> Radio<'a, LFOSC, LFSTAT> {

  pub fn new(radio: RADIO,
             clocks: &'a Clocks<ExternalOscillator, LFOSC, LFSTAT>) -> Self {

    Radio {
      radio,
      clocks,
      buffer: None,
    }
  }

  pub fn enable_interrupts(&self, bits: u32) -> &Self {
    self.radio.intenset.write(|w| unsafe { w.bits(bits) });
    self
  }

  pub fn disable_interrupts(&self, bits: u32) -> &Self {
    self.radio.intenclr.write(|w| unsafe { w.bits(bits) });
    self
  }

  pub fn disable_all_interrupts(&self) -> &Self {
    self.radio.intenclr.write(|w| unsafe { w.bits(0xffffffff) });
    self
  }

  pub fn enable_power(&self) -> &Self {
    self.radio.power.write(|w| w.power().enabled());
    self
  }

  pub fn disable_power(&self) -> &Self {
    self.radio.power.write(|w| w.power().enabled());
    self
  }

  pub fn set_tx_power(&self, tx_power: TxPower) -> &Self {
    self.radio.txpower.write(|w| unsafe { w.bits(tx_power.value()) });
    self
  }

  pub fn set_mode(&self, mode: Mode) -> &Self {
    self.radio.mode.write(|w| unsafe { w.bits(mode.value()) });
    self
  }

  pub fn set_packet_config(&self, pcfn: PacketConfig) -> &Self {
    self.radio.pcnf0.modify(|_, w| unsafe {
      Some(w)
        .map(|w| map_or!(&pcfn.length_bits, w, |bits| w.lflen().bits(*bits)))
        .map(|w| map_or!(&pcfn.s0_byte_included, w, |included| w.s0len().bit(*included)))
        .map(|w| map_or!(&pcfn.s1_len, w, |len| w.s1len().bits(u8::try_from(len.value()).unwrap())))
        .map(|w| map_or!(&pcfn.s1_include_in_ram, w, |included| w.s1incl().bit(*included == S1IncludeInRam::Always)))
        .map(|w| map_or!(&pcfn.preamble_len, w, |len| w.plen().bits(u8::try_from(len.value()).unwrap())))
        .map(|w| map_or!(&pcfn.crc_included_in_length, w, |included| w.crcinc().bit(*included)))
        .unwrap()
    });
    self.radio.pcnf1.modify(|_, w| unsafe {
      Some(w)
        .map(|w| map_or!(&pcfn.max_bytes, w, |bytes| w.maxlen().bits(*bytes)))
        .map(|w| map_or!(&pcfn.static_bytes, w, |bytes| w.statlen().bits(*bytes)))
        .map(|w| map_or!(&pcfn.endianess, w, |endianess| w.endian().bit(*endianess == Endianess::BigEndian)))
        .map(|w| map_or!(&pcfn.whitening_enabled, w, |enabled| w.whiteen().bit(*enabled)))
        .unwrap()
    });
    self
  }

  pub fn set_crc_disabled(&self) -> &Self {
    self.radio.crccnf.modify(|_, w| w.len().disabled());
    self
  }

  pub fn set_crc_8bits(&self, initial: u8, polynomial: u32) -> &Self {
    self.radio.crccnf.modify(|_, w| w.len().one());
    self.radio.crcinit.write(|w| unsafe { w.bits(initial.into()) });
    self.radio.crcpoly.write(|w| unsafe { w.bits(polynomial) });
    self
  }

  pub fn set_crc_16bits(&self, initial: u16, polynomial: u32) -> &Self {
    self.radio.crccnf.modify(|_, w| w.len().two());
    self.radio.crcinit.write(|w| unsafe { w.bits(initial.into()) });
    self.radio.crcpoly.write(|w| unsafe { w.bits(polynomial) });
    self
  }

  pub fn set_crc_24bits(&self, initial: u32, polynomial: u32) -> &Self {
    self.radio.crccnf.modify(|_, w| w.len().three());
    self.radio.crcinit.write(|w| unsafe { w.crcinit().bits(initial) });
    self.radio.crcpoly.write(|w| unsafe { w.bits(polynomial) });
    self
  }

  pub fn set_crc_include_address(&self) -> &Self {
    self.radio.crccnf.modify(|_, w| w.skipaddr().include());
    self
  }

  pub fn set_crc_skip_address(&self) -> &Self {
    self.radio.crccnf.modify(|_, w| w.skipaddr().skip());
    self
  }

  pub fn set_crc_ieee802154(self) -> Self {
    self.radio.crccnf.modify(|_, w| w.skipaddr().include());
    self
  }

  pub fn set_base_addresses(&self, addr: BaseAddresses) -> &Self {
    let (length, base0, base1) = match addr {
      BaseAddresses::TwoBytes(addr0, addr1) => (2, u32::from(addr0), u32::from(addr1)),
      BaseAddresses::ThreeBytes(addr0, addr1) => (3, addr0 & 0xffffff, addr1 & 0xffffff),
      BaseAddresses::FourBytes(addr0, addr1) => (4, addr0, addr1),
    };
    self.radio.pcnf1.modify(|_, w| unsafe { w.balen().bits(length) });
    self.radio.base0.write(|w| unsafe { w.bits(base0.reverse_bits()) } );
    self.radio.base1.write(|w| unsafe { w.bits(base1.reverse_bits()) } );
    self
  }

  pub fn set_prefixes(&self, prefixes: [u8; 8]) -> &Self {
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

  /// 6.20.14.10 FREQUENCY
  pub fn set_frequency(&self, freq: Frequency) -> &Self {
    self.radio.frequency.write(|w| unsafe {
      let (channel, w) = match freq {
        Frequency::Default2400MHz(channel) => (channel, w.map().default()),
        Frequency::Low2360MHz(channel) => (channel, w.map().low())
      };
      assert!(channel <= 100);
      w.frequency().bits(channel)
    });
    self
  }

  /// 6.20.14.20 RXADDRESSES: Receive address select
  // TODO use a library for bit fields
  pub fn set_rx_addresses(&self, mask: u8) -> &Self {
    self.radio.rxaddresses.write(|w| unsafe { w.bits(mask.into()) });
    self
  }

  pub fn get_state(&self) -> State {
    State::from_value(self.radio.state.read().state().bits())
  }

  pub fn is_ready(&self) -> bool {
    self.radio.events_ready.read().events_ready().bit_is_set()
  }

  pub fn is_address_received(&self) -> bool {
    self.radio.events_address.read().events_address().bit_is_set()
  }

  pub fn is_payload_received(&self) -> bool {
    self.radio.events_payload.read().events_payload().bit_is_set()
  }

  pub fn is_packet_received(&self) -> bool {
    self.radio.events_end.read().events_end().bit_is_set() ||
        self.radio.events_disabled.read().events_disabled().bit_is_set()
  }

  pub fn is_crc_ok(&self) -> bool {
    self.radio.crcstatus.read().crcstatus().is_crcok()
  }

  pub fn is_disabled(&self) -> bool {
    self.radio.events_disabled.read().events_disabled().bit_is_set()
  }

  /// 6.20.14.9 PACKETPTR
  fn set_packet_ptr(&self, buffer: &[u8]) {
    let ptr = buffer.as_ptr() as u32;
    self.radio.packetptr.write(|w| unsafe { w.bits(ptr) });
  }

  pub fn get_buffer(&self) -> &[u8] {
    match self.buffer.as_ref() {
      Some(buffer) => *buffer,
      None => &[],
    }
  }

  pub fn get_buffer_mut(&mut self) -> &mut [u8] {
    match self.buffer.as_mut() {
      Some(buffer) => *buffer,
      None => &mut [],
    }
  }

  // TODO should we use a critical section ?
  pub fn swap_buffer(&mut self, new_buffer: Option<&'a mut [u8]>) -> Option<&'a mut [u8]> {
    match new_buffer.as_ref() {
      Some(buffer) => self.set_packet_ptr(*buffer),
      None => (),
    }
    let prev_buffer = self.buffer.take();
    self.buffer = new_buffer;
    prev_buffer
  }

  pub fn extract_packet(&mut self, new_buffer: Option<&'a mut [u8]>) -> Packet {
    unimplemented!() // TODO implement extract_packet
  }

  pub fn enable_rx(&mut self) -> Result<()> {
    match (self.buffer.as_ref(), self.get_state()) {
      (Some(buffer), State::Disabled) => {
        self.set_packet_ptr(*buffer);

        self.radio.events_ready.reset();
        self.radio.events_disabled.reset();
        self.radio.events_end.reset();
        self.radio.events_address.reset();
        self.radio.events_payload.reset();

        // "Preceding reads and writes cannot be moved past subsequent writes."
        compiler_fence(Ordering::Release);

        self.radio.tasks_rxen.write(|w| w.tasks_rxen().set_bit());

        Ok(())
      },
      (Some(_), _) => Err(Error::WrongState),
      (None, _)    => Err(Error::BufferNotDefined),
    }
  }

  pub fn wait_idle(&self) -> AsyncResult<()> {
    if self.is_ready() {
      self.radio.events_ready.reset();
      Ok(())
    }
    else {
      Err(nb::Error::WouldBlock)
    }
  }

  pub fn start_rx(&self) -> Result<()> {
    match (self.buffer.as_ref(), self.get_state()) {
      (Some(buffer), State::RxIdle) => {
        self.set_packet_ptr(*buffer);

        self.radio.events_end.reset();
        self.radio.events_address.reset();
        self.radio.events_payload.reset();
        self.radio.events_disabled.reset();

        // "Preceding reads and writes cannot be moved past subsequent writes."
        compiler_fence(Ordering::Release);

        self.radio.tasks_start.write(|w| w.tasks_start().set_bit());

        Ok(())
      },
      (Some(_), _) => Err(Error::WrongState),
      (None, _)    => Err(Error::BufferNotDefined),
    }
  }

  pub fn wait_packet_received(&self) -> AsyncResult<()> {
    if self.is_packet_received() {
      self.radio.events_end.reset();
      self.radio.events_address.reset();
      self.radio.events_payload.reset();
      Ok(())
    }
    else {
      Err(nb::Error::WouldBlock)
    }
  }

  pub fn stop(&self) -> Result<()> {
    match self.get_state() {
      State::Rx | State::Tx => {
        // TODO clear events ???
        self.radio.tasks_stop.write(|w| w.tasks_stop().set_bit());
        Ok(())
      },
      _ => Err(Error::WrongState)
    }
  }

  pub fn disable(&self) {
    self.radio.events_disabled.reset();

    // "Preceding reads and writes cannot be moved past subsequent writes."
    compiler_fence(Ordering::Release);

    self.radio.tasks_disable.write(|w| w.tasks_disable().set_bit());
  }

  pub fn wait_disabled(&self) -> AsyncResult<()> {
    if self.is_disabled() {
      self.radio.events_disabled.reset();
      Ok(())
    }
    else {
      Err(nb::Error::WouldBlock)
    }
  }

  pub fn free(self) -> RADIO {
    // TODO disable, or fail if not disabled
    // TODO reset PACKETPTR ?
    self.radio
  }
}
