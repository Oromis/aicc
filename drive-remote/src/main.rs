extern crate termion;
extern crate nix;
extern crate byteorder;
extern crate messages;
extern crate serde;
extern crate bincode;

use std::env;
use std::thread;
use std::time;
use std::net::{ TcpStream };
use std::io::{ stdout, Cursor, Write };

use termion::raw::IntoRawMode;

use nix::fcntl::*;
use nix::sys::stat::Mode;
use nix::unistd;
use nix::Error;
use nix::errno::Errno;

use byteorder::{ ReadBytesExt, LittleEndian };
use bincode::serialize;

use messages::drive_core::MessageType;

const KEY_CODE_LEFT: u16 = 105;
const KEY_CODE_RIGHT: u16 = 106;
const KEY_CODE_UP: u16 = 103;
const KEY_CODE_DOWN: u16 = 108;
const KEY_CODE_CTRL: u16 = 29;
const KEY_CODE_C: u16 = 46;

const MIN_SEND_INTERVAL: time::Duration = time::Duration::from_millis(50);

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    println!("Usage: {} <keyboard-device> [<host>]", args[0]);
    return;
  }

  let ref host = if args.len() >= 3 {
    &args[2]
  } else {
    "localhost:41330"
  };

  println!("Connecting to {}", host);

  let mut socket = TcpStream::connect(host).unwrap();

  // Disables echoing as long as this object lives (until the end of main())
  let _stdout = stdout().into_raw_mode().unwrap();

  // Open the keyboard event device
  let mut flags = OFlag::empty();
  flags.insert(OFlag::O_RDONLY);
  flags.insert(OFlag::O_NONBLOCK);
  let fd = open(args[1].as_str(), flags, Mode::empty()).unwrap();

  let mut throttle = 0f32;
  let mut steering = 0f32;
  let mut last_send_time = time::Instant::now();
  let mut ctrl_down = false;
  let mut running = true;

  while running {
    let mut read = 1;
    while read > 0 {
      read = 0;

      // A linux input event structure is exactly 24 byte long
      let mut buffer = [0; 24];
      // Read keyboard events from the keyboard device
      match unistd::read(fd, &mut buffer) {
        Ok(count) => {
          if count == 24 {
            read = count;
            let mut cursor = Cursor::new(buffer);
            cursor.read_u64::<LittleEndian>().unwrap(); // Discard 8 bytes for UNIX timestamp
            cursor.read_u64::<LittleEndian>().unwrap(); // Discard 8 bytes for Nanoseconds part
            let event = cursor.read_u16::<LittleEndian>().unwrap();
            let code = cursor.read_u16::<LittleEndian>().unwrap();
            let value = cursor.read_i32::<LittleEndian>().unwrap();

            // Event = 1 means that it's either a keydown, keyup or keyrepeat event
            if event == 1 {
              if value == 0 || value == 1 {
                // Keydown or Keyup
                let key_down = value == 1;
                match code {
                  KEY_CODE_LEFT => steering = if key_down { -1f32 } else { 0f32 },
                  KEY_CODE_RIGHT => steering = if key_down { 1f32 } else { 0f32 },
                  KEY_CODE_UP => throttle = if key_down { 1f32 } else { 0f32 },
                  KEY_CODE_DOWN => throttle = if key_down { -1f32 } else { 0f32 },

                  // Handle Ctrl+C to stop the program
                  KEY_CODE_CTRL => ctrl_down = key_down,
                  KEY_CODE_C => if ctrl_down { running = false; },
                  _ => {},
                }
              }
            }
          }
        }
        Err(e) => {
          read = 0;
          match e {
            Error::Sys(Errno::EAGAIN) => {},
            err => println!("read failed: {:?}", err)
          }
        }
      };
    }

    // Make sure we don't send messages too quickly (20Hz should be fine)
    let now = time::Instant::now();
    let delta = now - last_send_time;
    if delta < MIN_SEND_INTERVAL {
      thread::sleep(MIN_SEND_INTERVAL - delta);
    }
    last_send_time = time::Instant::now();

    // Send the steering value
    let steering_msg = serialize(&MessageType::SetSteering(steering)).unwrap();
    socket.write(&steering_msg[..]).unwrap();

    let throttle_msg = serialize(&MessageType::SetThrottle(throttle)).unwrap();
    socket.write(&throttle_msg[..]).unwrap();
//    println!("Throttle: {}, Steering: {}\r", throttle, steering);
  }

  unistd::close(fd).unwrap();
}
