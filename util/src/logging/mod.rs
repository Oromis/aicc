pub mod data_types;

use std::io;
use std::io::{ Write };
use std::net::{ TcpStream };
use std::fmt::Display;
use std::cell::RefCell;
use std::rc::Rc;

use serde::Serialize;
use bincode::{ serialize, ErrorKind, deserialize_from };

use mesh::Service;
use logging::data_types::TypeInfo;
use variable::{ Variable, ListenerError };
use messages::logger::MessageType;

pub struct LogConnection {
  socket: Rc<RefCell<TcpStream>>,
}

impl LogConnection {
  pub fn new() -> io::Result<LogConnection> {
    let addr = "localhost:".to_owned() + &Service::Logger.port().to_string();
    let socket = TcpStream::connect(addr)?;

    Ok(LogConnection { socket: Rc::new(RefCell::new(socket)) })
  }

  pub fn log_variable<'a, T>(&mut self, var: &mut Variable<'a, T>, name: &str) -> io::Result<()>
    where T: PartialEq + TypeInfo + Serialize + Display + Copy + Into<f32> + 'a {
    // Attempt to register our log variable with the logging service
    match serialize(&MessageType::Register(name.to_string(), T::type_str().to_string())) {
      Ok(msg) => {
        self.socket.borrow_mut().write(&msg[..])?;
      },
      Err(e) => {
        match *e {
          ErrorKind::Io(err) => return Err(err),
          _ => return Err(io::Error::new(io::ErrorKind::Other, *e))
        }
      }
    }

    // Wait for the response
    let msg: MessageType = match deserialize_from(&mut *self.socket.borrow_mut()) {
      Ok(msg) => msg,
      Err(e) => {
        match *e {
          ErrorKind::Io(err) => return Err(err),
          _ => return Err(io::Error::new(io::ErrorKind::Other, *e))
        }
      }
    };

    // Got the response => handle it
    let log_id = match msg {
      // All fine, we got accepted and here is our ID
      MessageType::Acknowledge(id) => id,
      _ => return Err(io::Error::new(
        io::ErrorKind::Other, "Unexpected response from logging service"))
    };

    let socket = self.socket.clone();

    // Attach a logging listener to our variable
    var.add_listener(move |val| {
      match serialize(&MessageType::Log(log_id, (*val).into())) {
        Ok(msg) => {
          match socket.borrow_mut().write(&msg[..]) {
            Ok(_) => {},
            Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset
              || e.kind() == io::ErrorKind::BrokenPipe => {
              return Err(ListenerError::RemoveListener);
            }
            // Don't crash the program in case of an error, just write something to the console.
            // Logging is not ciritcal.
            Err(e) => println!("Failed to send log message: {:?}", e),
          };
        },
        Err(e) => {
          println!("Failed to serialize log message: {:?}", e);
        }
      };
      Ok(())
    });

//  let path = logging::get_timestamped_path()?;
//  let mut log_stream: LogStream<T> = LogStream::new(
//    &path.join(format!("{}.ebl", name)),
//    &format!("drive-core_{}", name))?;
//
//  // Log the variable's initial value
//  log_stream.log(*var.value())?;
//
//  var.add_listener(move |val| {
//    match log_stream.log(*val) {
//      Ok(()) => {},
//      Err(e) => println!("Failed to log value {}: {:?}", val, e)
//    }
//  });

    Ok(())
  }
}
