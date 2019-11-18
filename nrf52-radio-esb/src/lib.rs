#![no_std]

pub mod protocol;

use cortex_m_semihosting::hprintln;

use nrf52_radio::Radio;
use nrf52_radio::{Result as RadioResult, AsyncResult as RadioAsyncResult};
use nrf52_radio::Error as RadioError;
use nrf52_radio::logical_address::LogicalAddress;
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

#[derive(Debug, Clone, Copy)]
pub struct RxPacket {
  pub length: u8,
  pub pid: u8,
  pub no_ack: bool,
  pub address: LogicalAddress,
  pub crc: u32,
}

#[derive(Debug, Clone)]
enum RxStep {
  Disable,
  WaitingDisable,
  Enable,
  WaitingIdle,
  Start,
  WaitingEnd,
}

#[derive(Debug, Clone)]
enum TxStep {
  Disable,
  WaitingDisable,
  Enable,
  WaitingIdle,
  Start,
  WaitingEnd,
}

#[derive(Debug, Clone)]
enum State {
  /// Standby
  Standby,
  /// Receiving
  Rx(RxStep),
  /// Acknowledging the reception
  RxAck(RxPacket, TxStep),
  /// Sending
  Tx(TxStep),
  /// Waiting for an acknowledgement
  TxAck(RxStep),
  /// Disable radio
  Disable,
  /// Unexpected error
  Error,
}

// TODO save the start timestamp for waiting states, and consider timeouts
pub struct Esb<'a, LFOSC, LFSTAT> {
  protocol: Protocol,
  pub radio: Radio<'a, LFOSC, LFSTAT>,
  state: State,
  next_buffer: Option<&'a mut [u8]>,
  rx_packet: Option<RxPacket>,
}

impl<'a, LFOSC, LFSTAT> Esb<'a, LFOSC, LFSTAT> {
  pub fn new(mut radio: Radio<'a, LFOSC, LFSTAT>,
             protocol: Protocol,
             read_buffer: &'a mut [u8],
             write_buffer: &'a mut [u8]) -> Esb<'a, LFOSC, LFSTAT> {

    // TODO check Radio state, stop, disable
    Self::setup_protocol(&radio, &protocol);
    drop(radio.swap_buffer(Some(write_buffer)));
    Esb {
      protocol,
      radio,
      state: State::Standby,
      next_buffer: Some(read_buffer),
      rx_packet: None,
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

  // TODO expose the packet rather than the raw buffer ?

  pub fn get_buffer(&self) -> &[u8] {
    match self.next_buffer.as_ref() {
      Some(buffer) => *buffer,
      None => &[],
    }
  }

  pub fn get_buffer_mut(&mut self) -> &mut [u8] {
    match self.next_buffer.as_mut() {
      Some(buffer) => *buffer,
      None => &mut [],
    }
  }

  pub fn get_last_received_packet(&self) -> Option<RxPacket> {
    self.rx_packet
  }

  // TODO ack option as a parameter or as a different method ?

  pub fn start_rx(&mut self) -> Result<()> {
    match self.state {
      State::Standby => {
        self.state = State::Rx(self.rx_step_from_radio_state());
        Ok(())
      },
      _ => Err(Error::StandbyRequired)
    }
  }

  pub fn wait_rx(&mut self) -> AsyncResult<()> {
    match self.state {
      State::Rx(ref step) => {
        let (next_state, result) = match step {
          RxStep::Disable => {
            self.radio.disable();
            self.next_state(State::Rx(RxStep::WaitingDisable))
          },
          RxStep::WaitingDisable => match self.radio.wait_disabled() {
            Ok(()) => self.next_state(State::Rx(RxStep::Enable)),
            Err(error) => self.handle_async_radio_error(error),
          },
          RxStep::Enable => match self.radio.enable_rx() {
            Ok(()) => self.next_state(State::Rx(RxStep::WaitingIdle)),
            Err(error) => self.handle_radio_error(error),
          },
          RxStep::WaitingIdle => match self.radio.wait_idle() {
            Ok(()) => self.next_state(State::Rx(RxStep::Start)),
            Err(error) => self.handle_async_radio_error(error),
          },
          RxStep::Start => match self.radio.start() {
            Ok(()) => self.next_state(State::Rx(RxStep::WaitingEnd)),
            Err(error) => self.handle_radio_error(error),
          },
          RxStep::WaitingEnd => match self.radio.wait_end_or_disable() {
            Ok(()) => {
              if self.radio.is_crc_ok() {
                self.next_buffer = self.radio.swap_buffer(self.next_buffer.take());
                let mut rx_buffer = self.get_buffer().iter();
                let length = *rx_buffer.next().unwrap();
                let pid_noack = *rx_buffer.next().unwrap();
                let pid = (pid_noack >> 1) & 0x03;
                // TODO check PID and skip repeated packet
                let packet = RxPacket {
                  length,
                  pid,
                  no_ack: (pid_noack & 0x01) == 0x01,
                  address: self.radio.get_received_address(),
                  crc: self.radio.get_received_crc(),
                };
                if packet.no_ack {
                  self.rx_packet = Some(packet);
                  self.radio.disable();
                  self.next_state(State::Disable)
                }
                else {
                  self.next_state(State::RxAck(packet, self.tx_step_from_radio_state()))
                }
              }
              else {
                // TODO maximum number of CRC retries
                self.next_state(State::Rx(self.rx_step_from_radio_state()))
              }
            },
            Err(error) => self.handle_async_radio_error(error),
          },
          _ => unimplemented!()
        };
        self.state = next_state;
        result
      },
      State::RxAck(packet, ref step) => {
        let (next_state, result) = match step {
          TxStep::Disable => {
            self.radio.disable();
            self.next_state(State::RxAck(packet, TxStep::WaitingDisable))
          },
          TxStep::WaitingDisable => match self.radio.wait_disabled() {
            Ok(()) => self.next_state(State::RxAck(packet, TxStep::Enable)),
            Err(error) => self.handle_async_radio_error(error),
          },
          TxStep::Enable => {
            let ack_buffer = self.radio.get_buffer_mut();
            ack_buffer[0] = 0;
            ack_buffer[1] = packet.pid << 1;
            self.radio.set_tx_address(packet.address);
            match self.radio.enable_tx() {
              Ok(()) => self.next_state(State::RxAck(packet, TxStep::WaitingIdle)),
              Err(error) => self.handle_radio_error(error),
            }
          },
          TxStep::WaitingIdle => match self.radio.wait_idle() {
            Ok(()) => self.next_state(State::RxAck(packet, self.tx_step_from_radio_state())),
            Err(error) => self.handle_async_radio_error(error),
          },
          TxStep::Start => match self.radio.start() {
            Ok(()) => self.next_state(State::RxAck(packet, TxStep::WaitingEnd)),
            Err(error) => self.handle_radio_error(error),
          },
          TxStep::WaitingEnd => match self.radio.wait_end_or_disable() {
            Ok(()) => {
              self.rx_packet = Some(packet);
              if self.radio.is_disabled() {
                (State::Standby, Ok(()))
              }
              else {
                self.radio.disable();
                self.next_state(State::Disable)
              }
            },
            Err(error) => self.handle_async_radio_error(error),
          },
          _ => unimplemented!()
        };
        self.state = next_state;
        result
      }
      State::Disable => {
        let (next_state, result) = match self.radio.wait_disabled() {
          Ok(()) => (State::Standby, Ok(())),
          Err(error) => self.handle_async_radio_error(error),
        };
        self.state = next_state;
        result
      },
      _ => Err(nb::Error::Other(Error::ReceiveNotStarted)),
    }
  }

  pub fn start_tx(&mut self, address: LogicalAddress) -> Result<()> {
    match self.state {
      State::Standby => {
        self.radio.set_tx_address(address);
        self.next_buffer = self.radio.swap_buffer(self.next_buffer.take());
        self.state = State::Tx(self.tx_step_from_radio_state());
        Ok(())
      },
      _ => Err(Error::StandbyRequired)
    }
  }

  pub fn wait_tx(&mut self) -> AsyncResult<()> {
    match self.state {
      State::Tx(ref step) => {
        let (next_state, result) = match step {
          TxStep::Disable => {
            self.radio.disable();
            self.next_state(State::Tx(TxStep::WaitingDisable))
          },
          TxStep::WaitingDisable => match self.radio.wait_disabled() {
            Ok(()) => self.next_state(State::Tx(TxStep::Enable)),
            Err(error) => self.handle_async_radio_error(error),
          },
          TxStep::Enable => match self.radio.enable_tx() {
            Ok(()) => self.next_state(State::Tx(TxStep::WaitingIdle)),
            Err(error) => self.handle_radio_error(error),
          },
          TxStep::WaitingIdle => match self.radio.wait_idle() {
            Ok(()) => self.next_state(State::Tx(self.tx_step_from_radio_state())),
            Err(error) => self.handle_async_radio_error(error),
          },
          TxStep::Start => match self.radio.start() {
            Ok(()) => self.next_state(State::Tx(TxStep::WaitingEnd)),
            Err(error) => self.handle_radio_error(error),
          },
          TxStep::WaitingEnd => match self.radio.wait_end_or_disable() {
            Ok(()) => {
              if self.radio.is_disabled() {
                (State::Standby, Ok(()))
              }
              else {
                self.radio.disable();
                self.next_state(State::Disable)
              }
            },
            Err(error) => self.handle_async_radio_error(error),
          },
          _ => unimplemented!()
        };
        self.state = next_state;
        result
      },
      State::Disable => {
        let (next_state, result) = match self.radio.wait_disabled() {
          Ok(()) => (State::Standby, Ok(())),
          Err(error) => self.handle_async_radio_error(error),
        };
        self.state = next_state;
        result
      },
      _ => Err(nb::Error::Other(Error::ReceiveNotStarted)),
    }
  }

  fn next_state<T>(&self, state: State) -> (State, AsyncResult<T>) {
    (state, Err(nb::Error::WouldBlock))
  }

  fn handle_radio_error<T>(&self, radio_error: RadioError) -> (State, AsyncResult<T>) {
    (State::Error, Err(nb::Error::Other(Error::RadioError(radio_error))))
  }

  fn handle_async_radio_error<T>(&self, error: nb::Error<RadioError>) -> (State, AsyncResult<T>) {
    match error {
      nb::Error::WouldBlock => (self.state.clone(), Err(nb::Error::WouldBlock)),
      nb::Error::Other(radio_error) => (State::Error, Err(nb::Error::Other(Error::RadioError(radio_error)))),
    }
  }

  fn rx_step_from_radio_state(&self) -> RxStep {
    match self.radio.get_state() {
      RadioState::Disabled  => RxStep::Enable,
      RadioState::RxRumpUp  => RxStep::WaitingIdle,
      RadioState::RxIdle    => RxStep::Start,
      RadioState::Rx        => RxStep::WaitingEnd,
      RadioState::RxDisable => RxStep::WaitingDisable,
      _                     => RxStep::Disable,
    }
  }

  fn tx_step_from_radio_state(&self) -> TxStep {
    match self.radio.get_state() {
      RadioState::Disabled  => TxStep::Enable,
      RadioState::TxRumpUp  => TxStep::WaitingIdle,
      RadioState::TxIdle    => TxStep::Start,
      RadioState::Tx        => TxStep::WaitingEnd,
      RadioState::TxDisable => TxStep::WaitingDisable,
      _                     => TxStep::Disable,
    }
  }
}
