/*!

Data rate and modulation

See [Product Specification](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.0.pdf): 6.20.14.12 MODE

*/

pub enum Mode {
  Nrf1Mbit,            // 1 Mbit/s Nordic proprietary radio mode
  Nrf2Mbit,            // 2 Mbit/s Nordic proprietary radio mode
  Ble1Mbit,            // 1 Mbit/s BLE
  Ble2Mbit,            // 2 Mbit/s BLE
  BleLongRange125Kbit, // Long range 125 kbit/s TX, 125 kbit/s and 500 kbit/s RX
  BleLongRange500Kbit, // Long range 500 kbit/s TX, 125 kbit/s and 500 kbit/s RX
  Ieee802154At250Kbit, // IEEE 802.15.4-2006 250 kbit/s
}

impl Mode {
  pub fn value(&self) -> u32 {
    match self {
      Mode::Nrf1Mbit => 0,
      Mode::Nrf2Mbit => 1,
      Mode::Ble1Mbit => 3,
      Mode::Ble2Mbit => 4,
      Mode::BleLongRange125Kbit => 5,
      Mode::BleLongRange500Kbit => 6,
      Mode::Ieee802154At250Kbit => 15,
    }
  }
}
