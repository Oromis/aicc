extern crate i2cdev;
extern crate ctrlc;

extern crate serde;
extern crate bincode;
extern crate bufstream;

extern crate messages;

mod error;
mod pwm_driver;

use pwm_driver::*;
use messages::drive_core::MessageType;

use std::net::*;
use std::io;
use std::time::Duration;
use std::sync::atomic::{ AtomicBool, Ordering };
use bufstream::BufStream;
use bincode::deserialize_from;

const PWM_DRIVER_ADDRESS: u16 = 0x40;
const PWM_FREQUENCY: f32 = 50f32;
const I2C_DEVICE_PATH: &str = "/dev/i2c-1";

static RUNNING: AtomicBool = AtomicBool::new(true);

fn accept_connection(listener: &TcpListener) -> io::Result<BufStream<TcpStream>> {
  let (socket, addr) = listener.accept()?;
  println!("Client connected from {}", addr);

  // We don't want to block longer than 100ms. If we go above this threshold, then we consider
  // the client unresponsive and cut the power to the motor.
  socket.set_read_timeout(Option::Some(Duration::from_millis(100)))?;

  return Ok(BufStream::new(socket));
}

fn main() {
  // Create and initialize the PWM driver. It's constructor
  // will set both servos to their respective neutral position.
  let mut device = PwmDriver::new(I2C_DEVICE_PATH, PwmArgs {
    address: PWM_DRIVER_ADDRESS,
    freq: PWM_FREQUENCY,
    steering: PwmChannelArgs {
      neutral: 0.085f32,
      range: 0.01625f32,
    },
    throttle: PwmChannelArgs {
      neutral: 0.075f32,
      range: 0.025f32,
    }
  }).unwrap();

  // Set up a listening network socket to receive new connections on.
  // The drive-core service is controlled by such network
  // connections only (they may come from localhost).
  let listener = TcpListener::bind("0.0.0.0:41330").unwrap();
  listener.set_nonblocking(true).unwrap();

  // Set a handler to listen for the Ctrl+C key sequence and perform a
  // clean shutdown if it happens.
  ctrlc::set_handler(move || {
    RUNNING.store(false, Ordering::SeqCst);
  }).unwrap();

  println!("Setup complete. Waiting for connection.");

  while RUNNING.load(Ordering::Acquire) {
    std::thread::sleep(std::time::Duration::from_millis(10));
    let mut socket = match accept_connection(&listener) {
      Ok(socket) => socket,
      Err(e) => {
        if e.kind() != io::ErrorKind::WouldBlock {
          println!("Failed to accept incoming socket: {:?}", e);
        }
        continue
      }
    };

    while RUNNING.load(Ordering::Acquire) {
      // Repeatedly read messages from the socket. If the socket fails,
      // then we'll drop the client and wait for a new one
      let msg: MessageType = match deserialize_from(&mut socket) {
        Ok(msg) => msg,
        Err(e) => {
          match *e {
            bincode::ErrorKind::Io(ref e) if e.kind() == io::ErrorKind::TimedOut
                || e.kind() == io::ErrorKind::WouldBlock => {
              // Timeout => Disable power
              device.set_steering(0f32).unwrap();
              device.set_throttle(0f32).unwrap();
              continue;
            }
            e => {
              println!("Reading from socket failed. Dropping client. {:?}", e);
              device.set_steering(0f32).unwrap();
              device.set_throttle(0f32).unwrap();
              break;
            }
          }
        }
      };

      // Handle the message
      match msg {
        MessageType::SetSteering(val) => device.set_steering(val),
        MessageType::SetThrottle(val) => device.set_throttle(val),
        MessageType::Bye => {
          println!("Client logging out.");
          break;
        },
      }.unwrap();
    }
  }
}

