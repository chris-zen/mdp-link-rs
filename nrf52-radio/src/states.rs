/*!

Possible states for the Radio

See [Product Specification](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.0.pdf): 6.20.5 Radio states

*/

pub enum State {
  Disabled,

  RxRumpUp,
  RxIdle,
  Rx,
  RxDisable,

  TxRumpUp,
  TxIdle,
  Tx,
  TxDisable,

  Unknown(u8),
}

impl State {
  pub fn from_value(value: u8) -> State {
    match value {
      0  => State::Disabled,
      1  => State::RxRumpUp,
      2  => State::RxIdle,
      3  => State::Rx,
      4  => State::RxDisable,
      9  => State::TxRumpUp,
      10 => State::TxIdle,
      11 => State::Tx,
      12 => State::TxDisable,
      _  => State::Unknown(value)
    }
  }
}
