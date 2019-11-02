
pub enum PacketPreamble {
  Length8Bits,
  Length16Bits,
  Length32Bits,
  LongRange,
}

pub enum PacketEndianess {
  LittleEndian,
  BigEndian,
}
