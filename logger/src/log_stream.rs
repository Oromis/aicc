use std::fs;
use std::fs::File;
use std::path::{ Path, PathBuf };
use std::time::{ SystemTime, UNIX_EPOCH };
use std::io;
use std::io::Write;
use std::marker::{ PhantomData };

use serde::Serialize;
use bincode::serialize;
use byteorder::*;
use chrono::Local;

use util::timing::milliseconds;
use util::logging::data_types::TypeInfo;

pub trait LogStreamBase {
  fn log_generic(&mut self, val: f32) -> io::Result<()>;
}

pub struct LogStream<T> {
  file: File,
  _p: PhantomData<T>,
}

#[derive(Serialize, Debug)]
struct LogRecord<T> {
  time: f32,
  value: T,
}

impl<T> LogStream<T> where T: TypeInfo + Serialize {
  pub fn new(path: &Path, name: &str) -> io::Result<LogStream<T>> {
    let mut file = File::create(path)?;

    // Write file header:
    // Version
    file.write_i32::<LittleEndian>(1 << 16)?;

    // ID (not used by us at the moment)
    file.write_u16::<LittleEndian>(0)?;

    // Name
    file.write_u16::<LittleEndian>(name.len() as u16)?;
    file.write_all(name.as_bytes())?;

    // Type
    let type_str = T::type_str();
    file.write_u16::<LittleEndian>(type_str.len() as u16)?;
    file.write_all(type_str.as_bytes())?;

    // Time
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    file.write_u64::<LittleEndian>(now.as_secs())?;

    // Tags - we don't use them (yet?)
    file.write_u16::<LittleEndian>(0)?;

    Ok(LogStream { file, _p: PhantomData })
  }

  pub fn log(&mut self, value: T) -> io::Result<()> {
    let record = LogRecord { time: milliseconds(), value };
    Ok(self.file.write_all(&serialize(&record).unwrap())?)
  }
}

impl<T> LogStreamBase for LogStream<T> where T: TypeInfo + Serialize {
  fn log_generic(&mut self, val: f32) -> io::Result<()> {
    self.log(T::from_f32(val))
  }
}

pub fn get_timestamped_path() -> io::Result<PathBuf> {
  let time = Local::now().format("%Y-%m-%d_%H-%M").to_string();
  let path = Path::new("/var/log/aicc").join(time);
  fs::create_dir_all(&path)?;
  Ok(path)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Read;
  use tempdir::TempDir;

  #[test]
  fn it_creates_and_opens_a_new_file() {
    let tmp = TempDir::new("log").unwrap();
    let file_name = "file.log";
    let var_name = "test_var";

    assert!(!tmp.path().join(file_name).exists());
    let mut stream: LogStream<i32> = LogStream::new(&tmp.path().join(file_name), var_name).unwrap();
    assert!(tmp.path().join(file_name).exists());

    // Make sure that the byte layout is correct
    let mut reader = File::open(tmp.path().join(file_name)).unwrap();

    // Version
    assert_eq!(1 << 16, reader.read_i32::<LittleEndian>().unwrap());

    // ID
    assert_eq!(0, reader.read_u16::<LittleEndian>().unwrap());

    // Name
    assert_eq!(var_name.len(), reader.read_u16::<LittleEndian>().unwrap() as usize);

    let mut name_buffer = vec![0; var_name.len()];
    reader.read_exact(&mut name_buffer).unwrap();
    assert_eq!(var_name.as_bytes(), &name_buffer[..]);

    assert_eq!(3, reader.read_u16::<LittleEndian>().unwrap() as usize);

    let mut type_buffer = vec![0; "int".len()];
    reader.read_exact(&mut type_buffer).unwrap();
    assert_eq!("int".as_bytes(), &type_buffer[..]);

    let timestamp = reader.read_u64::<LittleEndian>().unwrap();
    assert!(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - timestamp <= 1);

    assert_eq!(0, reader.read_u16::<LittleEndian>().unwrap());

    // Write something to the file and make sure the layout is correct
    stream.log(42).unwrap();

    assert!((reader.read_f32::<LittleEndian>().unwrap() - milliseconds()).abs() <= 10_f32);
    assert_eq!(42, reader.read_i32::<LittleEndian>().unwrap());
  }
}