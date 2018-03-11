// Enum of the messages this service accepts
#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
  SetSteering(f32),
  SetThrottle(f32),
  Bye,
}

#[cfg(test)]
mod tests {
  use super::*;
  use bincode::{ serialize, deserialize };

  #[test]
  fn serialize_set_steering() {
    let expected = -0.987f32;
    let vec = serialize(&MessageType::SetSteering(expected)).unwrap();
    assert_eq!(vec.len(), 8);

    let msg = deserialize(&vec[..]).unwrap();
    match msg {
      MessageType::SetSteering(actual) => assert_eq!(actual, expected),
      _ => panic!("Deserialized the wrong value")
    }
  }

  #[test]
  fn serialize_set_throttle() {
    let value = 132.321f32;
    let vec = serialize(&MessageType::SetThrottle(value)).unwrap();
    assert_eq!(vec.len(), 8);

    let msg = deserialize(&vec[..]).unwrap();
    match msg {
      MessageType::SetThrottle(actual) => assert_eq!(actual, value),
      _ => panic!("Deserialized the wrong value")
    }
  }

  #[test]
  fn deserialize_set_steering() {
    let raw = [0, 0, 0, 0, 0xd5, 0xd0, 0x4d, 0x44];
    let msg = deserialize(&raw).unwrap();
    match msg {
      MessageType::SetSteering(actual) => assert_eq!(actual, 823.263f32),
      _ => panic!("Deserialized the wrong value")
    }
  }

  #[test]
  fn deserialize_set_throttle() {
    let raw = [1, 0, 0, 0, 0x87, 0x16, 0x39, 0xbf];
    let msg = deserialize(&raw).unwrap();
    match msg {
      MessageType::SetThrottle(actual) => assert_eq!(actual, -0.723f32),
      _ => panic!("Deserialized the wrong value")
    }
  }

  #[test]
  fn deserialize_additional_bytes() {
    let raw = [1, 0, 0, 0, 0x42, 0x60, 0x05, 0x3f, 1, 0, 0, 0];
    let msg = deserialize(&raw).unwrap();
    match msg {
      MessageType::SetThrottle(actual) => assert_eq!(actual, 0.521f32),
      _ => panic!("Deserialized the wrong value")
    }
  }

  #[test]
  fn deserialize_bye() {
    let raw = [2, 0, 0, 0];
    let msg = deserialize(&raw).unwrap();
    match msg {
      MessageType::Bye => {},
      _ => panic!("Deserialized the wrong value")
    }
  }
}