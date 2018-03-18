mod data_types;

use self::data_types::TypeInfo;
use super::timing::milliseconds;

use std::fs::File;
use std::path::Path;
use std::io;
use std::io::Write;
use std::marker::{ PhantomData, Sized };

use serde::Serialize;
use bincode::serialize;
use byteorder::*;
use time;

pub struct LogStream<T> where T: TypeInfo + Sized {
  file: File,
  _p: PhantomData<T>,
}

#[derive(Serialize, Debug)]
struct LogRecord<T> {
  time: f32,
  value: T,
}

impl<T> LogStream<T> where T: TypeInfo + Sized + Serialize {
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
    file.write_u64::<LittleEndian>(time::now().to_timespec().sec as u64)?;

    // Tags - we don't use them (yet?)
    file.write_u16::<LittleEndian>(0)?;

    Ok(LogStream { file, _p: PhantomData })
  }

  pub fn log(&mut self, value: T) -> io::Result<()> {
    let record = LogRecord { time: milliseconds(), value };
    Ok(self.file.write_all(&serialize(&record).unwrap())?)
  }
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

    let timestamp = reader.read_u64::<LittleEndian>().unwrap() as i64;
    assert!((time::now().to_timespec().sec - timestamp).abs() <= 1);

    assert_eq!(0, reader.read_u16::<LittleEndian>().unwrap());

    // Write something to the file and make sure the layout is correct
    stream.log(42).unwrap();

    assert!((reader.read_f32::<LittleEndian>().unwrap() - milliseconds()).abs() <= 10_f32);
    assert_eq!(42, reader.read_i32::<LittleEndian>().unwrap());
  }
}