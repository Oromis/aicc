use std::io;
use std::collections::HashMap;
use std::path::Path;

use log_stream::*;

pub struct StreamManager {
  stream_by_id: HashMap<i32, Box<LogStreamBase>>,
  id_by_name: HashMap<String, i32>,
  next_id: i32,
  base_path: Box<Path>,
}

impl StreamManager {
  pub fn new() -> io::Result<StreamManager> {
    let path = get_timestamped_path()?;

    Ok(StreamManager {
      stream_by_id: HashMap::new(),
      id_by_name: HashMap::new(),
      next_id: 0,
      base_path: path.into(),
    })
  }

  pub fn register(&mut self, name: String, typename: String) -> io::Result<i32> {
    // See if we know this variable name already
    {
      if let Some(id) = self.id_by_name.get(&name) {
        return Ok(*id)
      }
    }

    // Does not exist yet. Let's create a new instance.
    let id = self.next_id;
    self.next_id = self.next_id + 1;
    self.id_by_name.insert(name.clone(), id);

    let path = self.base_path.to_path_buf().join(format!("{}.ebl", name));

    let stream: Box<LogStreamBase> = match typename.as_ref() {
      "int" => {
        let s: LogStream<i32> = LogStream::new(&path, &name)?;
        Box::new(s)
      },
      "bool" => {
        let s: LogStream<bool> = LogStream::new(&path, &name)?;
        Box::new(s)
      },
      _ => {
        let s: LogStream<f32> = LogStream::new(&path, &name)?;
        Box::new(s)
      },
    };

    self.stream_by_id.insert(id.clone(), stream);
    Ok(id)
  }

  pub fn log(&mut self, id: i32, val: f32) -> io::Result<()> {
    match self.stream_by_id.get_mut(&id) {
      Some(stream) => stream.log_generic(val),
      None => Err(io::Error::new(io::ErrorKind::Other, "Stream not registered."))
    }
  }
}
