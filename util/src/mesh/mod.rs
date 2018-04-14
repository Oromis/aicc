// Contains information about the organization of the mesh of microservices that makes up the AICC
pub enum Service {
  DriveCore,
  Logger,
}

impl Service {
  pub fn port(&self) -> u16 {
    match *self {
      Service::DriveCore => 41330,
      Service::Logger => 41331,
    }
  }

  pub fn name(&self) -> &str {
    match *self {
      Service::DriveCore => "drive-core",
      Service::Logger => "logger",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn port_returns_the_correct_ports() {
    assert_eq!(41330, Service::DriveCore.port());
    assert_eq!(41331, Service::Logger.port());
  }

  #[test]
  fn name_returns_the_correct_names() {
    assert_eq!("drive-core", Service::DriveCore.name());
    assert_eq!("logger", Service::Logger.name());
  }
}
