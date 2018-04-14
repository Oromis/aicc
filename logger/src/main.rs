#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate bincode;
extern crate byteorder;
extern crate chrono;
extern crate mio;

extern crate messages;
extern crate util;

mod log_stream;
mod stream_manager;

use std::io;
use std::io::Write;
use std::collections::HashMap;

use messages::logger::MessageType;
use bincode::{ Result, ErrorKind };

use util::mesh::Service;
use stream_manager::StreamManager;

use bincode::{ serialize, deserialize_from };
use mio::*;
use mio::net::{TcpListener, TcpStream};

const MAX_CLIENTS: usize = 64;

const TOKEN_ACCEPT: Token = Token(MAX_CLIENTS);

fn find_unused_token(sockets: &HashMap<Token, TcpStream>) -> Option<Token> {
  let client_count = sockets.len();
  if client_count >= MAX_CLIENTS {
    return None;
  } else {
    for i in 0..MAX_CLIENTS {
      let token = Token((client_count + i) % MAX_CLIENTS);
      if sockets.get(&token).is_none() {
        return Some(token);
      }
    }
    return None;
  }
}

fn register_stream(name: String,
                   typename: String,
                   socket: &mut TcpStream,
                   stream_manager: &mut StreamManager) -> io::Result<i32> {
  let id = stream_manager.register(name, typename)?;
  match serialize(&MessageType::Acknowledge(id)) {
    Ok(msg) => {
      socket.write(&msg[..])?;
      Ok(id)
    },
    Err(e) => {
      match *e {
        ErrorKind::Io(err) => return Err(err),
        _ => return Err(io::Error::new(io::ErrorKind::Other, *e))
      }
    }
  }
}

fn main() {
  let mut clients = HashMap::new();

  let addr = (&("0.0.0.0:".to_owned() + &Service::Logger.port().to_string())).parse().unwrap();
  let server = TcpListener::bind(&addr).unwrap();

  let mut stream_manager = StreamManager::new().unwrap();

  // Create a poll instance
  let poll = Poll::new().unwrap();

  // Start listening for incoming connections
  poll.register(&server, TOKEN_ACCEPT, Ready::readable(),
                PollOpt::edge()).unwrap();

  // Create storage for events
  let mut events = Events::with_capacity(1024);

  loop {
    poll.poll(&mut events, None).unwrap();

    for event in events.iter() {
      match event.token() {
        TOKEN_ACCEPT => {
          // Perform operations in a loop until `WouldBlock` is
          // encountered.
          loop {
            match server.accept() {
              Ok((socket, _)) => {
                match find_unused_token(&clients) {
                  Some(token) => {
                    // Register the new socket w/ poll
                    poll.register(&socket,
                                  token,
                                  Ready::readable(),
                                  PollOpt::edge()).unwrap();

                    println!("New client connected with token {:?}", token);

                    // Store the socket
                    clients.insert(token, socket);
                  },
                  None => {
                    println!("Rejecting client because the client limit has been reached.")
                  }
                }
              }
              Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Socket is not ready anymore, stop accepting
                break;
              }
              e => panic!("err={:?}", e), // Unexpected error
            }
          }
        },
        token => {
          // Always operate in a loop
          let mut remove_socket = false;
          loop {
            let mut socket = clients.get_mut(&token).unwrap();
            let result: Result<MessageType> = deserialize_from(&mut socket);
            match result {
              Ok(msg) => {
                match msg {
                  MessageType::Register(name, typename) => {
                    println!("Registering variable {}", &name);
                    match register_stream(name, typename, socket, &mut stream_manager) {
                      Ok(_) => {},
                      Err(e) => println!("Failed to register log stream: {:?}", e)
                    };
                  },
                  MessageType::Log(id, val) => {
                    println!("Received log message for {}", &id);
                    match stream_manager.log(id, val) {
                      Ok(_) => {},
                      Err(e) => println!("Failed to write log message: {:?}", e)
                    }
                  }
                  _ => unreachable!("Sent an invalid message type to the logger service!")
                }
                break;
              }
              Err(e) => {
                match *e {
                  bincode::ErrorKind::Io(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Socket is not ready anymore, stop reading
                    break;
                  },
                  bincode::ErrorKind::Io(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    // Socket closed => Drop client
                    remove_socket = true;
                    break;
                  },
                  e => {
                    panic!("Unexpected socket error. {:?}", e);
                  }
                }
              }
            }
          }

          if remove_socket {
            println!("Dropping socket for token {:?}", &token);
            clients.remove(&token);
          }
        }
      }
    }
  }
}
