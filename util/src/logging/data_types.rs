pub trait TypeInfo {
  fn type_str() -> &'static str;
  fn from_f32(val: f32) -> Self;
}

impl TypeInfo for i32 {
  fn type_str() -> &'static str {
    "int"
  }
  fn from_f32(val: f32) -> Self {
    val as i32
  }
}

impl TypeInfo for f32 {
  fn type_str() -> &'static str {
    "real"
  }
  fn from_f32(val: f32) -> Self {
    val
  }
}

impl TypeInfo for bool {
  fn type_str() -> &'static str {
    "bool"
  }
  fn from_f32(val: f32) -> Self {
    val != 0.0
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