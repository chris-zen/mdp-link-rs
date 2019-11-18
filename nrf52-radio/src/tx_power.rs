///!
///! Output power
///!
///! See [Product Specification](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.0.pdf): 6.20.14.11 TXPOWER
///!

pub enum TxPower {
  /// +8 dBm
  Pos8dBm,
  /// +7 dBm
  Pos7dBm,
  /// +6 dBm
  Pos6dBm,
  /// +5 dBm
  Pos5dBm,
  /// +4 dBm
  Pos4dBm,
  /// +3 dBm
  Pos3dBm,
  /// +2 dBm
  Pos2dBm,
  ///  0 dBm
  ZerodBm,
  /// -4 dBm
  Neg4dBm,
  /// -8 dBm
  Neg8dBm,
  /// -12 dBm
  Neg12dBm,
  /// -16 dBm
  Neg16dBm,
  /// -20 dBm
  Neg20dBm,
  /// -40 dBm
  Neg40dBm,
  /// Custom value
  Custom(u8)
}

impl TxPower {
  pub fn value(&self) -> u32 {
    match self {
      TxPower::Pos8dBm  => 0x08,
      TxPower::Pos7dBm  => 0x07,
      TxPower::Pos6dBm  => 0x06,
      TxPower::Pos5dBm  => 0x05,
      TxPower::Pos4dBm  => 0x04,
      TxPower::Pos3dBm  => 0x03,
      TxPower::Pos2dBm  => 0x02,
      TxPower::ZerodBm  => 0x00,
      TxPower::Neg4dBm  => 0xfc,
      TxPower::Neg8dBm  => 0xf8,
      TxPower::Neg12dBm => 0xf4,
      TxPower::Neg16dBm => 0xf0,
      TxPower::Neg20dBm => 0xec,
      TxPower::Neg40dBm => 0xd8,
      TxPower::Custom(custom) => *custom as u32,
    }
  }
}
