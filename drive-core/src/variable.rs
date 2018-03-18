pub struct Variable<'a, T> where T: PartialEq {
  value: T,
  listeners: Vec<Box<FnMut(&T) + 'a>>,
}

impl<'a, T> Variable<'a, T> where T: PartialEq {
  pub fn new(value: T) -> Variable<'a, T> {
    Variable { value, listeners: Vec::new() }
  }

  pub fn add_listener<L>(&mut self, listener: L) where L: FnMut(&T) + 'a {
    self.listeners.push(Box::new(listener));
  }

  pub fn set_value(&mut self, val: T) {
    if self.value != val {
      for listener in &mut self.listeners {
        listener(&val);
      }
      self.value = val;
    }
  }

  pub fn value(&self) -> &T {
    &self.value
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::cell::Cell;

  #[test]
  fn test_callbacks() {
    let calls = Cell::new(0);
    let mut var = Variable::new(42_f32);

    var.add_listener(|_| calls.set(calls.get() + 1));

    var.set_value(42_f32);
    assert_eq!(0, calls.get());

    var.set_value(12.34_f32);
    assert_eq!(1, calls.get());

    var.add_listener(|v| assert_eq!(-42_f32, *v));

    var.set_value(12.34_f32);
    assert_eq!(1, calls.get());

    var.set_value(-42_f32);
    assert_eq!(2, calls.get());
  }

  #[test]
  fn test_value() {
    let mut var = Variable::new(1_f32);

    {
      // Test getting the value of an immutable reference works
      let reference = &var;
      assert_eq!(1_f32, *reference.value());
    }

    var.set_value(42_f32);
    assert_eq!(42_f32, *var.value());
  }
}