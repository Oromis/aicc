AICC (Artificial Intelligence Controlled Car)
=============================================

Attempt at converting a normal RC car into an autonomous race car.

Uses: 

  - NVIDIA Jetson TX2 as the ECU
  - Adafruit PWM driver

Development Environmen:

I usually use CLion on Windows 10 and compile with clang. Perform the following steps to set it up:

  - 
  - Open CLion and clone this project.
  - Install rust (using rustup)
  - Install Jetson TX2 Cross Compiler (from https://developer.nvidia.com/embedded/downloads)
  - Install Rust cross compiler target: `rustup target add aarch64-unknown-linux-gnu`
  - Add the following to `~/.cargo/config`:
      ```
      [target.aarch64-unknown-linux-gnu]
      linker = "aarch64-unknown-linux-gnu-gcc"
      ```