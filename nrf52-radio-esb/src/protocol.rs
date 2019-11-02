
#[derive(Clone, Copy)]
pub enum Protocol {
  DynamicPayloadLength(u8), // max length
  FixedPayloadLength(u8),   // fixed length
}
