use crate::values_as_enum;

/// Packet configuration for registers PCFN0 and PCFN1
///
/// [Product Specification](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.0.pdf):
/// - 6.20.14.13 PCNF0
/// - 6.20.14.14 PCNF1
///
pub struct PacketConfig {

  /// Length on air of LENGTH field in number of bits.
  pub length_bits: Option<u8>,

  /// Length on air of S0 field in number of bytes.
  pub s0_byte_included: Option<bool>,

  /// Length on air of S1 field in number of bits.
  pub s1_len: Option<S1Length>,

  /// Include or exclude S1 field in RAM.
  pub s1_include_in_ram: Option<S1IncludeInRam>,

  // TODO CILEN Length of code indicator - long range

  /// Length of preamble on air.
  pub preamble_len: Option<PreambleLength>,

  /// Indicates if LENGTH field contains CRC or not.
  pub crc_included_in_length: Option<bool>,

  // TODO TERMLEN Length of TERM field in Long Range operation

  /// Maximum length of packet payload in bytes. Allowed values between 0 and 255.
  /// If the packet payload is larger than `max_len`,
  /// then the radio will truncate the payload to it.
  pub max_bytes: Option<u8>,

  /// Static length in number of bytes. Allowed values between 0 and 255.
  /// The static length parameter is added to the total length of the payload when
  /// sending and receiving packets, e.g. if the static length is set to N the radio will
  /// receive or send N bytes more than what is defined in the LENGTH field of the packet.
  pub static_bytes: Option<u8>,

  /// On air endianness of packet, this applies to the S0, LENGTH, S1 and the PAYLOAD fields.
  pub endianess: Option<Endianess>,

  /// Enable or disable packet whitening
  pub whitening_enabled: Option<bool>,
}

impl Default for PacketConfig {
  fn default() -> Self {
    PacketConfig {
      length_bits: None,
      s0_byte_included: None,
      s1_len: None,
      s1_include_in_ram: None,
      preamble_len: None,
      crc_included_in_length: None,
      max_bytes: None,
      static_bytes: None,
      endianess: None,
      whitening_enabled: None,
    }
  }
}

impl PacketConfig {
  pub fn with_length_bits(self, bits: u8) -> Self {
    Self { length_bits: Some(bits), .. self }
  }

  pub fn with_s0_byte_included(self, included: bool) -> Self {
    Self { s0_byte_included: Some(included), .. self }
  }

  pub fn with_s1_len(self, bits: S1Length) -> Self {
    Self { s1_len: Some(bits), .. self }
  }

  pub fn with_s1_include_in_ram(self, include: S1IncludeInRam) -> Self {
    Self { s1_include_in_ram: Some(include), .. self }
  }

  pub fn with_preamble_len(self, bits: PreambleLength) -> Self {
    Self { preamble_len: Some(bits), .. self }
  }

  pub fn with_crc_included_in_length(self, included: bool) -> Self {
    Self { crc_included_in_length: Some(included), .. self }
  }

  pub fn with_max_bytes(self, bytes: u8) -> Self {
    Self { max_bytes: Some(bytes), .. self }
  }

  pub fn with_static_bytes(self, bytes: u8) -> Self {
    Self { static_bytes: Some(bytes), .. self }
  }

  pub fn with_endianess(self, endianess: Endianess) -> Self {
    Self { endianess: Some(endianess), .. self }
  }

  pub fn with_whitening_enabled(self, enabled: bool) -> Self {
    Self { whitening_enabled: Some(enabled), .. self }
  }
}

values_as_enum!(
  S1Length, "Possible values for the length in bits of the S1 field",
  (0, Of0Bits), (1, Of1Bits), (2, Of2Bits), (3, Of3Bits),
  (4, Of4Bits), (5, Of5Bits), (6, Of6Bits), (7, Of7Bits),
  (8, Of8Bits), (9, Of9Bits), (10, Of10Bits), (11, Of11Bits),
  (12, Of12Bits), (13, Of13Bits), (14, Of14Bits), (15, Of15Bits)
);

values_as_enum!(
  S1IncludeInRam, "Whether to include or not the S1 field in RAM",
  (0, Automatic, "Include S1 field in RAM only if S1LEN > 0"),
  (1, Always, "Always include S1 field in RAM independent of S1LEN")
);

values_as_enum!(
  PreambleLength, "Length of preamble on air",
  (0, Of8Bits, "8-bit preamble"),
  (1, Of16Bits, "16-bit preamble"),
  (2, Of32Bits, "32-bit zero preamble used for IEEE 802.15.4"),
  (3, ForLongRange, "Preamble used for BLE long range")
);

values_as_enum!(
  Endianess, "On air endianness of packet, this applies to the S0, LENGTH,S1 and the PAYLOAD fields",
  (0, LittleEndian, "Least significant bit on air first"),
  (1, BigEndian, "Most significant bit on air first")
);
