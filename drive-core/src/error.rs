use std::error::Error;
use std::fmt;

use i2cdev::linux::LinuxI2CError;

#[derive(Debug)]
pub struct SetupError {
  msg: String
}

impl SetupError {
  pub fn new(msg: &str) -> SetupError {
    SetupError { msg: String::from(msg) }
  }
}

impl Error for SetupError {
  fn description(&self) -> &str {
    &self.msg
  }
}

impl fmt::Display for SetupError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Setup error: {}", self.msg)
  }
}

#[derive(Debug)]
pub enum I2CError {
  I2CDevice(LinuxI2CError),
  Setup(SetupError)
}

impl From<LinuxI2CError> for I2CError {
  fn from(e: LinuxI2CError) -> Self {
    I2CError::I2CDevice(e)
  }
}

impl From<SetupError> for I2CError {
  fn from(e: SetupError) -> Self {
    I2CError::Setup(e)
  }
}