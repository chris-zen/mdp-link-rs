use nrf52840_hal::{
  gpio::{
    Pin,
    Input,
    PullUp,
  },
};

/// A Button on the nRF52840-MDK board
pub struct Button(Pin<Input<PullUp>>);

impl Button {
  pub fn new<Mode>(pin: Pin<Mode>) -> Self {
    Button(pin.into_pullup_input())
  }
}
