use std::time::Instant;

lazy_static! {
  static ref START_TIME: Instant = Instant::now();
}

pub fn milliseconds() -> f32 {
  let duration = START_TIME.elapsed();
  duration.as_secs() as f32 + ((duration.subsec_nanos() / 1000000) as f32 / 1000f32) as f32
}
