extern crate i2cdev;

mod error;
mod pwm_driver;

use pwm_driver::*;

const PWM_DRIVER_ADDRESS: u16 = 0x40;
const PWM_FREQUENCY: f32 = 50f32;
const I2C_DEVICE_PATH: &str = "/dev/i2c-1";

fn main() {
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

  device.set_steering(0f32).unwrap();
  device.set_throttle(0f32).unwrap();
}
