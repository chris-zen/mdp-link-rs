pub enum BaseAddresses {
  TwoBytes(u16, u16),
  ThreeBytes(u32, u32),
  FourBytes(u32, u32),
}

impl BaseAddresses {
  /// Two bytes addresses with the same value for base0 and base1
  pub fn from_same_two_bytes(a: [u8; 2]) -> Self {
    let addr = u16::from(a[0]) << 8 | u16::from(a[1]);
    BaseAddresses::TwoBytes(addr, addr)
  }

  /// Three bytes addresses with the same value for base0 and base1
  pub fn from_same_three_bytes(a: [u8; 3]) -> Self {
    let addr = u32::from(a[0]) << 16 | u32::from(a[1]) << 8 | u32::from(a[2]);
    BaseAddresses::ThreeBytes(addr, addr)
  }

  /// Four bytes addresses with the same value for base0 and base1
  pub fn from_same_four_bytes(a: [u8; 4]) -> Self {
    let addr = u32::from(a[0]) << 24 | u32::from(a[1]) << 16 | u32::from(a[2]) << 8 | u32::from(a[3]);
    BaseAddresses::ThreeBytes(addr, addr)
  }
}
