use logging::data_types::TypeInfo;
use logging::{ self, LogStream };
use variable::Variable;

use std::io;
use serde::Serialize;
use std::fmt::Display;

pub fn create_log_for<'a, T>(var: &mut Variable<'a, T>, name: &str) -> io::Result<()>
  where T: PartialEq + TypeInfo + Serialize + Display + Copy + 'a {
  let path = logging::get_timestamped_path()?;
  let mut log_stream: LogStream<T> = LogStream::new(
    &path.join(format!("{}.ebl", name)),
    &format!("drive-core_{}", name))?;

  // Log the variable's initial value
  log_stream.log(*var.value())?;

  var.add_listener(move |val| {
    match log_stream.log(*val) {
      Ok(()) => {},
      Err(e) => println!("Failed to log value {}: {:?}", val, e)
    }
  });

  Ok(())
}