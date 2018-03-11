use error::*;

use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;

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
}

pub struct PwmDriver {
  device: LinuxI2CDevice,
  steering: PwmChannel,
  throttle: PwmChannel,
}

impl PwmChannel {
  fn new(device: &mut LinuxI2CDevice, args: PwmChannelArgs, channel_num: u8) -> Result<PwmChannel, I2CError> {
    let channel_base_reg = calc_channel_base_reg(channel_num);

    // Set the "on" register to 0 for this channel
    // (we expect the PWM signal to go high at the beginning of each cycle)
    device.smbus_write_word_data(channel_base_reg, 0x0000)?;

    // Set the channel to its neutral position
    device.smbus_write_word_data(channel_base_reg + 2, calc_duty_cycle(args.neutral))?;

    Ok(PwmChannel { config: args, channel_num })
  }

  /// Sets the position of the servo connected to this PWM channel
  /// value: The new position for the servo in the range [-1:1]. 0 Means neutral.
  /// device: The I2C device to set the value on
  fn set_value(&self, value: f32, device: &mut LinuxI2CDevice) -> Result<(), I2CError> {
    device.smbus_write_word_data(
      calc_channel_base_reg(self.channel_num) + 2,
      calc_duty_cycle(self.config.neutral + value * self.config.range))?;
    Ok(())
  }
}

impl PwmDriver {
  pub fn new(path: &str, args: PwmArgs) -> Result<PwmDriver, I2CError> {
    let mut device = LinuxI2CDevice::new(path, args.address)?;

    match device.smbus_read_byte_data(REG_MODE1) {
      // We don't really care about the actual register contents.
      // This read operation was just done to see whether we're able to communicate at all.
      Ok(_) => (),
      Err(e) => return Err(I2CError::Setup(SetupError::new(
        &format!("Could not communicate with PWM driver. Please check the connection [{:?}]", e))))
    };

    // Put the driver in sleep mode (the is required to configure the PWM frequency prescaler)
    device.smbus_write_byte_data(REG_MODE1, 0x11)?;

    // Configure the PWM frequency prescaler
    device.smbus_write_byte_data(REG_PRESCALER, calc_prescaler(args.freq))?;

    // Start normal device operation
    device.smbus_write_byte_data(REG_MODE1, 0x21)?;

    // Initialize the two PWM channels we need to control our car
    let steering = PwmChannel::new(&mut device, args.steering, CHANNEL_STEERING)?;
    let throttle = PwmChannel::new(&mut device, args.throttle, CHANNEL_THROTTLE)?;

    Ok(PwmDriver {
      device,
      steering,
      throttle,
    })
  }

  /// Sets the steering angle of the car.
  /// value: The steering angle in the range [-1:1].
  ///         0 means straight, -1 means full left, 1 means full right
  pub fn set_steering(&mut self, value: f32) -> Result<(), I2CError> {
    self.steering.set_value(-value, &mut self.device)
  }

  /// Sets the throttle / brake value of the car
  /// value: The new value for the car's propulsion system in the range [-1:1]
  ///         0 means neither throttle nor brake,
  ///         1 is full throttle,
  ///         -1 is full braking (or reverse if the car was not moving forward)
  pub fn set_throttle(&mut self, value: f32) -> Result<(), I2CError> {
    self.throttle.set_value(value, &mut self.device)
  }
}

impl Drop for PwmDriver {
  fn drop(&mut self) {
    // Put the driver in sleep mode when we shut down
    println!("Shutting down PWM driver ...");
    self.device.smbus_write_byte_data(REG_MODE1, 0x11).unwrap();
  }
}
