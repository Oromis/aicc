pub trait TypeInfo {
  fn type_str() -> &'static str;
}

impl TypeInfo for i32 {
  fn type_str() -> &'static str {
    "int"
  }
}

impl TypeInfo for f32 {
  fn type_str() -> &'static str {
    "real"
  }
}

impl TypeInfo for bool {
  fn type_str() -> &'static str {
    "bool"
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_provides_type_info_for_i32() {
    assert_eq!("int", i32::type_str());
  }

  #[test]
  fn it_provides_type_info_for_f32() {
    assert_eq!("real", f32::type_str());
  }

  #[test]
  fn it_provides_type_info_for_bool() {
    assert_eq!("bool", bool::type_str());
  }
}