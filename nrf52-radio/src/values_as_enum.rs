#[macro_export]
macro_rules! values_as_enum {
  ( $enum_name:ident, $enum_doc:expr, $( ( $value:expr, $name:ident $(, $doc:expr)? ) ),* ) => {
    #[derive(Clone, PartialEq)]
    #[doc=$enum_doc]
    pub enum $enum_name {
      $(
         $( #[doc=$doc] )?
         $name,
      )*
    }

    impl $enum_name {
      pub fn from(value: usize) -> Option<Self> {
        match value {
          $(
            $value => Some($enum_name::$name),
          )*

          _ => None
        }
      }

      pub fn value(&self) -> usize {
        match self {
          $(
            $enum_name::$name => $value,
          )*
        }
      }
    }
  };
}
