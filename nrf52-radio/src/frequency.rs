/// 6.20.14.10 FREQUENCY
pub enum Frequency {
  Default2400MHz(u8),
  Low2360MHz(u8),
}

impl Frequency {
  pub fn from_2400mhz_channel(channel: u8) -> Self {
    assert!(channel <= 100);
    Frequency::Default2400MHz(channel)
  }

  pub fn from_2360mhz_channel(channel: u8) -> Self {
    assert!(channel <= 100);
    Frequency::Low2360MHz(channel)
  }
}
