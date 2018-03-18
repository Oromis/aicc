extern crate i2cdev;
extern crate ctrlc;

extern crate serde;
extern crate bincode;
extern crate bufstream;

extern crate messages;
extern crate util;

mod error;
mod pwm_driver;
mod variable;

use pwm_driver::*;
use variable::Variable;
use messages::drive_core::MessageType;
use util::logging::{ self, LogStream };

use std::net::*;
use std::io;
use std::rc::*;
use std::time::Duration;
use std::sync::atomic::{ AtomicBool, Ordering };
use bufstream::BufStream;
use bincode::deserialize_from;
use std::cell::RefCell;

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

fn create_log_for(var: &mut Variable<f32>, name: &str) -> io::Result<()> {
  let path = logging::get_timestamped_path()?;
  let mut log_stream: LogStream<f32> = LogStream::new(
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

fn connect_variable_with_channel(var: & mut Variable<f32>, channel: &mut Rc<RefCell<PwmChannel>>) {
  let weak_ptr = Rc::downgrade(channel);
  var.add_listener(move |val| {
    match weak_ptr.upgrade() {
      Some(rc) => rc.borrow_mut().set_value(*val).unwrap(),
      None => println!("Failed to de-reference the weak reference to the PWM channel :|")
    }
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


  create_log_for(&mut steering, "steering").unwrap();
  create_log_for(&mut throttle, "throttle").unwrap();

  connect_variable_with_channel(&mut steering, &mut device.steering);
  connect_variable_with_channel(&mut throttle, &mut device.throttle);

  steering.add_listener(|v| println!("steering: {}", v));
  throttle.add_listener(|v| println!("throttle: {}", v));

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
          break;
        },
      };
    }
  }
}

