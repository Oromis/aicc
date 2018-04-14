#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
  Register(String, String),   // Name and type
  Acknowledge(i32),           // Log ID
  Log(i32, f32),              // Log ID and value
}

#[cfg(test)]
mod tests {
  use super::*;
  use bincode::{ serialize, deserialize };

  #[test]
  fn serialize_register() {
    let out = MessageType::Register("test_foo".to_string(), "int".to_string());
    let vec = serialize(&out).unwrap();

    let msg = deserialize(&vec[..]).unwrap();
    match msg {
      MessageType::Register(name, typename) => {
        assert_eq!("test_foo", name);
        assert_eq!("int", typename);
      },
      _ => panic!("Deserialized the wrong value")
    }
  }

  #[test]
  fn serialize_acknowledge() {
    let out = MessageType::Acknowledge(42);
    let vec = serialize(&out).unwrap();

    let msg = deserialize(&vec[..]).unwrap();
    match msg {
      MessageType::Acknowledge(log_id) => {
        assert_eq!(42, log_id);
      },
      _ => panic!("Deserialized the wrong value")
    }
  }

  #[test]
  fn serialize_log_message() {
    let out = MessageType::Log(42, 12.34f32);
    let vec = serialize(&out).unwrap();

    let msg = deserialize(&vec[..]).unwrap();
    match msg {
      MessageType::Log(log_id, val) => {
        assert_eq!(42, log_id);
        assert_eq!(12.34f32, val);
      },
      _ => panic!("Deserialized the wrong value")
    }
  }
}