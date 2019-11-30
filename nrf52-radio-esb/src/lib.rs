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

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Error {
  /// Standby required before starting a rx/tx transaction
  StandbyRequired,

  /// Rx buffer not ready to start a rx transaction
  RxBufferBusy,

  /// Tx buffer not ready to start a tx transaction
  TxBufferBusy,

  /// wait_rx called without a successful start_rx was called before
  ReceiveNotStarted,

  /// Unexpected error from the radio
  RadioError(RadioError),
}

#[derive(Debug, Clone, Copy)]
pub struct RxConfig {
  skip_ack: bool,
  retries: usize,
}

impl Default for RxConfig {
  fn default() -> Self {
    RxConfig {
      skip_ack: false,
      retries: 1,
    }
  }
}

impl RxConfig {
  pub fn with_skip_ack(self, skip_ack: bool) -> Self {
    RxConfig { skip_ack, .. self }
  }

  pub fn with_retries(self, retries: usize) -> Self {
    RxConfig { retries, .. self }
  }
}


#[derive(Debug, Clone, Copy)]
pub struct TxConfig {
  address: LogicalAddress,
  skip_ack: bool,
  retries: usize,
}

impl Default for TxConfig {
  fn default() -> Self {
    TxConfig {
      address: LogicalAddress::Of0,
      skip_ack: false,
      retries: 1,
    }
  }
}

impl TxConfig {
  pub fn new(address: LogicalAddress) -> Self {
    TxConfig { address, .. TxConfig::default() }
  }

  pub fn with_skip_ack(self, skip_ack: bool) -> Self {
    TxConfig { skip_ack, .. self }
  }

  pub fn with_retries(self, retries: usize) -> Self {
    TxConfig { retries, .. self }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct RxPacket {
  pub length: u8,
  pub pid: u8,
  pub no_ack: bool,
  pub address: LogicalAddress,
  pub crc: u32,
}

pub struct TxPacket {
  address: LogicalAddress,
  wait_ack: bool,
}

#[derive(Debug, Clone)]
enum Step {
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
  Rx(RxConfig, Step),
  /// Transmitting an acknowledgement
  TxAck(RxPacket, Step),
  /// Transmitting
  Tx(TxConfig, Step),
  /// Receiving an acknowledgement
  RxAck(TxConfig, Step),
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
  rx_buffer: Option<&'a mut [u8]>,
  tx_buffer: Option<&'a mut [u8]>,
  rx_packet: Option<RxPacket>,
  tx_packet: Option<TxPacket>,
}

impl<'a, LFOSC, LFSTAT> Esb<'a, LFOSC, LFSTAT> {
  pub fn new(mut radio: Radio<'a, LFOSC, LFSTAT>,
             protocol: Protocol,
             read_buffer: &'a mut [u8],
             write_buffer: &'a mut [u8]) -> Esb<'a, LFOSC, LFSTAT> {

    // TODO check Radio state, stop, disable
    Self::setup_protocol(&radio, &protocol);
    drop(radio.swap_buffer(None));
    Esb {
      protocol,
      radio,
      state: State::Standby,
      rx_buffer: Some(read_buffer),
      tx_buffer: Some(write_buffer),
      rx_packet: None,
      tx_packet: None,
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

  pub fn get_rx_buffer(&self) -> &[u8] {
    match self.rx_buffer.as_ref() {
      Some(buffer) => *buffer,
      None => &[],
    }
  }

  pub fn get_tx_buffer(&mut self) -> &mut [u8] {
    match self.tx_buffer.as_mut() {
      Some(buffer) => *buffer,
      None => &mut [],
    }
  }

  pub fn get_last_received_packet(&self) -> Option<RxPacket> {
    self.rx_packet
  }

  // TODO ack option as a parameter or as a different method ?

  pub fn start_rx(&mut self, rx_config: RxConfig) -> Result<()> {
    match self.state {
      State::Standby => {
        if self.rx_buffer.is_some() {
          self.rx_packet = None;
          self.state = State::Rx(rx_config, self.rx_step_from_radio_state());
          Ok(())
        }
        else {
          Err(Error::RxBufferBusy)
        }
      },
      _ => Err(Error::StandbyRequired)
    }
  }

  pub fn wait_rx(&mut self) -> AsyncResult<()> {
    match self.state {
      State::Rx(config, ref step) => {
        // TODO check timeout

        let (next_state, result) = match step {
          Step::Disable => {
            self.radio.disable();
            self.next_state(State::Rx(config, Step::WaitingDisable))
          },
          Step::WaitingDisable => match self.radio.wait_disabled() {
            Ok(()) => self.next_state(State::Rx(config, Step::Enable)),
            Err(error) => self.handle_async_radio_error(error),
          },
          Step::Enable => {
            self.ensure_rx_buffer();
            match self.radio.enable_rx() {
              Ok(()) => self.next_state(State::Rx(config, Step::WaitingIdle)),
              Err(error) => self.handle_radio_error(error),
            }
          },
          Step::WaitingIdle => {
            self.ensure_rx_buffer();
            match self.radio.wait_idle() {
              Ok(()) => self.next_state(State::Rx(config, Step::Start)),
              Err(error) => self.handle_async_radio_error(error),
            }
          },
          Step::Start => {
            self.ensure_rx_buffer();
            match self.radio.start() {
              Ok(()) => self.next_state(State::Rx(config, Step::WaitingEnd)),
              Err(error) => self.handle_radio_error(error),
            }
          },
          Step::WaitingEnd => match self.radio.wait_end_or_disable() {
            Ok(()) => {
              if self.radio.is_crc_ok() {
                self.rx_buffer = self.radio.swap_buffer(self.tx_buffer.take());
                let mut rx_buffer = self.get_rx_buffer().iter();
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
                if config.skip_ack || packet.no_ack {
                  self.rx_packet = Some(packet);
                  self.tx_buffer = self.radio.swap_buffer(None);
                  self.disable()
                }
                else {
                  self.next_state(State::TxAck(packet, self.tx_step_from_radio_state()))
                }
              }
              else {
                self.next_state(State::Rx(config, self.rx_step_from_radio_state()))
              }
            },
            Err(error) => self.handle_async_radio_error(error),
          }
        };
        self.state = next_state;
        result
      },
      State::TxAck(packet, ref step) => {
        let (next_state, result) = match step {
          Step::Disable => {
            self.radio.disable();
            self.next_state(State::TxAck(packet, Step::WaitingDisable))
          },
          Step::WaitingDisable => match self.radio.wait_disabled() {
            Ok(()) => self.next_state(State::TxAck(packet, Step::Enable)),
            Err(error) => self.handle_async_radio_error(error),
          },
          Step::Enable => {
            self.prepare_tx_ack(&packet);
            match self.radio.enable_tx() {
              Ok(()) => self.next_state(State::TxAck(packet, Step::WaitingIdle)),
              Err(error) => self.handle_radio_error(error),
            }
          },
          Step::WaitingIdle => match self.radio.wait_idle() {
            Ok(()) => self.next_state(State::TxAck(packet, self.tx_step_from_radio_state())),
            Err(error) => self.handle_async_radio_error(error),
          },
          Step::Start => match self.radio.start() {
            Ok(()) => self.next_state(State::TxAck(packet, Step::WaitingEnd)),
            Err(error) => self.handle_radio_error(error),
          },
          Step::WaitingEnd => match self.radio.wait_end_or_disable() {
            Ok(()) => {
              self.rx_packet = Some(packet);
              self.tx_buffer = self.radio.swap_buffer(None);
              self.disable()
            },
            Err(error) => self.handle_async_radio_error(error),
          }
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

  pub fn start_tx(&mut self, tx_config: TxConfig) -> Result<()> {
    match self.state {
      State::Standby => {
        if self.tx_buffer.is_some() {
          self.radio.set_tx_address(tx_config.address);
          drop(self.radio.swap_buffer(self.tx_buffer.take()));
          self.state = State::Tx(tx_config, self.tx_step_from_radio_state());
          Ok(())
        }
        else {
          Err(Error::TxBufferBusy)
        }
      },
      _ => Err(Error::StandbyRequired)
    }
  }

  pub fn wait_tx(&mut self) -> AsyncResult<()> {
    match self.state {
      State::Tx(config, ref step) => {
        let (next_state, result) = match step {
          Step::Disable => {
            self.radio.disable();
            self.next_state(State::Tx(config, Step::WaitingDisable))
          },
          Step::WaitingDisable => match self.radio.wait_disabled() {
            Ok(()) => self.next_state(State::Tx(config, Step::Enable)),
            Err(error) => self.handle_async_radio_error(error),
          },
          Step::Enable => match self.radio.enable_tx() {
            Ok(()) => self.next_state(State::Tx(config, Step::WaitingIdle)),
            Err(error) => self.handle_radio_error(error),
          },
          Step::WaitingIdle => match self.radio.wait_idle() {
            Ok(()) => self.next_state(State::Tx(config, self.tx_step_from_radio_state())),
            Err(error) => self.handle_async_radio_error(error),
          },
          Step::Start => match self.radio.start() {
            Ok(()) => self.next_state(State::Tx(config, Step::WaitingEnd)),
            Err(error) => self.handle_radio_error(error),
          },
          Step::WaitingEnd => match self.radio.wait_end_or_disable() {
            Ok(()) => {
              if config.skip_ack {
                self.tx_buffer = self.radio.swap_buffer(None);
                self.disable()
              }
              else {
                self.tx_buffer = self.radio.swap_buffer(self.rx_buffer.take());
                self.next_state(State::RxAck(config, self.rx_step_from_radio_state()))
              }
            },
            Err(error) => self.handle_async_radio_error(error),
          }
        };
        self.state = next_state;
        result
      },
      State::RxAck(config, ref step) => {
        // TODO check timeout
        let (next_state, result) = match step {
          Step::Disable => {
            self.radio.disable();
            self.next_state(State::RxAck(config, Step::WaitingDisable))
          },
          Step::WaitingDisable => match self.radio.wait_disabled() {
            Ok(()) => self.next_state(State::RxAck(config, Step::Enable)),
            Err(error) => self.handle_async_radio_error(error),
          },
          Step::Enable => match self.radio.enable_rx() {
            Ok(()) => self.next_state(State::RxAck(config, Step::WaitingIdle)),
            Err(error) => self.handle_radio_error(error),
          },
          Step::WaitingIdle => match self.radio.wait_idle() {
            Ok(()) => self.next_state(State::RxAck(config, self.rx_step_from_radio_state())),
            Err(error) => self.handle_async_radio_error(error),
          },
          Step::Start => match self.radio.start() {
            Ok(()) => self.next_state(State::RxAck(config, Step::WaitingEnd)),
            Err(error) => self.handle_radio_error(error),
          },
          Step::WaitingEnd => match self.radio.wait_end_or_disable() {
            Ok(()) => {
              if self.radio.is_crc_ok() {
                // TODO check PID
                self.rx_buffer = self.radio.swap_buffer(None);
                self.disable()
              }
              else {
                self.next_state(State::RxAck(config, self.rx_step_from_radio_state()))
              }
            },
            Err(error) => self.handle_async_radio_error(error),
          },
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

  fn disable(&self) -> (State, AsyncResult<()>) {
    if self.radio.is_disabled() {
      (State::Standby, Ok(()))
    }
    else {
      self.radio.disable();
      self.next_state(State::Disable)
    }
  }

  fn ensure_rx_buffer(&mut self) {
    if self.rx_buffer.is_some() {
      drop(self.radio.swap_buffer(self.rx_buffer.take()));
    }
  }

  fn set_tx_buffer(&mut self) {
    if self.tx_buffer.is_some() {
      drop(self.radio.swap_buffer(self.tx_buffer.take()));
    }
  }

  fn rx_step_from_radio_state(&self) -> Step {
    match self.radio.get_state() {
      RadioState::Disabled  => Step::Enable,
      RadioState::RxRumpUp  => Step::WaitingIdle,
      RadioState::RxIdle    => Step::Start,
      RadioState::Rx        => Step::WaitingEnd,
      RadioState::RxDisable => Step::WaitingDisable,
      _                     => Step::Disable,
    }
  }

  fn tx_step_from_radio_state(&self) -> Step {
    match self.radio.get_state() {
      RadioState::Disabled  => Step::Enable,
      RadioState::TxRumpUp  => Step::WaitingIdle,
      RadioState::TxIdle    => Step::Start,
      RadioState::Tx        => Step::WaitingEnd,
      RadioState::TxDisable => Step::WaitingDisable,
      _                     => Step::Disable,
    }
  }

  fn prepare_tx_ack(&mut self, packet: &RxPacket) {
    let ack_buffer = self.radio.get_buffer_mut();
    ack_buffer[0] = 0;
    ack_buffer[1] = packet.pid << 1;
    self.radio.set_tx_address(packet.address);
  }
}
