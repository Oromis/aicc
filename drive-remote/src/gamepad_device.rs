use input_device::InputDevice;
use inputs::Inputs;

use joy;

pub struct GamepadDevice {
  device: joy::Device,
}

impl GamepadDevice {
  pub fn new(dev: &str) -> GamepadDevice {
    let tmp = dev.to_string() + "\0";
    let device = joy::Device::open(tmp.as_bytes()).unwrap();
    GamepadDevice { device }
  }
}

impl InputDevice for GamepadDevice {
  fn poll(&mut self, inputs: &mut Inputs) {
    let (mut brake, mut throttle) = (0f32, 0f32);
    for ev in &mut self.device {
      use joy::Event::*;
      match ev {
        Axis(n, value) => {
          match n {
            0 => {
              inputs.steering = (value as f32) / (0x7FFF as f32);
//              println!("axis 0: {}\r", inputs.steering);
            }
            4 => {
              throttle = ((value as i32 + 0x7FFF_i32) as f32) / (0xFFFF as f32);
//              println!("axis 4: {}\r", inputs.throttle);
            }
            5 => {
              brake = -((value as i32 + 0x7FFF_i32) as f32) / (0xFFFF as f32);
//              println!("axis 5: {}\r", inputs.throttle);
            }
            _ => {}
          }
        },
        Button(_, _) => {}
      }
    }
    inputs.throttle = if brake.abs() > 0.05 {
      brake
    } else {
      throttle + brake
    };
  }
}
