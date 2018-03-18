use error::*;

use std::rc::*;

use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;
use std::cell::RefCell;

// Register numbers specific to the PCA9685 PWM driver
const REG_MODE1: u8 = 0x00;
const REG_PRESCALER: u8 = 0xFE;
const REG_CHANNEL_BASE: u8 = 0x06;

const CHANNEL_STEERING: u8 = 0;
const CHANNEL_THROTTLE: u8 = 1;

fn calc_prescaler(freq: f32) -> u8 {
  (25_000_000f32 / 4096f32 / freq - 1f32).round() as u8
}

fn calc_channel_base_reg(channel: u8) -> u8 {
  REG_CHANNEL_BASE + (channel * 4)
}

fn calc_duty_cycle(percentage: f32) -> u16 {
  (percentage * 4096f32) as u16
}

pub struct PwmChannelArgs {
  /// Duty cycle in the range [0:1] for the neutral position for this channel.
  pub neutral: f32,

  /// Duty cycle range for this channel. If the sum of this value and the neutral value is set,
  /// then the channel shall assume it's extreme position
  pub range: f32,
}

pub struct PwmArgs {
  pub address: u16,               // I2C address of the PWM driver board
  pub freq: f32,                  // PWM frequency to use in [Hz]
  pub steering: PwmChannelArgs,   // Steering configuration
  pub throttle: PwmChannelArgs,   // Throttle / Brake configuration
}

pub struct PwmChannel {
  config: PwmChannelArgs,
  channel_num: u8,
  device: Weak<RefCell<LinuxI2CDevice>>,
}

pub struct PwmDriver {
  device: Rc<RefCell<LinuxI2CDevice>>,
  pub steering: Rc<RefCell<PwmChannel>>,
  pub throttle: Rc<RefCell<PwmChannel>>,
}

impl PwmChannel {
  fn new(device: Weak<RefCell<LinuxI2CDevice>>, args: PwmChannelArgs, channel_num: u8) -> Result<PwmChannel, I2CError> {
    let channel_base_reg = calc_channel_base_reg(channel_num);

    let dev = device.upgrade().expect("device reference invalid during PwmChannel construction!");
    let mut i2c = dev.borrow_mut();

    // Set the "on" register to 0 for this channel
    // (we expect the PWM signal to go high at the beginning of each cycle)
    i2c.smbus_write_word_data(channel_base_reg, 0x0000)?;

    // Set the channel to its neutral position
    i2c.smbus_write_word_data(channel_base_reg + 2, calc_duty_cycle(args.neutral))?;

    Ok(PwmChannel { config: args, channel_num, device })
  }

  /// Sets the position of the servo connected to this PWM channel
  /// value: The new position for the servo in the range [-1:1]. 0 Means neutral.
  /// device: The I2C device to set the value on
  pub fn set_value(&mut self, value: f32) -> Result<(), I2CError> {
    match self.device.upgrade() {
      Some(dev) => {
        let mut i2c = dev.borrow_mut();
        i2c.smbus_write_word_data(
          calc_channel_base_reg(self.channel_num) + 2,
          calc_duty_cycle(self.config.neutral + value * self.config.range))?;
        Ok(())
      }
      None => Err(I2CError::ReferenceInvalid)
    }
  }
}

impl PwmDriver {
  pub fn new(path: &str, args: PwmArgs) -> Result<PwmDriver, I2CError> {
    let device = Rc::new(RefCell::new(LinuxI2CDevice::new(path, args.address)?));
    {
      let mut i2c = device.borrow_mut();

      match i2c.smbus_read_byte_data(REG_MODE1) {
        // We don't really care about the actual register contents.
        // This read operation was just done to see whether we're able to communicate at all.
        Ok(_) => (),
        Err(e) => return Err(I2CError::Setup(SetupError::new(
          &format!("Could not communicate with PWM driver. Please check the connection [{:?}]", e))))
      };

      // Put the driver in sleep mode (the is required to configure the PWM frequency prescaler)
      i2c.smbus_write_byte_data(REG_MODE1, 0x11)?;

      // Configure the PWM frequency prescaler
      i2c.smbus_write_byte_data(REG_PRESCALER, calc_prescaler(args.freq))?;

      // Start normal device operation
      i2c.smbus_write_byte_data(REG_MODE1, 0x21)?;
    }

    // Initialize the two PWM channels we need to control our car
    let steering = Rc::new(RefCell::new(
      PwmChannel::new(Rc::downgrade(&device), args.steering, CHANNEL_STEERING)?));
    let throttle = Rc::new(RefCell::new(
      PwmChannel::new(Rc::downgrade(&device), args.throttle, CHANNEL_THROTTLE)?));

    Ok(PwmDriver {
      device,
      steering,
      throttle,
    })
  }
}

impl Drop for PwmDriver {
  fn drop(&mut self) {
    // Put the driver in sleep mode when we shut down
    println!("Shutting down PWM driver ...");
    self.device.borrow_mut().smbus_write_byte_data(REG_MODE1, 0x11).unwrap();
  }
}
