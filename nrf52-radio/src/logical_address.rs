use crate::values_as_enum;

values_as_enum!(
  LogicalAddress, "Logical address to be used when transmitting a packet.",
  (0, Of0), (1, Of1), (2, Of2), (3, Of3),
  (4, Of4), (5, Of5), (6, Of6), (7, Of7)
);
