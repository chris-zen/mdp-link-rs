use nrf52840_hal::{
  prelude::*,
  gpio::{
    Pin,
    Output,
    PushPull,
    Level,
  },
};

use embedded_hal::digital::v1::StatefulOutputPin;

/// The LEDs on the nRF52840-MDK board
pub struct Leds {
  /// nRF52840-MDK: LED red
  pub red: Led,

  /// nRF52840-MDK: LED green
  pub green: Led,

  /// nRF52840-MDK: LED blue
  pub blue: Led,
}

/// An LED on the nRF52840-MDK board
pub struct Led(Pin<Output<PushPull>>);

impl Led {
  pub fn new<Mode>(pin: Pin<Mode>) -> Self {
    Led(pin.into_push_pull_output(Level::High))
  }

  /// Enable the LED
  pub fn on(&mut self) {
    #[allow(deprecated)]
        self.0.set_low()
  }

  /// Disable the LED
  pub fn off(&mut self) {
    #[allow(deprecated)]
        self.0.set_high()
  }

  /// Invert the LED
  pub fn invert(&mut self) {
    if self.0.is_set_low() {
      self.off()
    }
    else {
      self.on()
    }
  }
}
