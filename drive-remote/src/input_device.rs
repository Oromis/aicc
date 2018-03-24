use inputs::Inputs;

pub trait InputDevice {
  fn poll(&mut self, inputs: &mut Inputs);
}
