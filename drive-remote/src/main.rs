extern crate termion;
extern crate nix;
extern crate byteorder;
extern crate messages;
extern crate serde;
extern crate bincode;
extern crate clap;
extern crate joy;

mod input_device;
mod inputs;
mod keyboard_device;
mod gamepad_device;

use std::thread;
use std::time;
use std::net::{ TcpStream };
use std::io::{ Write };

use clap::{ Arg, App };
use bincode::serialize;

use messages::drive_core::MessageType;
use input_device::InputDevice;
use keyboard_device::KeyboardDevice;
use gamepad_device::GamepadDevice;

const MIN_SEND_INTERVAL: time::Duration = time::Duration::from_millis(50);

fn main() {
  let matches = App::new("drive-remote")
    .author("David Bauske <david.bauske@googlemail.com>")
    .about("Direct remote control for the AICC car project. Use the keyboard to control your car!")
    .arg(Arg::with_name("keyboard")
      .help("Sets the keyboard device (from /dev/input) to use for keyboard control")
      .short("k")
      .long("keyboard")
      .takes_value(true)
    )
    .arg(Arg::with_name("gamepad")
      .help("Sets the gamepad device (from /dev/input) to use for gamepad control")
      .short("g")
      .long("gamepad")
      .takes_value(true)
    )
    .arg(Arg::with_name("host")
      .short("h")
      .long("host")
      .help("Specify the host to connect to (must run drive-core)")
      .default_value("localhost")
      .takes_value(true)
    )
    .arg(Arg::with_name("speed")
      .short("s")
      .long("speed")
      .help("Sets the maximum motor speed. Must be a value between 0 and 1.")
      .default_value("1")
      .takes_value(true)
    )
    .get_matches();

  let mut inputs = inputs::Inputs { steering: 0f32, throttle: 0f32, running: true };
  let mut devices: Vec<Box<InputDevice>> = Vec::new();

  // Open the keyboard event device
  if let Some(device) = matches.value_of("keyboard") {
    devices.push(Box::new(KeyboardDevice::new(device)));
  }

  // Open the gamepad event device
  if let Some(device) = matches.value_of("gamepad") {
    devices.push(Box::new(GamepadDevice::new(device)));
  }

  if devices.len() == 0 {
    println!("Please specify either keyboard or gamepad!");
    std::process::exit(1);
  }

  let mut host = matches.value_of("host").unwrap().to_owned();

  // Add the default port (AICC 0 in leetspeak)
  host.push_str(":41330");

  println!("Connecting to {}", &host);

  let mut socket = TcpStream::connect(host).unwrap();

  let speed_factor: f32 = matches.value_of("speed").unwrap().parse()
    .expect("Invalid number given for speed.");

  println!("Connected to AICC.\n\
    Use the arrow keys to remote-control the car.\n\
    Your motor uses a speed factor of {}.\n\
    Stop the program using Ctrl+C.\n\
    Have fun! :)", speed_factor);

  let mut last_send_time = time::Instant::now();

  while inputs.running {
    // Query devices
    for device in &mut devices {
      device.poll(&mut inputs);
    }

    // Make sure we don't send messages too quickly (20Hz should be fine)
    let now = time::Instant::now();
    let delta = now - last_send_time;
    if delta < MIN_SEND_INTERVAL {
      thread::sleep(MIN_SEND_INTERVAL - delta);
    }
    last_send_time = time::Instant::now();

    // Send the steering value
    let steering_msg = serialize(&MessageType::SetSteering(inputs.steering)).unwrap();
    socket.write(&steering_msg[..]).unwrap();

    let throttle_msg = serialize(&MessageType::SetThrottle(inputs.throttle * speed_factor)).unwrap();
    socket.write(&throttle_msg[..]).unwrap();
  }

  // Clean shutdown => Send Bye message
  let bye_msg = serialize(&MessageType::Bye).unwrap();
  socket.write(&bye_msg[..]).unwrap();
}
