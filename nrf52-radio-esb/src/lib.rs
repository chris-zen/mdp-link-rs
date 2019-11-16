#![no_std]

pub mod protocol;

use nrf52_radio::Radio;
use nrf52_radio::{Result as RadioResult, AsyncResult as RadioAsyncResult};
use nrf52_radio::Error as RadioError;
use nrf52_radio::packet_config::{S1Length, S1IncludeInRam, PreambleLength, Endianess, PacketConfig};
use nrf52_radio::states::State as RadioState;

use nb;

use crate::protocol::Protocol;

pub type Result<A> = core::result::Result<A, Error>;
pub type AsyncResult<A> = nb::Result<A, Error>;

#[derive(Debug, Clone)]
pub enum Error {
  StandbyRequired,
  ReceiveNotStarted,
  RadioError(RadioError),
}

// TODO save the start timestamp for waiting states, and consider timeouts
#[derive(Debug, Clone)]
enum Receiving {
  RequiresEnable,
  WaitingIdle,
  RequiresStart,
  WaitingPacket,
  RequiresDisable,
  WaitingDisable,
}

#[derive(Debug, Clone)]
enum Transmitting {
  // TODO
}

#[derive(Debug, Clone)]
enum State {
  Standby,
  Receiving(Receiving),
  Transmitting(Transmitting),
  Error,
}

pub struct Esb<'a, LFOSC, LFSTAT> {
  protocol: Protocol,
  pub radio: Radio<'a, LFOSC, LFSTAT>,
  state: State,
}

impl<'a, LFOSC, LFSTAT> Esb<'a, LFOSC, LFSTAT> {
  pub fn new(radio: Radio<'a, LFOSC, LFSTAT>, protocol: Protocol) -> Esb<'a, LFOSC, LFSTAT> {
    // TODO check Radio state, stop, disable
    Self::setup_protocol(&radio, &protocol);
    Esb {
      protocol,
      radio,
      state: State::Standby,
    }
  }

  fn setup_protocol(radio: &Radio<'a, LFOSC, LFSTAT>, protocol: &Protocol) {
    let pcfn = match protocol {
      Protocol::FixedPayloadLength(length) =>
        PacketConfig::default()
//            .with_length_bits(0)
//            .with_s0_byte_included(true)
            .with_length_bits(6)
            .with_s0_byte_included(false)
//            .with_s1_len(S1Length::Of1Bits)
            .with_s1_len(S1Length::Of3Bits)
            .with_s1_include_in_ram(S1IncludeInRam::Automatic)
            .with_preamble_len(PreambleLength::Of8Bits)
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
            .with_preamble_len(PreambleLength::Of8Bits)
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

  // TODO ack option as a parameter or as a different method ?

  pub fn start_receive(&mut self) -> Result<()> {
    match self.state {
      State::Standby => {
        self.state = State::Receiving(self.receiving_step_from_radio_state());
        Ok(())
      },
      _ => Err(Error::StandbyRequired)
    }
  }

  pub fn wait_receive(&mut self) -> AsyncResult<()> {
    match self.state {
      State::Receiving(ref step) => {
        let (next_state, result) = match step {
          Receiving::RequiresEnable => match self.radio.enable_rx() {
            Ok(()) => (State::Receiving(Receiving::WaitingIdle), Err(nb::Error::WouldBlock)),
            Err(radio_error) => (State::Error, Err(nb::Error::Other(Error::RadioError(radio_error)))),
          },
          Receiving::WaitingIdle => match self.radio.wait_idle() {
            Ok(()) => (State::Receiving(Receiving::RequiresStart), Err(nb::Error::WouldBlock)),
            Err(nb::Error::WouldBlock) => (self.state.clone(), Err(nb::Error::WouldBlock)),
            Err(nb::Error::Other(radio_error)) => (State::Error, Err(nb::Error::Other(Error::RadioError(radio_error)))),
          },
          Receiving::RequiresStart => match self.radio.start_rx() {
            Ok(()) => (State::Receiving(Receiving::WaitingPacket), Err(nb::Error::WouldBlock)),
            Err(radio_error) => (State::Error, Err(nb::Error::Other(Error::RadioError(radio_error)))),
          },
          Receiving::WaitingPacket => match self.radio.wait_packet_received() {
            Ok(()) => {
              if self.radio.is_crc_ok() {
                (State::Standby, Ok(()))
              }
              else {
                (
                  // TODO maximum number of CRC retries
                  State::Receiving(self.receiving_step_from_radio_state()),
                  Err(nb::Error::WouldBlock)
                )
              }
            },
            Err(nb::Error::WouldBlock) => (self.state.clone(), Err(nb::Error::WouldBlock)),
            Err(nb::Error::Other(radio_error)) => (State::Error, Err(nb::Error::Other(Error::RadioError(radio_error)))),
          },
          Receiving::RequiresDisable => {
            self.radio.disable();
            (State::Receiving(Receiving::WaitingDisable), Err(nb::Error::WouldBlock))
          },
          Receiving::WaitingDisable => match self.radio.wait_disabled() {
            Ok(()) => (State::Receiving(Receiving::RequiresEnable), Err(nb::Error::WouldBlock)),
            Err(nb::Error::WouldBlock) => (self.state.clone(), Err(nb::Error::WouldBlock)),
            Err(nb::Error::Other(radio_error)) => (State::Error, Err(nb::Error::Other(Error::RadioError(radio_error)))),
          },
          _ => unimplemented!()
        };
        self.state = next_state;
        result
      },
      _ => Err(nb::Error::Other(Error::ReceiveNotStarted)),
    }
  }

  fn receiving_step_from_radio_state(&self) -> Receiving {
    match self.radio.get_state() {
      RadioState::Disabled  => Receiving::RequiresEnable,
      RadioState::RxRumpUp  => Receiving::WaitingIdle,
      RadioState::RxIdle    => Receiving::RequiresStart,
      RadioState::Rx        => Receiving::WaitingPacket,
      RadioState::RxDisable => Receiving::WaitingDisable,
      _                     => Receiving::RequiresDisable,
    }
  }


// TODO buffer operations: exchange/switch, take
// TODO transmit operation
}
