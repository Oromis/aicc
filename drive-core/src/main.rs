extern crate i2cdev;
extern crate ctrlc;

extern crate serde;
extern crate bincode;
extern crate bufstream;

extern crate messages;
extern crate util;

mod error;
mod pwm_driver;

use pwm_driver::*;
use messages::drive_core::MessageType;
use util::variable::Variable;
use util::logging::LogConnection;
use util::mesh::Service;

use std::net::*;
use std::io;
use std::rc::*;
use std::cell::RefCell;
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

fn connect_variable_with_channel(var: & mut Variable<f32>, channel: &mut Rc<RefCell<PwmChannel>>, prescaler: f32) {
  let weak_ptr = Rc::downgrade(channel);
  var.add_listener(move |val| {
    match weak_ptr.upgrade() {
      Some(rc) => rc.borrow_mut().set_value(*val * prescaler).unwrap(),
      None => println!("Failed to de-reference the weak reference to the PWM channel :|")
    };
    Ok(())
  });
}

fn main() {
  let mut steering = Variable::new(0f32);
  let mut throttle = Variable::new(0f32);

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

  let mut log_connection = LogConnection::new().unwrap();
  log_connection.log_variable(&mut steering, "drive-core_steering").unwrap();
  log_connection.log_variable(&mut throttle, "drive-core_throttle").unwrap();

  connect_variable_with_channel(&mut steering, &mut device.steering, -1_f32);
  connect_variable_with_channel(&mut throttle, &mut device.throttle, 1_f32);

  steering.add_listener(|v| { println!("steering: {}", v); Ok(()) });
  throttle.add_listener(|v| { println!("throttle: {}", v); Ok(()) });

  // Set up a listening network socket to receive new connections on.
  // The drive-core service is controlled by such network
  // connections only (they may come from localhost).
  let listener = TcpListener::bind("0.0.0.0:".to_string() + &Service::DriveCore.port().to_string())
    .unwrap();
  listener.set_nonblocking(true).unwrap();

  // Set a handler to listen for the Ctrl+C key sequence and perform a
  // clean shutdown if it fires.
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
              steering.set_value(0f32);
              throttle.set_value(0f32);
              continue;
            }
            e => {
              println!("Reading from socket failed. Dropping client. {:?}", e);
              steering.set_value(0f32);
              throttle.set_value(0f32);
              break;
            }
          }
        }
      };

      // Handle the message
      match msg {
        MessageType::SetSteering(val) => steering.set_value(val),
        MessageType::SetThrottle(val) => throttle.set_value(val),
        MessageType::Bye => {
          println!("Client logging out.");
          steering.set_value(0f32);
          throttle.set_value(0f32);
          break;
        },
      };
    }
  }
}

