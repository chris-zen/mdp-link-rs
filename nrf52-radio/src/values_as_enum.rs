#[macro_export]
macro_rules! values_as_enum {
  ( $enum_name:ident, $enum_doc:expr, $( ( $value:expr, $name:ident $(, $doc:expr)? ) ),* ) => {
    #[derive(Debug, Clone, PartialEq, Copy)]
    #[doc=$enum_doc]
    pub enum $enum_name {
      $(
         $( #[doc=$doc] )?
         $name,
      )*
    }

    impl $enum_name {
      pub fn from(value: u32) -> Option<Self> {
        match value {
          $(
            $value => Some($enum_name::$name),
          )*

          _ => None
        }
      }

      pub fn value(&self) -> u32 {
        match self {
          $(
            $enum_name::$name => $value,
          )*
        }
      }
    }
  };
}
