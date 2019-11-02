
#[derive(Clone, Copy)]
pub enum Protocol {
  /// Dynamic Payload up to a maximum number of bytes
  DynamicPayloadLength(u8),

  /// Fixed Payload of a given number of bytes
  FixedPayloadLength(u8),
}

impl Protocol {
  /// Dynamic Payload up to a maximum number of bytes
  pub fn dynamic_payload(max_length: u8) -> Self {
    assert!(max_length <= 252);
    Protocol::DynamicPayloadLength(max_length)
  }

  /// Fixed Payload of a given number of bytes
  pub fn fixed_payload(length: u8) -> Self {
    assert!(length <= 32);
    Protocol::FixedPayloadLength(length)
  }
}
