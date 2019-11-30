#[feature(option_result_contains)]

use core::fmt::Write;

use nb::block;

use crate::hal::target::UARTE0;

use nrf52_radio_esb::{Esb, RxConfig, Error as EsbError, TxConfig};
use nrf52840_hal::Uarte;
use crate::nrf52840_mdk::{Led, Leds};

// M01 asks P905 to connect
const PAIRING_REQUEST: [u8; 34] = [33, 0,
  0x09, 0x08, 0x62, 0x6d, 0xfa, 0x5d, 0x00, 0x01,
  0x5a, 0x73, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

// P905 responds the request from M01
const PAIRING_RESPONSE: [u8; 34] = [33, 0,
  0x09, 0x0d, 0x62, 0x6d, 0xfa, 0x5d, 0x00, 0x00,
  0x3e, 0xc2, 0x3b, 0x00, 0x0f, 0x78, 0x6d, 0xf9,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Error {
  EsbError(EsbError),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum State {
  Unpaired,
  WaitPairingRequest,
  SendPairingResponse,
  WaitPairingResponseSent,
  Paired,
  WaitRequest,
  Error(Error),
}

pub struct Protocol<'a, LFOSC, LFSTAT> {
  esb: Esb<'a, LFOSC, LFSTAT>,
  state: State,

  last_state: Option<State>,
  uarte: &'a mut Uarte<UARTE0>,
}

impl<'a, LFOSC, LFSTAT> Protocol<'a, LFOSC, LFSTAT> {
  pub fn new(esb: Esb<'a, LFOSC, LFSTAT>, uarte: &'a mut Uarte<UARTE0>) -> Self {
    Self {
      esb,
      state: State::Unpaired,

      last_state: None,
      uarte
    }
  }

  pub fn run(&mut self) {
    let next_state = match self.state {
      State::Unpaired => {
        self.uarte.write_fmt(format_args!("{:?}: Listening for pairing request ...\n", self.state));
        let rx_config = RxConfig::default();
        if let Err(err) = self.esb.start_rx(rx_config) {
          State::Error(Error::EsbError(err))
        }
        else {
          State::WaitPairingRequest
        }
      },
      State::WaitPairingRequest => {
        match self.esb.wait_rx() {
          Ok(()) => {
            self.uarte.write_fmt(format_args!("{:?}: Received pairing request ...\n", self.state));
            self.print_received_packet();
            let buf = &self.esb.get_rx_buffer()[2..];
            let code = (buf[0] as u16) << 8 | buf[1] as u16;
            match code {
              0x0908 => State::SendPairingResponse,
              _      => self.state,
            }
          },
          Err(error) => self.handle_esb_error(error),
        }
      },
      State::SendPairingResponse => {
        self.uarte.write_fmt(format_args!("{:?}: Sending pairing response ...\n", self.state));
        let buf = self.esb.get_tx_buffer();
        buf.copy_from_slice(&PAIRING_RESPONSE);
        let tx_config = TxConfig::default();
        if let Err(err) = self.esb.start_tx(tx_config) {
          State::Error(Error::EsbError(err))
        }
        else {
          State::WaitPairingResponseSent
        }
      },
      State::WaitPairingResponseSent => {
        match self.esb.wait_tx() {
          Ok(()) => {
            self.uarte.write_fmt(format_args!("{:?}: Pairing response sent ...\n", self.state));
            State::Paired
          },
          Err(error) => self.handle_esb_error(error),
        }
      },
      State::Paired => {
        if self.last_state.map(|s| s != State::WaitRequest).unwrap_or(true) {
          self.uarte.write_fmt(format_args!("{:?}: Listening for requests ...\n", self.state));
        }
        let rx_config = RxConfig::default();
        if let Err(err) = self.esb.start_rx(rx_config) {
          State::Error(Error::EsbError(err))
        }
        else {
          State::WaitRequest
        }
      },
      State::WaitRequest => {
        match self.esb.wait_rx() {
          Ok(()) => {
            let buf = &self.esb.get_rx_buffer()[2..];
            let code = (buf[0] as u16) << 8 | buf[1] as u16;
            match code {
              0x0908 => State::SendPairingResponse,
              unknown => {
                self.print_received_packet();
                self.uarte.write_str("Unknown request\n");
                State::Paired
              }
            }
          },
          Err(error) => self.handle_esb_error(error),
        }
      },
      State::Error(_) => self.state,
    };
    self.last_state = Some(self.state);
    self.state = next_state;
  }

  fn state_changed(&self) -> bool {
    self.last_state.map(|s| s != self.state).unwrap_or(true)
  }

  fn handle_esb_error(&self, error: nb::Error<EsbError>) -> State {
    match error {
      nb::Error::WouldBlock => self.state,
      nb::Error::Other(error) => State::Error(Error::EsbError(error))
    }
  }

  fn print_state(&mut self) {
    if self.last_state.map(|s| s != self.state).unwrap_or(true) {
      self.last_state = Some(self.state);
      self.uarte.write_fmt(format_args!("state: {:?}\n", self.state));
    }
  }

  fn print_received_packet(&mut self) {
    let buf = &self.esb.get_rx_buffer()[2..];
    let packet = self.esb.get_last_received_packet().unwrap();
    let no_ack = if packet.no_ack { 1 } else { 0 };
    drop(self.uarte.write_fmt(format_args!("[{} {:02} {} {}] ",
                                           packet.address.value(),
                                           packet.length,
                                           packet.pid,
                                           no_ack)));
    for b in buf.iter() {
      drop(self.uarte.write_fmt(format_args!("{:02x} ", *b)));
    }
    drop(self.uarte.write_char('\n'));
  }

  // ---------------- DEBUG ---------------------------------------

  pub fn sniffer(&mut self) {
    let rx_config = RxConfig::default().with_skip_ack(true);
    if let Err(err) = self.esb.start_rx(rx_config) {
      self.uarte.write_fmt(format_args!("Error: {:?}\n", err));
    }
    else {
      block!(self.esb.wait_rx());
      self.print_received_packet();
    }
  }

  pub fn receive_pairing_response(&mut self) {
    self.uarte.write_str("Receiving pairing response ...\n");
    let rx_config = RxConfig::default();
    if let Err(err) = self.esb.start_rx(rx_config) {
      self.uarte.write_fmt(format_args!("Error: {:?}\n", err));
    }
    else {
      block!(self.esb.wait_rx());
      self.uarte.write_str("Pairing response received\n");
      self.print_received_packet();
    }
  }

  pub fn send_pairing_response(&mut self, leds: &mut Leds) {
    self.uarte.write_str("Sending pairing response ...\n");
    let buf = self.esb.get_tx_buffer();
    buf.copy_from_slice(&PAIRING_RESPONSE);
    let tx_config = TxConfig::default().with_skip_ack(true);
    leds.blue.on();
    if let Err(err) = self.esb.start_tx(tx_config) {
      self.uarte.write_fmt(format_args!("Error: {:?}\n", err));
    }
    else {
      leds.red.on();
      block!(self.esb.wait_tx());
      leds.red.off();
      leds.blue.off();
      self.uarte.write_str("Pairing response sent\n");
    }
  }

  pub fn send_pairing_request(&mut self, leds: &mut Leds) {
    self.uarte.write_str("Sending pairing request ...\n");
    let buf = self.esb.get_tx_buffer();
    buf.copy_from_slice(&PAIRING_REQUEST);
    let tx_config = TxConfig::default().with_skip_ack(true);
    leds.blue.on();
    if let Err(err) = self.esb.start_tx(tx_config) {
      self.uarte.write_fmt(format_args!("Error: {:?}\n", err));
    }
    else {
      leds.red.on();
      block!(self.esb.wait_tx());
      leds.red.off();
      leds.blue.off();
      self.uarte.write_str("Pairing response sent\n");
    }
  }
}
