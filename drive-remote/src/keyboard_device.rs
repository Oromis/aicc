use input_device::InputDevice;
use inputs::Inputs;

use std::io::{ stdout, Cursor };
use std::os::unix::io::RawFd;

use byteorder::{ ReadBytesExt, LittleEndian };

use nix::fcntl::*;
use nix::sys::stat::Mode;
use nix::unistd;
use nix::Error;
use nix::errno::Errno;

use termion::raw::IntoRawMode;

const KEY_CODE_LEFT: u16 = 105;
const KEY_CODE_RIGHT: u16 = 106;
const KEY_CODE_UP: u16 = 103;
const KEY_CODE_DOWN: u16 = 108;
const KEY_CODE_CTRL: u16 = 29;
const KEY_CODE_C: u16 = 46;

pub struct KeyboardDevice {
  fd: RawFd,
  ctrl_down: bool,
}

impl KeyboardDevice {
  pub fn new(dev: &str) -> KeyboardDevice {
    let mut flags = OFlag::empty();
    flags.insert(OFlag::O_RDONLY);
    flags.insert(OFlag::O_NONBLOCK);
    let fd = open(dev, flags, Mode::empty()).unwrap();

    // Disables echoing as long as this object lives (until the end of main())
    let _stdout = stdout().into_raw_mode().unwrap();

    KeyboardDevice { fd, ctrl_down: false }
  }
}

impl InputDevice for KeyboardDevice {
  fn poll(&mut self, inputs: &mut Inputs) {
    let mut read = 1;
    while read > 0 {
      read = 0;

      // A linux input event structure is exactly 24 byte long
      let mut buffer = [0; 24];
      // Read keyboard events from the keyboard device
      match unistd::read(self.fd, &mut buffer) {
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
                  KEY_CODE_LEFT => inputs.steering = if key_down { -1f32 } else { 0f32 },
                  KEY_CODE_RIGHT => inputs.steering = if key_down { 1f32 } else { 0f32 },
                  KEY_CODE_UP => inputs.throttle = if key_down { 1f32 } else { 0f32 },
                  KEY_CODE_DOWN => inputs.throttle = if key_down { -1f32 } else { 0f32 },

                  // Handle Ctrl+C to stop the program
                  KEY_CODE_CTRL => self.ctrl_down = key_down,
                  KEY_CODE_C => if self.ctrl_down { inputs.running = false; },
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
  }
}

impl Drop for KeyboardDevice {
  fn drop(&mut self) {
    unistd::close(self.fd).unwrap();
  }
}
