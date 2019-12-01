use core::fmt::Write;

use crate::hal::target::UARTE0;

use nrf52_esb::{Esb, RxConfig, Error as EsbError, TxConfig};
use nrf52840_hal::Uarte;
//use nrf52840_mdk::Leds;

// M01 asks P905 to connect
const PAIRING_REQUEST: [u8; 34] = [51, 2,
  0x09, 0x08, 0x62, 0x6d, 0xfa, 0x5d, 0x00, 0x01,
  0x5a, 0x73, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

// P905 responds the request from M01
const PAIRING_RESPONSE: [u8; 34] = [51, 2,
  0x09, 0x0d, 0x62, 0x6d, 0xfa, 0x5d, 0x00, 0x00,
  0x3e, 0xc2, 0x3b, 0x00, 0x0f, 0x78, 0x6d, 0xf9,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

// M01 asks P905 for data
const DATA_REQUEST: [u8; 34] = [51, 2,
  0x07, 0x06, 0x62, 0x6d, 0xfa, 0x5d, 0x00, 0x01,
  0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

// P905 answers with data
const DATA_RESPONSE: [u8; 34] = [51, 2,
  0x07, 0x1b, 0x00, 0x00, 0x02, 0x55, 0x00, 0x01,
  0x21, 0x38, 0x00, 0x65, 0x00, 0x40, 0x04, 0x00,
  0x30, 0x04, 0x00, 0x30, 0x03, 0x00, 0x30, 0x04,
  0x00, 0x30, 0x00, 0x10, 0x00, 0xe9, 0x00, 0x00];


#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Error {
  EsbError(EsbError),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum State {
  Unpaired,
  SendPairingRequest,
  WaitPairingRequest,
  ReceivePairingResponse,
  WaitPairingResponse,
  SendDataRequest,
  WaitDataRequest,
  ReceiveDataResponse,
  WaitDataResponse,
  Error(Error),
}

pub struct Protocol<'a, LFOSC, LFSTAT> {
  esb: Esb<'a, LFOSC, LFSTAT>,
  state: State,
  tx_config: TxConfig,
  rx_config: RxConfig,
  pid: u8,

  last_state: Option<State>,
  uarte: &'a mut Uarte<UARTE0>,
}

impl<'a, LFOSC, LFSTAT> Protocol<'a, LFOSC, LFSTAT> {
  pub fn new(esb: Esb<'a, LFOSC, LFSTAT>, uarte: &'a mut Uarte<UARTE0>) -> Self {
    Self {
      esb,
      state: State::Unpaired,
      tx_config: TxConfig::default(),
      rx_config: RxConfig::default(),
      pid: 0,

      last_state: None,
      uarte
    }
  }

  fn new_pid(&mut self) -> u8 {
    let pid = self.pid;
    self.pid += 1;
    self.pid = self.pid & 0x03;
    pid
  }

  pub fn run(&mut self) {
    let next_state = match self.state {
      State::Unpaired => {
        drop(self.uarte.write_fmt(format_args!("{:?}: Looking for P905 ...\n", self.state)));
        State::SendPairingRequest
      },
      State::SendPairingRequest => {
        drop(self.uarte.write_fmt(format_args!("{:?}: Sending pairing request ...\n", self.state)));
        let pid = self.new_pid();
        let buf = self.esb.get_tx_buffer();
        buf.copy_from_slice(&PAIRING_REQUEST);
        buf[1] |= pid << 1;
        if let Err(err) = self.esb.start_tx(self.tx_config) {
          State::Error(Error::EsbError(err))
        }
        else {
          State::WaitPairingRequest
        }
      },
      State::WaitPairingRequest => {
        match self.esb.wait_tx() {
          Ok(()) => {
//            self.uarte.write_fmt(format_args!("{:?}: Pairing request sent ...\n", self.state));
            State::ReceivePairingResponse
          },
          Err(error) => self.handle_esb_error(error),
        }
      },
      State::ReceivePairingResponse => {
//        self.uarte.write_fmt(format_args!("{:?}: Listening for pairing response ...\n", self.state));
        if let Err(err) = self.esb.start_rx(self.rx_config) {
          drop(self.uarte.write_str("Error receiving pairing response"));
          State::Error(Error::EsbError(err))
        }
        else {
          State::WaitPairingResponse
        }
      },
      State::WaitPairingResponse => {
        match self.esb.wait_rx() {
          Ok(()) => {
            drop(self.uarte.write_char('.'));
            let buf = &self.esb.get_rx_buffer()[2..];
            let code = (buf[0] as u16) << 8 | buf[1] as u16;
            match code {
              0x090d => State::SendDataRequest,
              _ => {
                drop(self.uarte.write_str("Unknown request\n"));
                self.print_received_packet();
                State::SendPairingRequest
              }
            }
          },
          Err(error) => self.handle_esb_error(error),
        }
      },
      State::SendDataRequest => {
        drop(self.uarte.write_fmt(format_args!("{:?}: Sending data request ...\n", self.state)));
        let buf = self.esb.get_tx_buffer();
        buf.copy_from_slice(&DATA_REQUEST);
        if let Err(err) = self.esb.start_tx(self.tx_config) {
          State::Error(Error::EsbError(err))
        }
        else {
          State::WaitDataRequest
        }
      },
      State::WaitDataRequest => {
        match self.esb.wait_tx() {
          Ok(()) => {
//            self.uarte.write_fmt(format_args!("{:?}: Data request sent ...\n", self.state));
            State::ReceiveDataResponse
          },
          Err(error) => self.handle_esb_error(error),
        }
      },
      State::ReceiveDataResponse => {
//        self.uarte.write_fmt(format_args!("{:?}: Listening for data response ...\n", self.state));
        if let Err(err) = self.esb.start_rx(self.rx_config) {
          State::Error(Error::EsbError(err))
        }
        else {
          State::WaitDataResponse
        }
      },
      State::WaitDataResponse => {
        match self.esb.wait_rx() {
          Ok(()) => {
            let buf = &self.esb.get_rx_buffer()[2..];
            let code = (buf[0] as u16) << 8 | buf[1] as u16;
            match code {
              0x071b => State::SendDataRequest,
              _ => {
                drop(self.uarte.write_str("Unknown request\n"));
                self.print_received_packet();
                State::SendDataRequest
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

  fn handle_esb_error(&self, error: nb::Error<EsbError>) -> State {
    match error {
      nb::Error::WouldBlock => self.state,
      nb::Error::Other(error) => State::Error(Error::EsbError(error))
    }
  }

//  fn state_changed(&self) -> bool {
//    self.last_state.map(|s| s != self.state).unwrap_or(true)
//  }

//  fn print_state(&mut self) {
//    if self.last_state.map(|s| s != self.state).unwrap_or(true) {
//      self.last_state = Some(self.state);
//      drop(self.uarte.write_fmt(format_args!("state: {:?}\n", self.state)));
//    }
//  }

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
}
